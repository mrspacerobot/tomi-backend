//! A fridge photo as it arrives, and the copy of it that may be stored.
//!
//! Only JPEG and PNG are taken: the app converts other formats before
//! sending. A stored copy never carries the photo's metadata, which can
//! hold where and when it was taken and on which phone.

use serde::Serialize;

mod jpeg;
mod png;
#[cfg(test)]
mod tests;

/// Larger than any phone photo the app sends after resizing.
pub const MAX_PHOTO_BYTES: usize = 10 * 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ImageKind {
    Jpeg,
    Png,
}

impl ImageKind {
    pub fn from_content_type(content_type: &str) -> Option<Self> {
        match content_type.trim() {
            "image/jpeg" => Some(ImageKind::Jpeg),
            "image/png" => Some(ImageKind::Png),
            _ => None,
        }
    }

    pub fn mime_type(self) -> &'static str {
        match self {
            ImageKind::Jpeg => "image/jpeg",
            ImageKind::Png => "image/png",
        }
    }

    pub fn extension(self) -> &'static str {
        match self {
            ImageKind::Jpeg => "jpg",
            ImageKind::Png => "png",
        }
    }
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum PhotoError {
    #[error("send the photo as image/jpeg or image/png")]
    UnsupportedType,
    #[error("the photo is empty")]
    Empty,
    #[error("the photo is larger than {MAX_PHOTO_BYTES} bytes")]
    TooLarge,
    #[error("the photo is not the type its header says")]
    WrongType,
    #[error("the photo could not be read")]
    Unreadable,
}

pub struct Photo {
    pub kind: ImageKind,
    pub bytes: Vec<u8>,
}

impl Photo {
    pub fn parse(content_type: Option<&str>, bytes: Vec<u8>) -> Result<Self, PhotoError> {
        let kind = content_type.and_then(ImageKind::from_content_type);
        let Some(kind) = kind else {
            return Err(PhotoError::UnsupportedType);
        };
        check_size(&bytes)?;
        if !starts_like(kind, &bytes) {
            return Err(PhotoError::WrongType);
        }
        Ok(Photo { kind, bytes })
    }

    /// The same picture without its metadata, for storing.
    pub fn without_metadata(&self) -> Result<Vec<u8>, PhotoError> {
        let stripped = match self.kind {
            ImageKind::Jpeg => jpeg::without_metadata(&self.bytes),
            ImageKind::Png => png::without_metadata(&self.bytes),
        };
        stripped.ok_or(PhotoError::Unreadable)
    }
}

fn check_size(bytes: &[u8]) -> Result<(), PhotoError> {
    if bytes.is_empty() {
        return Err(PhotoError::Empty);
    }
    if bytes.len() > MAX_PHOTO_BYTES {
        return Err(PhotoError::TooLarge);
    }
    Ok(())
}

fn starts_like(kind: ImageKind, bytes: &[u8]) -> bool {
    match kind {
        ImageKind::Jpeg => bytes.starts_with(&jpeg::START),
        ImageKind::Png => bytes.starts_with(&png::SIGNATURE),
    }
}
