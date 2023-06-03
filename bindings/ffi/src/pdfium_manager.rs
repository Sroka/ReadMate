use std::cmp::{max, min};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use std::thread::JoinHandle;
use anyhow::{Context, Result};
use pdfium_render::metadata::PdfDocumentMetadataTagType;
use pdfium_render::prelude::*;
use crate::global_state::GlobalResult;

use uuid::Uuid;
use crate::domain::{Bitmap, Page};

pub fn generate_pdf_uuid() -> String {
    Uuid::new_v4().to_string()
}

pub struct PdfiumManager {
    pub pdfium_action_sender: Mutex<Sender<PdfiumAction>>,
    pub pdfium_thread_handle: JoinHandle<()>,
}

impl PdfiumManager {
    pub fn new(global_action_sender: Arc<Mutex<Sender<GlobalResult>>>) -> PdfiumManager {
        let (action_sender, action_receiver): (Sender<PdfiumAction>, Receiver<PdfiumAction>) = channel();
        let pdfium_thread_handle = thread::spawn(move || {
            let pdfium_bindings = Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path("./"))
                .or_else(|_| Pdfium::bind_to_system_library())
                .unwrap();
            let pdfium = Pdfium::new(pdfium_bindings);
            let mut current_pdfium_document: Option<PdfDocument> = None;
            let mut current_document_pages: HashMap<i32, Arc<Bitmap>> = HashMap::new();
            loop {
                let action = action_receiver.recv().unwrap();
                match action {
                    PdfiumAction::LoadPdf { uuid, file_name, bytes } => {
                        let result = pdfium.load_pdf_from_byte_slice(Box::leak(Box::new(bytes)), None);
                        match result {
                            Ok(pdf) => {
                                let metadata = pdf.metadata();
                                let title = metadata
                                    .get(PdfDocumentMetadataTagType::Title)
                                    .map(|item| item.value().to_string())
                                    .unwrap_or("".to_string());
                                let author = metadata
                                    .get(PdfDocumentMetadataTagType::Author)
                                    .map(|item| item.value().to_string())
                                    .unwrap_or("".to_string());
                                let display_title = if title.is_empty() {
                                    file_name
                                } else {
                                    title
                                };
                                let page_count: i32 = pdf.pages().len().into();
                                let thumbnail = get_thumbnail(&pdf).ok();
                                global_action_sender
                                    .lock()
                                    .unwrap()
                                    .send(GlobalResult::PdfLoaded {
                                        id: uuid.clone(),
                                        title: display_title,
                                        author,
                                        thumbnail,
                                        page_count,
                                    })
                                    .unwrap();
                                current_pdfium_document = Some(pdf);
                                current_document_pages.clear();
                            }
                            Err(error) => {
                                error!("Loading pdf failed: {error}");
                                global_action_sender
                                    .lock()
                                    .unwrap()
                                    .send(GlobalResult::PdfLoadingFailed { uuid: uuid.clone() })
                                    .unwrap();
                            }
                        }
                    }
                    PdfiumAction::PageLoadRequested { page_index: index } => {
                        info!("PdfiumAction::PageLoadRequested");
                        current_pdfium_document = match current_pdfium_document.take() {
                            None => None,
                            Some(pdf) => {
                                let pages = pdf.pages();
                                let pages_count = pages.len() as i32;
                                let page_indices_to_load: Vec<i32> = (max(0, index - 5)..min(index + 5, pages_count))
                                    .filter(|key| !current_document_pages.contains_key(key))
                                    .collect();
                                let mut rendered_pages: Vec<Arc<Page>> = vec![];
                                for page_index_to_load in page_indices_to_load {
                                    let page_index = page_index_to_load as u16;
                                    info!("PdfiumAction::PageLoadRequested - pages size - {pages_count} - requested index - {page_index_to_load}");
                                    let page_result = pages.get(page_index);
                                    match page_result {
                                        Ok(page) => {
                                            let image = get_page_image(page, 1000);
                                            match image {
                                                Ok(page_image) => {
                                                    rendered_pages.push( Arc::new(Page { index: page_index_to_load, image: Some(page_image) } ));
                                                }
                                                Err(error) => {
                                                    error!("PdfiumAction::PageLoadRequested - error rendering page - {error}")
                                                }
                                            }
                                        }
                                        Err(error) => {
                                            error!("PdfiumAction::PageLoadRequested - error loading page - {error}")
                                        }
                                    }
                                }
                                for page in &rendered_pages {
                                    let image = page.image();
                                    if let Some(image) = image {
                                        current_document_pages.insert(page.index, Arc::clone(&image));
                                    };
                                }
                                global_action_sender
                                    .lock()
                                    .unwrap()
                                    .send(GlobalResult::PagesLoaded {
                                        pages: rendered_pages,
                                    })
                                    .unwrap();
                                Some(pdf)
                            }
                        }
                    }
                }
            }
        });
        PdfiumManager {
            pdfium_action_sender: Mutex::new(action_sender),
            pdfium_thread_handle,
        }
    }
}

fn get_thumbnail(pdf: &PdfDocument) -> Result<Arc<Bitmap>> {
    let first_page = pdf.pages().get(0)?;
    get_page_image(first_page, 1000)
}

fn get_page_image(page: PdfPage, max_width: u16) -> Result<Arc<Bitmap>> {
    let width = max_width;
    let page_ratio = (page.height().value / page.width().value) as u16;
    let height = max_width * page_ratio;
    let pdf_bitmap = page.render(width, height, None)?;
    let pdf_bitmap: Vec<u32> = pdf_bitmap
        .as_bytes()
        .chunks(4)
        .map(|pixel| {
            let a = u32::from(pixel[3]) << 24;
            let r = u32::from(pixel[0]) << 16;
            let g = u32::from(pixel[1]) << 8;
            let b = u32::from(pixel[2]);
            let argb: u32 = a | r | g | b;
            argb
        })
        .collect();
    let bitmap_uid = Uuid::new_v4().to_string();
    Ok(Bitmap::new(width.into(), height.into(), bitmap_uid, pdf_bitmap))
}


pub enum PdfiumAction {
    LoadPdf { uuid: String, file_name: String, bytes: Vec<u8> },
    PageLoadRequested { page_index: i32 },
}
