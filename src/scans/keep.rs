//! Keeping every scanned photo for the dataset, as it arrived.
//!
//! Proof of concept: nothing asks the cook yet. Before real cooks use this,
//! a consent step must come first (see AGENTS.md). A failure to keep never
//! fails the scan: the cook still gets their answer, and the photo is simply
//! not kept.

use super::db as fridge_photos;
use crate::scans::fridge::SeenItem;
use crate::scans::photo::Photo;
use crate::storage::StoredFile;
use crate::AppState;
use uuid::Uuid;

pub struct Scan<'a> {
    pub user_id: Uuid,
    pub scan_id: Uuid,
    pub photo: &'a Photo,
    pub items: &'a [SeenItem],
}

impl Scan<'_> {
    /// One folder per cook, so all their photos can be found and deleted.
    fn path(&self) -> String {
        format!("{}/{}.{}", self.user_id, self.scan_id, self.photo.kind.extension())
    }
}

/// Whether the photo was kept. Without photo storage it is not, and that is no failure.
pub async fn photo(state: &AppState, scan: &Scan<'_>) -> bool {
    if !state.photos.is_configured() {
        return false;
    }
    match keep(state, scan).await {
        Ok(()) => true,
        Err(reason) => {
            log_not_kept(reason);
            false
        }
    }
}

async fn keep(state: &AppState, scan: &Scan<'_>) -> Result<(), &'static str> {
    let path = scan.path();
    store(state, scan, &path).await?;
    let recorded = record(state, scan, &path).await;
    if recorded.is_err() {
        // Without its record the file could not be found again to delete.
        let _ = state.photos.delete(&[path]).await;
    }
    recorded
}

async fn store(state: &AppState, scan: &Scan<'_>, path: &str) -> Result<(), &'static str> {
    let file = StoredFile { path, bytes: scan.photo.bytes.clone(), mime_type: scan.photo.kind.mime_type() };
    state.photos.put(file).await.map_err(|_| "the photo storage refused it")
}

async fn record(state: &AppState, scan: &Scan<'_>, path: &str) -> Result<(), &'static str> {
    let photo = fridge_photos::KeptPhoto {
        id: scan.scan_id,
        user_id: scan.user_id,
        object_path: path,
        mime_type: scan.photo.kind.mime_type(),
        model: state.ai.model_name(),
        seen: scan.items,
    };
    let inserted = fridge_photos::insert(&state.db, &photo).await;
    inserted.map_err(|_| "the record could not be written")
}

#[expect(clippy::cognitive_complexity, reason = "tracing's macro expands into branches no reader sees")]
fn log_not_kept(reason: &'static str) {
    tracing::warn!(reason, "a photo was not kept");
}
