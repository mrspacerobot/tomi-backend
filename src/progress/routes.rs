//! Progress snapshots computed from the stored cook log.

use super::{CookCalendar, CookEntry, ProgressAnswers, ProgressSnapshot, QuizAnswers, SnapshotInputs};
use crate::error::AppError;
use crate::http::{Caller, CookLanguage};
use crate::kernel::language::Language;
use crate::profile::read as read_profile;
use crate::AppState;
use axum::extract::rejection::QueryRejection;
use axum::extract::Query;
use axum::routing::get;
use axum::{Json, Router};
use jiff::Timestamp;
use serde::Deserialize;
use uuid::Uuid;

pub fn router() -> Router<AppState> {
    Router::new().route("/v1/progress", get(progress))
}

/// The cook's calendar, as `GET /v1/progress` takes it. Both are required: there is no default zone.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct CalendarQuery {
    time_zone: String,
    first_weekday: i64,
}

/// What a snapshot is asked for with, besides the cook: the calendar, the language and the moment of the request.
pub struct SnapshotRequest<'a> {
    pub calendar: &'a CookCalendar,
    pub language: &'a Language,
    pub now: Timestamp,
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
    CookCalendar::parse(&query.time_zone, query.first_weekday)
        .map_err(|error| AppError::Invalid(vec![error.problem("")]))
}

/// The snapshot of the cook's stored log, with their profile answers, as of the request.
pub async fn snapshot(
    state: &AppState,
    user_id: Uuid,
    request: &SnapshotRequest<'_>,
) -> Result<ProgressSnapshot, AppError> {
    let profile = read_profile(&state.db, user_id).await?.profile;
    let log = read_log(state, user_id).await?;
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

/// Reads the cook's entries and derives each quiz result from its recorded first answers.
async fn read_log(state: &AppState, user_id: Uuid) -> Result<Vec<CookEntry>, AppError> {
    let stored = crate::cook_entries::cook_log(&state.db, user_id).await?;
    let answers = QuizAnswers::new(crate::quiz::first_answers(&state.db, user_id).await?);
    Ok(stored.iter().map(|entry| entry.read(&answers)).collect::<Result<Vec<_>, _>>()?)
}
