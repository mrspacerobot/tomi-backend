//! Progress snapshots computed from the stored cook log.

use super::{CookCalendar, ParseCalendarError, ProgressAnswers, ProgressSnapshot, SnapshotInputs};
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
pub async fn snapshot(
    state: &AppState,
    user_id: Uuid,
    request: &SnapshotRequest<'_>,
) -> Result<ProgressSnapshot, AppError> {
    let profile = read_profile(&state.db, user_id).await?.profile;
    let stored = crate::cook_entries::cook_log(&state.db, user_id).await?;
    let log = stored.iter().map(crate::cook_entries::StoredEntry::read).collect::<Result<Vec<_>, _>>()?;
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
