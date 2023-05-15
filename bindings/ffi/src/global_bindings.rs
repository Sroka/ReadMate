uniffi_macros::include_scaffolding!("global_bindings");

use crate::books_state::{BooksState, BooksResult, BooksAction, BooksSideEffect, BooksStateListener, BooksStore};
use crate::global_state::{GlobalState, PdfLoadingState, Book, Bitmap, GlobalResult, GlobalAction, GlobalStateListener, GlobalStore};
use crate::pdfium_manager::{generate_pdf_uuid};
