use std::sync::Arc;

#[derive(Clone, PartialEq)]
pub struct Book {
    pub uuid: String,
    pub thumbnail: Option<Arc<Bitmap>>,
    pub loading_state: PdfLoadingState,
}

#[derive(Clone, PartialEq)]
pub struct Page {
    pub index: i32,
    pub image: Option<Arc<Bitmap>>,
}

impl Page {
    pub fn index(&self) -> i32 {
        self.index
    }

    pub fn image(&self) -> Option<Arc<Bitmap>> {
        match &self.image {
            None => None,
            Some(image) => Some(Arc::clone(image)),
        }
    }
}

#[derive(Clone, PartialEq)]
pub enum PdfLoadingState {
    LoadingPdf,
    ValidPdf {
        title: String,
        author: String,
        thumbnail: Option<Arc<Bitmap>>,
        page_count: i32,
    },
    ErrorPdf,
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
        error!("ERROR - cloned bitmap. Ok, maybe there are some valid use cases but I don't have one");
        Bitmap {
            width: self.width,
            height: self.height,
            uid: self.uid.clone(),
            pixels: vec![],
        }
    }
}
