//! `POST /v1/cook-entries`: the cooks in the phone's outbox in, what became of each out, and the progress after them.
//! `GET /v1/progress`: the progress snapshot, computed from the stored cook log on every request.

use super::db::{self as store, EntryWrite};
use super::received::{id_of, BatchFile, EntryBatch, EntryChecks, EntryResult, EntryStatus, ReceivedEntry};
use super::{CookCalendar, ParseCalendarError, ProgressAnswers, ProgressSnapshot, SnapshotInputs};
use crate::error::AppError;
use crate::http::{json_body, Caller, CookLanguage};
use crate::kernel::language::Language;
use crate::profile::read as read_profile;
use crate::AppState;
use axum::extract::rejection::{JsonRejection, QueryRejection};
use axum::extract::Query;
use axum::routing::{get, post};
use axum::{Json, Router};
use jiff::Timestamp;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

pub fn router() -> Router<AppState> {
    Router::new().route("/v1/cook-entries", post(receive)).route("/v1/progress", get(progress))
}

#[derive(Serialize)]
struct EntriesResponse {
    /// One answer per entry, in the order the entries were sent.
    results: Vec<EntryResult>,
    /// The snapshot after the batch, the same `GET /v1/progress` gives.
    progress: ProgressSnapshot,
}

/// The cook's calendar, as `GET /v1/progress` takes it. Both are required: there is no default zone.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct CalendarQuery {
    time_zone: String,
    first_weekday: i64,
}

/// What a snapshot is asked for with, besides the cook: the calendar, the language and the moment of the request.
struct SnapshotRequest<'a> {
    calendar: &'a CookCalendar,
    language: &'a Language,
    now: Timestamp,
}

/// The cook, the server and the moment the batch arrived, shared by every entry of it.
struct Receipt<'a> {
    state: &'a AppState,
    user_id: Uuid,
    checks: EntryChecks<'a>,
}

async fn progress(
    caller: Caller,
    CookLanguage(language): CookLanguage,
    query: Result<Query<CalendarQuery>, QueryRejection>,
) -> Result<Json<ProgressSnapshot>, AppError> {
    let Caller { state, user_id } = caller;
    let calendar = calendar_of(query)?;
    let request = SnapshotRequest { calendar: &calendar, language: &language, now: state.clock.now() };
    Ok(Json(snapshot(&state, user_id, &request).await?))
}

fn calendar_of(query: Result<Query<CalendarQuery>, QueryRejection>) -> Result<CookCalendar, AppError> {
    let Query(query) = query.map_err(|rejection| AppError::Invalid(vec![rejection.body_text()]))?;
    match CookCalendar::parse(&query.time_zone, query.first_weekday) {
        Ok(calendar) => Ok(calendar),
        Err(ParseCalendarError::UnknownTimeZone) => Err(invalid("timeZone: not a known IANA time zone")),
        Err(ParseCalendarError::FirstWeekdayOutOfRange) => {
            Err(invalid("firstWeekday: from 1 (Sunday) to 7 (Saturday)"))
        }
    }
}

fn invalid(problem: &str) -> AppError {
    AppError::Invalid(vec![problem.to_string()])
}

/// The snapshot of the cook's stored log, with their profile answers, as of the request.
async fn snapshot(
    state: &AppState,
    user_id: Uuid,
    request: &SnapshotRequest<'_>,
) -> Result<ProgressSnapshot, AppError> {
    let profile = read_profile(&state.db, user_id).await?.profile;
    let log = store::cook_log(&state.db, user_id).await?;
    let answers = ProgressAnswers::from(&profile);
    let inputs = SnapshotInputs {
        catalog: &state.library.catalog,
        rank_names: &state.library.ranks,
        language: request.language,
        answers: &answers,
        calendar: request.calendar,
        now: request.now,
    };
    Ok(ProgressSnapshot::compute(&log, &inputs))
}

async fn receive(
    caller: Caller,
    CookLanguage(language): CookLanguage,
    body: Result<Json<BatchFile>, JsonRejection>,
) -> Result<Json<EntriesResponse>, AppError> {
    let Caller { state, user_id } = caller;
    let batch = EntryBatch::parse(json_body(body)?).map_err(AppError::Invalid)?;
    let checks = EntryChecks { received_at: state.clock.now(), graph: state.graph.as_deref() };
    let receipt = Receipt { state: &state, user_id, checks };
    let mut results = Vec::with_capacity(batch.entries.len());
    for raw in &batch.entries {
        results.push(answer(&receipt, raw).await?);
    }
    let request = SnapshotRequest { calendar: &batch.calendar, language: &language, now: receipt.checks.received_at };
    let progress = snapshot(&state, user_id, &request).await?;
    Ok(Json(EntriesResponse { results, progress }))
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
