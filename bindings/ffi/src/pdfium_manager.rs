use std::sync::{Arc, Mutex};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use std::thread::JoinHandle;
use anyhow::{Result, Context};
use pdfium_render::metadata::PdfDocumentMetadataTagType;
use pdfium_render::prelude::*;
use crate::global_state::{Bitmap, GlobalAction};

use uuid::Uuid;

pub fn generate_pdf_uuid() -> String {
    Uuid::new_v4().to_string()
}

pub struct PdfiumManager {
    pub pdfium_action_sender: Mutex<Sender<PdfiumAction>>,
    pub pdfium_thread_handle: JoinHandle<()>,
}

impl PdfiumManager {
    pub fn new(global_action_sender: Arc<Mutex<Sender<GlobalAction>>>) -> PdfiumManager {
        let (action_sender, action_receiver): (Sender<PdfiumAction>, Receiver<PdfiumAction>) = channel();
        let pdfium_thread_handle = thread::spawn(move || {
            let pdfium_bindings = Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path("./"))
                .or_else(|_| Pdfium::bind_to_system_library())
                .unwrap();
            let pdfium = Pdfium::new(pdfium_bindings);
            let mut current_pdfium_document: Option<PdfDocument> = None;
            loop {
                let action = action_receiver.recv().unwrap();
                match action {
                    PdfiumAction::LoadPdf { uuid, file_name, bytes } => {
                        let result = pdfium.load_pdf_from_byte_slice(Box::leak(Box::new(bytes)), None);
                        match result {
                            Ok(pdf) => {
                                current_pdfium_document = Some(pdf);
                                match current_pdfium_document {
                                    None => {}
                                    Some(pdf) => {
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
                                        let thumbnail = get_thumbnail(pdf).ok();
                                        global_action_sender
                                            .lock()
                                            .unwrap()
                                            .send(GlobalAction::PdfLoaded {
                                                uuid: uuid.clone(),
                                                title: display_title,
                                                author,
                                                thumbnail,
                                            })
                                            .unwrap();
                                    }
                                }
                            }
                            Err(error) => {
                                error!("Loading pdf failed: {error}");
                                global_action_sender
                                    .lock()
                                    .unwrap()
                                    .send(GlobalAction::PdfLoadingFailed { uuid: uuid.clone() })
                                    .unwrap();
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

fn get_thumbnail(pdf: PdfDocument) -> Result<Arc<Bitmap>> {
    let first_page = pdf.pages().get(0)?;
    let width = 1000;
    let height = 1410;
    let pdf_bitmap = first_page.render(width, height, None)?;
    let pdf_bitmap: Vec<u32> = pdf_bitmap
        .as_bytes()
        .chunks(4)
        .map(|pixel| {
            let a = u32::from(pixel[3]) << 24;
            let r = u32::from(pixel[0]) << 16;
            let g = u32::from(pixel[1]) << 8;
            let b = u32::from(pixel[2]);
            let argb: u32 = a |r | g | b;
            argb
        })
        .collect();
    let bitmap_uid = Uuid::new_v4().to_string();
    Ok(Bitmap::new(width.into(), height.into(), bitmap_uid, pdf_bitmap))
}


pub enum PdfiumAction {
    LoadPdf { uuid: String, file_name: String, bytes: Vec<u8> }
}
