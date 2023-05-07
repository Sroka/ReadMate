use std::sync::{Arc, Mutex};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;
use std::thread::JoinHandle;
use pdfium_render::metadata::PdfDocumentMetadataTagType;
use pdfium_render::prelude::{PdfDocument, Pdfium};
use crate::global_state::{GlobalAction};

use uuid::Uuid;

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
                    PdfiumAction::LoadPdf { file_name, bytes } => {
                        let pdf_uuid = Uuid::new_v4();
                        global_action_sender
                            .lock()
                            .unwrap()
                            .send(GlobalAction::PdfLoading { uuid: pdf_uuid.to_string() })
                            .unwrap();
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
                                    .unwrap_or("".to_string())
                                    ;
                                let display_title = if title.is_empty() {
                                    file_name
                                } else {
                                    title
                                };
                                current_pdfium_document = Some(pdf);
                                global_action_sender
                                    .lock()
                                    .unwrap()
                                    .send(GlobalAction::PdfLoaded {
                                        title: display_title,
                                        author,
                                        uuid: pdf_uuid.to_string(),
                                    })
                                    .unwrap();
                            }
                            Err(_) => {
                                global_action_sender
                                    .lock()
                                    .unwrap()
                                    .send(GlobalAction::PdfLoadingFailed { uuid: pdf_uuid.to_string() })
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


pub enum PdfiumAction {
    LoadPdf { file_name: String, bytes: Vec<u8> }
}
