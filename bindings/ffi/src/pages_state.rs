use std::collections::HashMap;
use std::string::ToString;
use std::sync::{Arc, Mutex};
use crate::domain::{Book, Page, PdfLoadingState};
use crate::global_state::{GlobalAction, GlobalState, GlobalStateListener, GlobalStore};

#[derive(Clone)]
pub struct PagesState {
    pub current_book: Option<Book>,
    pub current_book_pages: Vec<Arc<Page>>,
}

pub enum PagesAction {
    CurrentPage { page_index: i32 },
}

pub enum PagesResult {
    PagesListUpdated { pages: Vec<Arc<Page>> },
}

pub trait PagesStateListener: Send + Sync {
    fn new_state(&self, state: PagesState);
}

const PAGES_GLOBAL_STORE_LISTENER_ID: &str = "PAGES_GLOBAL_STORE_LISTENER_ID";

pub struct PagesStore {
    global_store: Mutex<Arc<GlobalStore>>,
    state: Mutex<PagesState>,
    listeners: Mutex<HashMap<String, Box<dyn PagesStateListener>>>,
    // cache
    last_global_state: Option<GlobalState>,
}

impl PagesStore {
    pub fn new(global_store: Arc<GlobalStore>) -> Self {
        let initial_state = PagesState { current_book: None, current_book_pages: vec![] };
        Self {
            global_store: Mutex::new(global_store),
            state: Mutex::new(initial_state),
            listeners: Mutex::new(HashMap::new()),
            last_global_state: None,
        }
    }

    pub fn init(self: Arc<Self>) {
        self.global_store.lock().unwrap().add_listener(PAGES_GLOBAL_STORE_LISTENER_ID.to_string(), Box::new(self.clone()));
    }

    pub fn add_listener(&self, id: String, state_listener: Box<dyn PagesStateListener>) {
        state_listener.new_state(self.state.lock().unwrap().clone());
        self.listeners.lock().unwrap().insert(id, state_listener);
    }

    pub fn remove_listener(&self, id: String) {
        self.listeners.lock().unwrap().remove(&id);
    }

    pub fn dispatch_action(self: Arc<Self>, action: PagesAction) {
        match action {
            PagesAction::CurrentPage { page_index } => {
                // let pages = self.state.lock().unwrap().pages;
                // pages

            }
        }
    }

    pub fn process_result(self: Arc<Self>, result: PagesResult) {
        let mut state = self.state.lock().unwrap();
        let new_state = Self::reduce(state.clone(), result);
        *state = new_state;
        for listener in self.listeners.lock().unwrap().values() {
            listener.new_state(state.clone())
        }
    }

    fn reduce(state: PagesState, action: PagesResult) -> PagesState {
        match action {
            PagesResult::PagesListUpdated { pages } => {
                let mut new_state = state.clone();
                new_state.current_book_pages = pages;
                new_state
            }
        }
    }
}

impl Drop for PagesStore {
    fn drop(&mut self) {
        self.global_store.lock().unwrap().remove_listener(PAGES_GLOBAL_STORE_LISTENER_ID.to_string());
    }
}

impl GlobalStateListener for Arc<PagesStore> {
    fn new_state(&self, new_global_state: GlobalState) {
        let Some(last_global_state) = &self.last_global_state  else {
            self.clone().process_result(PagesResult::PagesListUpdated { pages: new_global_state.current_book_pages.clone() });
            return;
        };
        if last_global_state.current_book_pages != new_global_state.current_book_pages {
            self.clone().process_result(PagesResult::PagesListUpdated { pages: new_global_state.current_book_pages.clone() });
        }
    }
}
