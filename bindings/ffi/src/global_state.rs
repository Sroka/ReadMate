use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{channel, Receiver, Sender};
use pdfium_render::prelude::*;
use log::LevelFilter;
use android_logger::Config;

use std::thread;
use std::thread::JoinHandle;
use anyhow::{Result, Context};
use crate::pdfium_manager::{PdfiumManager, PdfiumAction};


#[derive(Clone)]
pub struct GlobalState {
    pub some_text: String,
    pub books: Vec<Book>,
}

#[derive(Clone, PartialEq)]
pub struct Book {
    pub uuid: String,
    pub thumbnail: Option<Arc<Bitmap>>,
    pub loading_state: PdfLoadingState,
}

#[derive(Clone, PartialEq)]
pub enum PdfLoadingState {
    LoadingPdf,
    ValidPdf {
        title: String,
        author: String,
        thumbnail: Option<Arc<Bitmap>>,
    },
    ErrorPdf,
}

pub enum GlobalAction {
    MarkPdfLoading { uuid: String },
    LoadPdf { uuid: String, file_name: String, bytes: Vec<u8> },
    MarkPdfLoadingFailed { uuid: String },
}

pub struct Bitmap {
    pub width: i32,
    pub height: i32,
    pub uid: String,
    pub pixels: Vec<u32>,
}

impl Bitmap {
    pub fn new(
        width: i32,
        height: i32,
        uid: String,
        pixels: Vec<u32>,
    ) -> Arc<Bitmap> {
        Arc::new(
            Bitmap {
                width,
                height,
                uid,
                pixels,
            }
        )
    }

    pub fn width(&self) -> i32 {
        self.width
    }

    pub fn height(&self) -> i32 {
        self.height
    }

    pub fn uid(&self) -> String {
        self.uid.clone()
    }

    pub fn copy_pixels(&self) -> Vec<u32> {
        self.pixels.clone()
    }
}

impl PartialEq for Bitmap {
    fn eq(&self, other: &Self) -> bool {
        self.uid == other.uid
    }
}

impl Clone for Bitmap {
    fn clone(&self) -> Self {
        info!("ERROR - cloned bitmap");
        Bitmap {
            width: self.width,
            height: self.height,
            uid: self.uid.clone(),
            pixels: vec![],
        }
    }
}

pub enum GlobalResult {
    PdfLoading { uuid: String },
    PdfLoadingFailed { uuid: String },
    PdfLoaded {
        uuid: String,
        title: String,
        author: String,
        thumbnail: Option<Arc<Bitmap>>,
    },
}

pub trait GlobalStateListener: Send + Sync {
    fn new_state(&self, state: GlobalState);
}

pub trait GlobalDispatch {
    fn dispatch_action(self, action: GlobalResult);
}

pub struct GlobalStore {
    state: Mutex<GlobalState>,
    listeners: Mutex<HashMap<String, Box<dyn GlobalStateListener>>>,
    pdfium_manager: Mutex<Option<PdfiumManager>>,
    worker_thread_manager: Mutex<Option<WorkerThreadManager>>,
}

impl GlobalStore {
    pub fn new() -> Self {
        let initial_state = GlobalState { some_text: "initial_text".to_string(), books: Vec::new() };
        android_logger::init_once(Config::default().with_max_level(LevelFilter::Trace));
        Self {
            state: Mutex::new(initial_state),
            listeners: Mutex::new(HashMap::new()),
            pdfium_manager: Mutex::new(None),
            worker_thread_manager: Mutex::new(None),
        }
    }

    pub fn init(self: Arc<Self>) {
        let worker_thread_manager = Self::init_worker_thread(self.clone());
        let pdfium_manager = PdfiumManager::new(worker_thread_manager.global_action_sender.clone());
        let mut pdfium_manager_reference = self.pdfium_manager.lock().unwrap();
        *pdfium_manager_reference = Some(pdfium_manager);
        let mut worker_thread_manager_reference = self.worker_thread_manager.lock().unwrap();
        *worker_thread_manager_reference = Some(worker_thread_manager)
    }

    pub fn add_listener(&self, id: String, state_listener: Box<dyn GlobalStateListener>) {
        state_listener.new_state(self.state.lock().unwrap().clone());
        self.listeners.lock().unwrap().insert(id, state_listener);
    }

    pub fn remove_listener(&self, id: String) {
        self.listeners.lock().unwrap().remove(&id);
    }

    pub fn dispatch_action(self: Arc<Self>, action: GlobalAction) {
        match action {
            GlobalAction::MarkPdfLoading { uuid } => self.process_result(GlobalResult::PdfLoading { uuid }),
            GlobalAction::LoadPdf { uuid, file_name, bytes } => match self.load_pdf(uuid, file_name, bytes) {
                Ok(_) => {}
                Err(error) => { error!("dispatch_thunk error - {error}") }
            }
            GlobalAction::MarkPdfLoadingFailed { uuid } => self.process_result(GlobalResult::PdfLoadingFailed { uuid }),
        };
    }

    pub fn process_result(self: Arc<Self>, action: GlobalResult) {
        let mut state = self.state.lock().unwrap();
        let new_state = Self::reduce(state.clone(), action);
        *state = new_state;
        for listener in self.listeners.lock().unwrap().values() {
            listener.new_state(state.clone())
        }
    }

    fn reduce(state: GlobalState, action: GlobalResult) -> GlobalState {
        match action {
            GlobalResult::PdfLoading { uuid } => {
                let mut new_state = state.clone();
                new_state.books.push(
                    Book {
                        uuid,
                        thumbnail: None,
                        loading_state: PdfLoadingState::LoadingPdf,
                    }
                );
                new_state
            }
            GlobalResult::PdfLoadingFailed { uuid } => {
                let mut new_state = state.clone();
                for book in &mut new_state.books {
                    if uuid == book.uuid {
                        book.loading_state = PdfLoadingState::ErrorPdf
                    }
                }
                new_state
            }
            GlobalResult::PdfLoaded { title, author, uuid, thumbnail } => {
                let mut new_state = state.clone();
                for book in &mut new_state.books {
                    if uuid == book.uuid {
                        book.thumbnail = thumbnail.clone();
                        book.loading_state = PdfLoadingState::ValidPdf {
                            title: title.clone(),
                            author: author.clone(),
                            thumbnail: thumbnail.clone(),
                        }
                    }
                }
                new_state
            }
        }
    }


    // This really shouldn't be here. I should find a way to do this on a main thread for each platform
    fn init_worker_thread(store: Arc<GlobalStore>) -> WorkerThreadManager {
        let (action_sender, action_receiver): (Sender<GlobalResult>, Receiver<GlobalResult>) = channel();
        let handle = thread::spawn(move || {
            loop {
                let action = action_receiver.recv().unwrap();
                store.clone().process_result(action);
            }
        });
        WorkerThreadManager {
            global_action_sender: Arc::new(Mutex::new(action_sender)),
            worker_thread_handle: handle,
        }
    }

    fn load_pdf(&self, uuid: String, file_name: String, bytes: Vec<u8>) -> Result<()> {
        let guard = self.pdfium_manager.lock().unwrap();
        let pdfium_manager = guard.as_ref().context("No Pdfium Manager")?;
        let pdfium_action_sender = pdfium_manager.pdfium_action_sender.lock().unwrap();
        pdfium_action_sender.send(PdfiumAction::LoadPdf { uuid, file_name, bytes })?;
        Ok(())
    }
}

struct WorkerThreadManager {
    global_action_sender: Arc<Mutex<Sender<GlobalResult>>>,
    worker_thread_handle: JoinHandle<()>,
}
