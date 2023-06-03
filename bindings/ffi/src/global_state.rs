use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{channel, Receiver, Sender};
use pdfium_render::prelude::*;

#[cfg(target_os = "android")]
use android_logger::Config;
#[cfg(target_os = "android")]
use log::LevelFilter;

use std::thread;
use std::thread::JoinHandle;
use anyhow::{Context, Result};
use crate::domain::{Bitmap, Book, Page, PdfLoadingState};
use crate::pdfium_manager::{PdfiumAction, PdfiumManager};


#[derive(Clone)]
pub struct GlobalState {
    pub some_text: String,
    pub books: Vec<Book>,
    pub current_book: Option<Book>,
    pub current_book_pages: Vec<Arc<Page>>,
}

pub enum GlobalAction {
    MarkPdfLoading { uuid: String },
    LoadPdf { uuid: String, file_name: String, bytes: Vec<u8> },
    MarkPdfLoadingFailed { uuid: String },
    LoadPage { page_index: i32 },
}

pub enum GlobalResult {
    PdfLoading { uuid: String },
    PdfLoadingFailed { uuid: String },
    PdfLoaded {
        id: String,
        title: String,
        author: String,
        thumbnail: Option<Arc<Bitmap>>,
        page_count: i32,
    },
    PagesLoaded {
        pages: Vec<Arc<Page>>,
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
        let initial_state = GlobalState {
            some_text: "initial_text".to_string(),
            books: Vec::new(),
            current_book: None,
            current_book_pages: vec![],
        };
        #[cfg(target_os = "android")]
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
                Err(error) => { error!("GlobalAction::LoadPdf error - {error}") }
            }
            GlobalAction::MarkPdfLoadingFailed { uuid } => self.process_result(GlobalResult::PdfLoadingFailed { uuid }),
            GlobalAction::LoadPage { page_index } => match self.load_page(page_index) {
                Ok(_) => {}
                Err(error) => { error!("GlobalAction::LoadPage error - {error}") }
            }
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
            GlobalResult::PdfLoaded { id, title, author, thumbnail, page_count } => {
                let mut new_state = state.clone();
                for book in &mut new_state.books {
                    if id == book.uuid {
                        book.thumbnail = thumbnail.clone();
                        book.loading_state = PdfLoadingState::ValidPdf {
                            title: title.clone(),
                            author: author.clone(),
                            thumbnail: thumbnail.clone(),
                            page_count,
                        };
                        new_state.current_book = Some(book.clone());
                        new_state.current_book_pages = (0..page_count)
                            .map(|index| { Arc::new(Page { index, image: None }) })
                            .collect();
                    }
                }
                new_state
            }
            GlobalResult::PagesLoaded { pages } => {
                let mut new_state = state.clone();
                for page in pages {
                    let index = page.index as usize;
                    new_state.current_book_pages[index] = page
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

    fn load_page(&self, page_index: i32) -> Result<()> {
        self.pdfium_manager
            .lock()
            .unwrap()
            .as_ref()
            .unwrap()
            .pdfium_action_sender
            .lock()
            .unwrap()
            .send(PdfiumAction::PageLoadRequested { page_index })
            .unwrap();
        Ok(())
    }
}

struct WorkerThreadManager {
    global_action_sender: Arc<Mutex<Sender<GlobalResult>>>,
    worker_thread_handle: JoinHandle<()>,
}
