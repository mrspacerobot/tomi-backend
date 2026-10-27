//! `POST /v1/cook-entries`: the cooks in the phone's outbox in, what became of each out.

use super::db::{self as store, EntryWrite};
use super::{id_of, CheckedRequest, EntryChecks, EntryRequest, EntryResult, EntryStatus, ReceivedEntry};
use crate::error::AppError;
use crate::http::{json_body, Caller, CookLanguage};
use crate::progress::{snapshot, ProgressSnapshot, SnapshotRequest};
use crate::AppState;
use axum::extract::rejection::JsonRejection;
use axum::routing::post;
use axum::{Json, Router};
use serde::Serialize;
use serde_json::Value;
use uuid::Uuid;

pub fn router() -> Router<AppState> {
    Router::new().route("/v1/cook-entries", post(receive))
}

#[derive(Serialize)]
struct EntryResponse {
    result: EntryResult,
    progress: ProgressSnapshot,
}

/// The cook, the server and the moment the entry arrived.
struct Receipt<'a> {
    state: &'a AppState,
    user_id: Uuid,
    checks: EntryChecks<'a>,
}

async fn receive(
    caller: Caller,
    CookLanguage(language): CookLanguage,
    body: Result<Json<EntryRequest>, JsonRejection>,
) -> Result<Json<EntryResponse>, AppError> {
    let Caller { state, user_id } = caller;
    let request = CheckedRequest::parse(json_body(body)?).map_err(AppError::Invalid)?;
    let checks = EntryChecks { received_at: state.clock.now(), graph: state.graph.as_deref() };
    let receipt = Receipt { state: &state, user_id, checks };
    let result = answer(&receipt, &request.entry).await?;
    let inputs = SnapshotRequest { calendar: &request.calendar, language: &language, now: receipt.checks.received_at };
    let progress = snapshot(&state, user_id, &inputs).await?;
    Ok(Json(EntryResponse { result, progress }))
}

async fn answer(receipt: &Receipt<'_>, raw: &Value) -> Result<EntryResult, AppError> {
    match ReceivedEntry::read(raw, &receipt.checks) {
        Ok(entry) => {
            let status = keep(receipt, &entry).await?;
            Ok(EntryResult { id: Some(entry.id), status })
        }
        Err(problems) => Ok(EntryResult { id: id_of(raw), status: EntryStatus::Rejected { problems } }),
    }
}

/// Stores the entry, or tells a retry from a different body when the cook already has its id.
async fn keep(receipt: &Receipt<'_>, entry: &ReceivedEntry) -> Result<EntryStatus, AppError> {
    let payload_sha256 = entry.payload_sha256()?;
    let (user_id, received_at) = (receipt.user_id, receipt.checks.received_at);
    let write = EntryWrite { user_id, entry, payload_sha256: &payload_sha256, received_at };
    if store::insert(&receipt.state.db, &write).await? {
        return Ok(EntryStatus::Accepted);
    }
    let stored = store::stored_sha256(&receipt.state.db, user_id, entry.id).await?;
    let status = EntryStatus::of_copy(&stored, &payload_sha256);
    if status == EntryStatus::Conflict {
        log_conflict(entry.id);
    }
    Ok(status)
}

/// Only the entry id and the code: never what the entry says.
#[expect(clippy::cognitive_complexity, reason = "tracing's macro expands into branches no reader sees")]
fn log_conflict(entry_id: Uuid) {
    tracing::warn!(%entry_id, code = "conflict", "a cook entry arrived with a different body; the first copy stays");
}
