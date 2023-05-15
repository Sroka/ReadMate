use std::collections::HashMap;
use std::string::ToString;
use std::sync::{Arc, Mutex};
use crate::books_state::BooksResult::BooksListUpdated;
use crate::global_state::{Book, GlobalState, GlobalStateListener, GlobalStore, PdfLoadingState};

#[derive(Clone)]
pub struct BooksState {
    pub some_text: String,
    pub books: Vec<Book>,
}

#[derive(Clone)]
pub enum BooksSideEffect {
    OpenFilePicker,
}

pub enum BooksAction {
    AddClicked,
}

pub enum BooksResult {
    BooksListUpdated { books: Vec<Book> },
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
        let initial_state = BooksState { some_text: "initial_text".to_string(), books: Vec::new() };
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

    pub fn dispatch_action(self: Arc<Self>, action: BooksAction) {
        match action {
            BooksAction::AddClicked =>  self.dispatch_side_effect(BooksSideEffect::OpenFilePicker)
        }
    }

    pub fn process_result(self: Arc<Self>, result: BooksResult) {
        let mut state = self.state.lock().unwrap();
        let new_state = Self::reduce(state.clone(), result);
        *state = new_state;
        for listener in self.listeners.lock().unwrap().values() {
            listener.new_state(state.clone())
        }
    }

    fn reduce(state: BooksState, action: BooksResult) -> BooksState {
        match action {
            BooksListUpdated { books: pdfs } => {
                let mut new_state = state.clone();
                new_state.books = pdfs;
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
            self.clone().process_result(BooksListUpdated { books: state.books.clone() });
            return;
        };
        if last_global_state.books != state.books {
            self.clone().process_result(BooksListUpdated { books: state.books.clone() });
        }
    }
}
