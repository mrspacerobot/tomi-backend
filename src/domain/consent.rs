//! What the cook agreed to, each purpose on its own.
//!
//! Scanning a fridge and keeping the photo for Tomi's dataset are separate
//! choices. A consent counts only for the text version the app showed; a
//! new text needs the cook to agree again.

use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Purpose {
    /// The photo is sent to the model to read, then dropped.
    PhotoScan,
    /// The photo, without metadata, is kept with what was seen in it.
    PhotoDataset,
}

impl Purpose {
    pub const ALL: [Purpose; 2] = [Purpose::PhotoScan, Purpose::PhotoDataset];

    pub fn parse(slug: &str) -> Option<Self> {
        Purpose::ALL.into_iter().find(|purpose| purpose.slug() == slug)
    }

    /// The name in URLs and in the database. Permanent once released.
    pub fn slug(self) -> &'static str {
        match self {
            Purpose::PhotoScan => "photo-scan",
            Purpose::PhotoDataset => "photo-dataset",
        }
    }

    /// The version of the text the app must show. Raise it when the text
    /// changes in substance, so every cook is asked again.
    pub fn current_version(self) -> i32 {
        match self {
            Purpose::PhotoScan => 1,
            Purpose::PhotoDataset => 1,
        }
    }
}
