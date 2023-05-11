uniffi_macros::include_scaffolding!("global_bindings");

use crate::books_state::{BooksState, BooksAction, BooksThunk, BooksSideEffect, BooksStateListener, BooksStore};
use crate::global_state::{GlobalState, Pdf, BookCover, GlobalAction, GlobalThunk, GlobalStateListener, GlobalStore};
use crate::pdfium_manager::{generate_pdf_uuid};
