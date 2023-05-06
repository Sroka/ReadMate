use std::any::Any;
use std::collections::HashMap;
use std::sync::{Arc, LockResult, Mutex, RwLock};
use std::sync::mpsc::{channel, Receiver, Sender, SendError};
use pdfium_render::prelude::*;
use uniffi::deps::log::info;
use log::LevelFilter;
use android_logger::Config;

use std::thread;
use std::thread::JoinHandle;
use anyhow::{Result, Context};
use GlobalAction::{PdfLoaded, PdfLoading, PdfLoadingFailed};
use GlobalThunk::LoadPdf;
use crate::pdfium_manager::{PdfiumManager, PdfiumAction};


#[derive(Clone)]
pub struct GlobalState {
    pub some_text: String,
    pub pdfs: Vec<Pdf>,
}

#[derive(Clone, PartialEq)]
pub struct Pdf {
    pub uuid: String,
}

pub enum GlobalThunk {
    LoadPdf { bytes: Vec<u8> },
}

pub enum GlobalAction {
    PdfLoading { uuid: String },
    PdfLoadingFailed { uuid: String },
    PdfLoaded { uuid: String },
}

pub trait GlobalStateListener: Send + Sync {
    fn new_state(&self, state: GlobalState);
}

pub trait GlobalDispatch {
    fn dispatch_action(self, action: GlobalAction);
}

pub struct GlobalStore {
    state: Mutex<GlobalState>,
    listeners: Mutex<HashMap<String, Box<dyn GlobalStateListener>>>,
    pdfium_manager: Mutex<Option<PdfiumManager>>,
    worker_thread_manager: Mutex<Option<WorkerThreadManager>>,
}

impl GlobalStore {
    pub fn new() -> Self {
        let initial_state = GlobalState { some_text: "initial_text".to_string(), pdfs: Vec::new() };
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

    pub fn dispatch_thunk(self: Arc<Self>, thunk: GlobalThunk) {
        let result = match thunk {
            LoadPdf { bytes } => self.load_pdf(bytes)
        };
        match result {
            Ok(_) => {}
            Err(error) => {error!("dispatch_thunk error - {error}")}
        }
    }

    pub fn dispatch_action(self: Arc<Self>, action: GlobalAction) {
        let mut state = self.state.lock().unwrap();
        let new_state = Self::reduce(state.clone(), action);
        *state = new_state;
        for listener in self.listeners.lock().unwrap().values() {
            listener.new_state(state.clone())
        }
    }

    fn reduce(state: GlobalState, action: GlobalAction) -> GlobalState {
        match action {
            PdfLoading { uuid } => {
                let mut new_state = state.clone();
                new_state.pdfs.push(Pdf { uuid });
                new_state
            }
            PdfLoadingFailed { uuid } => { state }
            PdfLoaded { uuid } => { state }
        }
    }


    // This really shouldn't be here. I should find a way to do this on a main thread for each platform
    fn init_worker_thread(store: Arc<GlobalStore>) -> WorkerThreadManager {
        let (action_sender, action_receiver): (Sender<GlobalAction>, Receiver<GlobalAction>) = channel();
        let handle = thread::spawn(move || {
            loop {
                let action = action_receiver.recv().unwrap();
                store.clone().dispatch_action(action);
            }
        });
        WorkerThreadManager {
            global_action_sender: Arc::new(Mutex::new(action_sender)),
            worker_thread_handle: handle,
        }
    }

    fn load_pdf(&self, bytes: Vec<u8>) -> Result<()> {
        let guard = self.pdfium_manager.lock().unwrap();
        let pdfium_manager = guard.as_ref().context("No Pdfium Manager")?;
        let pdfium_action_sender = pdfium_manager.pdfium_action_sender.lock().unwrap();
        pdfium_action_sender.send(PdfiumAction::LoadPdf { bytes })?;
        Ok(())
    }
}

struct WorkerThreadManager {
    global_action_sender: Arc<Mutex<Sender<GlobalAction>>>,
    worker_thread_handle: JoinHandle<()>,
}
