//! A fridge photo as it arrives.
//!
//! Only JPEG and PNG are taken: the app converts other formats before
//! sending. A kept photo is stored as it arrived, metadata included: what
//! the app sends is the app's decision.

use serde::Serialize;

#[cfg(test)]
mod tests;

/// The first bytes of a JPEG.
const JPEG_START: [u8; 2] = [0xFF, 0xD8];
/// The first bytes of a PNG.
const PNG_SIGNATURE: [u8; 8] = [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];

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
        ImageKind::Jpeg => bytes.starts_with(&JPEG_START),
        ImageKind::Png => bytes.starts_with(&PNG_SIGNATURE),
    }
}
