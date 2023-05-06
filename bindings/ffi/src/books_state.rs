use std::collections::HashMap;
use std::ops::Deref;
use std::string::ToString;
use std::sync::{Arc, Mutex, RwLock};
use pdfium_render::prelude::Pdfium;
use crate::books_state::BooksAction::PdfsListUpdated;
use crate::global_state::{GlobalAction, GlobalState, GlobalStateListener, GlobalStore, GlobalThunk, Pdf};

#[derive(Clone)]
pub struct BooksState {
    pub some_text: String,
    pub pdfs: Vec<Pdf>,
}

#[derive(Clone)]
pub enum BooksSideEffect {
    OpenFilePicker,
}

pub enum BooksThunk {
    AddClicked,
}

pub enum BooksAction {
    PdfsListUpdated { pdfs: Vec<Pdf> },
}

pub trait BooksStateListener: Send + Sync {
    fn new_state(&self, state: BooksState);
    fn new_side_effect(&self, event: BooksSideEffect);
}

const BOOKS_GLOBAL_STORE_LISTENER_ID: &str = "BOOKS_GLOBAL_STORE_LISTENER_ID";

pub struct BooksStore {
    global_store: Mutex<Arc<GlobalStore>>,
    state: Mutex<BooksState>,
    listeners: Mutex<HashMap<String, Box<dyn BooksStateListener>>>,
    // cache
    last_global_state: Option<GlobalState>,
}

impl BooksStore {
    pub fn new(global_store: Arc<GlobalStore>) -> Self {
        let initial_state = BooksState { some_text: "initial_text".to_string(), pdfs: Vec::new() };
        Self {
            global_store: Mutex::new(global_store),
            state: Mutex::new(initial_state),
            listeners: Mutex::new(HashMap::new()),
            last_global_state: None,
        }
    }

    pub fn init(self: Arc<Self>) {
        self.global_store.lock().unwrap().add_listener(BOOKS_GLOBAL_STORE_LISTENER_ID.to_string(), Box::new(self.clone()));
    }

    pub fn add_listener(&self, id: String, state_listener: Box<dyn BooksStateListener>) {
        state_listener.new_state(self.state.lock().unwrap().clone());
        self.listeners.lock().unwrap().insert(id, state_listener);
    }

    pub fn remove_listener(&self, id: String) {
        self.listeners.lock().unwrap().remove(&id);
    }

    pub fn dispatch_thunk(self: Arc<Self>, thunk: BooksThunk) {
        match thunk {
            BooksThunk::AddClicked =>  self.dispatch_side_effect(BooksSideEffect::OpenFilePicker)
        }
    }

    pub fn dispatch_action(self: Arc<Self>, action: BooksAction) {
        let mut state = self.state.lock().unwrap();
        let new_state = Self::reduce(state.clone(), action);
        *state = new_state;
        for listener in self.listeners.lock().unwrap().values() {
            listener.new_state(state.clone())
        }
    }

    fn reduce(state: BooksState, action: BooksAction) -> BooksState {
        match action {
            PdfsListUpdated { pdfs } => {
                let mut new_state = state.clone();
                new_state.pdfs = pdfs;
                new_state
            }
        }
    }

    fn dispatch_side_effect(&self, side_effect:  BooksSideEffect) {
        for listener in self.listeners.lock().unwrap().values() {
            listener.new_side_effect(side_effect.clone());
        }
    }
}

impl Drop for BooksStore {
    fn drop(&mut self) {
        self.global_store.lock().unwrap().remove_listener(BOOKS_GLOBAL_STORE_LISTENER_ID.to_string());
    }
}

impl GlobalStateListener for Arc<BooksStore> {
    fn new_state(&self, state: GlobalState) {
        let Some(last_global_state) = &self.last_global_state  else {
            self.clone().dispatch_action(PdfsListUpdated { pdfs: state.pdfs.clone() });
            return;
        };
        if last_global_state.pdfs != state.pdfs {
            self.clone().dispatch_action(PdfsListUpdated { pdfs: state.pdfs.clone() });
        }
    }
}
