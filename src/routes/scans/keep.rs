//! Keeping a scanned photo for the dataset, only with the cook's consent.
//!
//! A failure to keep never fails the scan: the cook still gets their
//! answer, and the photo is simply not kept.

use crate::db::{consents, fridge_photos};
use crate::domain::consent::Purpose;
use crate::domain::fridge::SeenItem;
use crate::domain::photo::Photo;
use crate::error::AppError;
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

/// Whether the photo was kept.
pub async fn if_agreed(state: &AppState, scan: &Scan<'_>) -> Result<bool, AppError> {
    if !consents::is_current(&state.db, scan.user_id, Purpose::PhotoDataset).await? {
        return Ok(false);
    }
    match keep(state, scan).await {
        Ok(()) => Ok(true),
        Err(reason) => {
            log_not_kept(reason);
            Ok(false)
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
    let Ok(bytes) = scan.photo.without_metadata() else {
        return Err("the photo could not be read for storing");
    };
    let file = StoredFile { path, bytes, mime_type: scan.photo.kind.mime_type() };
    state.photos.put(file).await.map_err(|_| "the photo storage refused it")
}

async fn record(state: &AppState, scan: &Scan<'_>, path: &str) -> Result<(), &'static str> {
    let photo = fridge_photos::KeptPhoto {
        id: scan.scan_id,
        user_id: scan.user_id,
        object_path: path,
        mime_type: scan.photo.kind.mime_type(),
        consent_version: Purpose::PhotoDataset.current_version(),
        model: state.ai.model_name(),
        seen: scan.items,
    };
    let inserted = fridge_photos::insert(&state.db, &photo).await;
    inserted.map_err(|_| "the record could not be written")
}

#[expect(clippy::cognitive_complexity, reason = "tracing's macro expands into branches no reader sees")]
fn log_not_kept(reason: &'static str) {
    tracing::warn!(reason, "a consented photo was not kept");
}
