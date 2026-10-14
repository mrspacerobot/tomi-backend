//! `GET` and `PATCH /v1/profile`: the cook's profile and its version.

use super::db::{self as store, ProfileWrite, StoredProfile};
use crate::error::AppError;
use crate::http::{json_body, CurrentUser};
use crate::profile::{self, PatchError, Profile, ProfilePatch, PROFILE_SCHEMA_VERSION};
use crate::AppState;
use axum::extract::rejection::JsonRejection;
use axum::extract::State;
use axum::routing::get;
use axum::{Json, Router};
use serde::Serialize;

pub fn router() -> Router<AppState> {
    Router::new().route("/v1/profile", get(read).patch(change))
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ProfileResponse {
    version: i64,
    schema_version: i32,
    profile: Profile,
}

impl From<StoredProfile> for ProfileResponse {
    fn from(stored: StoredProfile) -> Self {
        ProfileResponse { version: stored.version, schema_version: PROFILE_SCHEMA_VERSION, profile: stored.profile }
    }
}

async fn read(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
) -> Result<Json<ProfileResponse>, AppError> {
    let stored = store::read(&state.db, user).await?;
    Ok(Json(stored.into()))
}

async fn change(
    State(state): State<AppState>,
    CurrentUser(user_id): CurrentUser,
    body: Result<Json<ProfilePatch>, JsonRejection>,
) -> Result<Json<ProfileResponse>, AppError> {
    let patch = json_body(body)?;
    let expected_version = patch.version;
    let stored = store::read(&state.db, user_id).await?;
    let profile = changed(stored, patch)?;
    let write = ProfileWrite { user_id, expected_version, profile };
    let saved = save(&state, write).await?;
    Ok(Json(saved.into()))
}

/// The profile after the patch, refused when the app read an older
/// version or the change breaks a rule.
fn changed(stored: StoredProfile, patch: ProfilePatch) -> Result<Profile, AppError> {
    if stored.version != patch.version {
        return Err(conflict(patch.version, stored));
    }
    profile::apply(&stored.profile, patch).map_err(refusal)
}

/// Saves, or reports a conflict when someone wrote between our read and
/// our write.
async fn save(state: &AppState, write: ProfileWrite) -> Result<StoredProfile, AppError> {
    if store::write(&state.db, &write).await? {
        let version = write.expected_version + 1;
        return Ok(StoredProfile { version, profile: write.profile });
    }
    let latest = store::read(&state.db, write.user_id).await?;
    Err(conflict(write.expected_version, latest))
}

fn refusal(error: PatchError) -> AppError {
    match error {
        PatchError::Invalid(problems) => AppError::Invalid(problems),
        PatchError::WeakensRestrictions { removed } => AppError::WeakensRestrictions { removed },
    }
}

fn conflict(expected: i64, stored: StoredProfile) -> AppError {
    let current = serde_json::to_value(ProfileResponse::from(stored));
    AppError::VersionConflict { expected, current: current.unwrap_or_default() }
}
