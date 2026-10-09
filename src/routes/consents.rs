//! `/v1/consents`: what the cook agreed to, given and withdrawn one purpose
//! at a time.

use super::{json_body, Caller, CurrentUser};
use crate::db::{consents, fridge_photos};
use crate::domain::consent::Purpose;
use crate::error::AppError;
use crate::AppState;
use axum::extract::rejection::JsonRejection;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{get, put};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub fn router() -> Router<AppState> {
    Router::new().route("/v1/consents", get(list)).route("/v1/consents/{purpose}", put(give).delete(withdraw))
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ConsentState {
    purpose: Purpose,
    current_version: i32,
    agreed: bool,
}

/// The app sends the version of the text it showed the cook.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Agreement {
    version: i32,
}

async fn list(
    State(state): State<AppState>,
    CurrentUser(user_id): CurrentUser,
) -> Result<Json<Vec<ConsentState>>, AppError> {
    let mut found = Vec::new();
    for purpose in Purpose::ALL {
        let agreed = consents::is_current(&state.db, user_id, purpose).await?;
        let current_version = purpose.current_version();
        found.push(ConsentState { purpose, current_version, agreed });
    }
    Ok(Json(found))
}

async fn give(
    caller: Caller,
    Path(slug): Path<String>,
    body: Result<Json<Agreement>, JsonRejection>,
) -> Result<StatusCode, AppError> {
    let Caller { state, user_id } = caller;
    let purpose = purpose(&slug)?;
    let agreement = json_body(body)?;
    if agreement.version != purpose.current_version() {
        let problem = format!("the current text is version {}", purpose.current_version());
        return Err(AppError::Invalid(vec![problem]));
    }
    if purpose == Purpose::PhotoDataset && !state.photos.is_configured() {
        return Err(AppError::StorageUnavailable);
    }
    consents::give(&state.db, user_id, purpose).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Withdrawing the dataset consent also deletes every photo kept with it.
async fn withdraw(
    State(state): State<AppState>,
    CurrentUser(user_id): CurrentUser,
    Path(slug): Path<String>,
) -> Result<StatusCode, AppError> {
    let purpose = purpose(&slug)?;
    consents::withdraw(&state.db, user_id, purpose).await?;
    if purpose == Purpose::PhotoDataset {
        delete_kept_photos(&state, user_id).await?;
    }
    Ok(StatusCode::NO_CONTENT)
}

/// Deletes the files first, then the records, so a failure can be retried:
/// the records still say which files to delete.
async fn delete_kept_photos(state: &AppState, user_id: Uuid) -> Result<(), AppError> {
    let paths = fridge_photos::paths(&state.db, user_id).await?;
    if paths.is_empty() {
        return Ok(());
    }
    let deleted = state.photos.delete(&paths).await;
    deleted.map_err(|_| AppError::StorageUnavailable)?;
    fridge_photos::delete_all(&state.db, user_id).await
}

fn purpose(slug: &str) -> Result<Purpose, AppError> {
    Purpose::parse(slug).ok_or(AppError::NotFound)
}
