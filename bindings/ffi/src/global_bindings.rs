uniffi_macros::include_scaffolding!("global_bindings");

use crate::books_state::{BooksAction, BooksResult, BooksSideEffect, BooksState, BooksStateListener, BooksStore};
use crate::domain::{Bitmap, Book, Page, PdfLoadingState};
use crate::global_state::{GlobalAction, GlobalResult, GlobalState, GlobalStateListener, GlobalStore};
use crate::pages_state::{PagesAction, PagesState, PagesStateListener, PagesStore};
use crate::pdfium_manager::generate_pdf_uuid;
