//! The routes of API version one that exist so far.

use crate::auth::{bearer, AuthError};
use crate::db;
use crate::error::AppError;
use crate::profile::{self, PatchError, Profile, ProfilePatch, PROFILE_SCHEMA_VERSION};
use crate::AppState;
use axum::extract::rejection::JsonRejection;
use axum::extract::{FromRequestParts, State};
use axum::http::header::AUTHORIZATION;
use axum::http::request::Parts;
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};
use serde::Serialize;
use serde_json::{json, Value};
use uuid::Uuid;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/healthz", get(healthz))
        .route("/readyz", get(readyz))
        .route("/v1/bootstrap", get(bootstrap))
        .route("/v1/profile", get(get_profile).patch(patch_profile))
}

/// The signed-in cook, as our own user id.
pub struct CurrentUser(pub Uuid);

impl FromRequestParts<AppState> for CurrentUser {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, AppError> {
        let header = parts
            .headers
            .get(AUTHORIZATION)
            .map(|value| value.to_str().map_err(|_| AuthError::Invalid));
        let header = header.transpose()?;
        let subject = state.verifier.verify(bearer(header)?).await?;
        Ok(CurrentUser(db::user_for_subject(&state.db, subject).await?))
    }
}

/// The process is up. Cloud Run uses this as its liveness check.
async fn healthz() -> &'static str {
    "ok"
}

/// The process can serve: the database answers.
async fn readyz(State(state): State<AppState>) -> StatusCode {
    match sqlx::query("select 1").execute(&state.db).await {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::SERVICE_UNAVAILABLE,
    }
}

/// What the app needs to know before anything else. No reviewed library,
/// safety snapshot or graph artifact exists yet, so those are null: the app
/// must treat null as "nothing to offer", never as "no restrictions apply".
async fn bootstrap(State(state): State<AppState>) -> Json<Value> {
    Json(json!({
        "api_versions": ["v1"],
        "profile_schema_version": PROFILE_SCHEMA_VERSION,
        "min_app_version": state.min_app_version,
        "library_version": null,
        "safety_version": null,
        "graph_artifact_version": null,
    }))
}

#[derive(Serialize)]
struct ProfileResponse {
    version: i64,
    schema_version: i32,
    profile: Profile,
}

impl ProfileResponse {
    fn new(version: i64, profile: Profile) -> Self {
        ProfileResponse {
            version,
            schema_version: PROFILE_SCHEMA_VERSION,
            profile,
        }
    }
}

async fn get_profile(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
) -> Result<Json<ProfileResponse>, AppError> {
    let stored = db::read_profile(&state.db, user).await?;
    Ok(Json(ProfileResponse::new(stored.version, stored.profile)))
}

async fn patch_profile(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    body: Result<Json<ProfilePatch>, JsonRejection>,
) -> Result<Json<ProfileResponse>, AppError> {
    let Json(patch) = body.map_err(|rejection| AppError::Invalid(vec![rejection.body_text()]))?;
    let expected = patch.version;
    let stored = db::read_profile(&state.db, user).await?;
    if stored.version != expected {
        return Err(conflict(expected, stored.version, &stored.profile));
    }

    let next = profile::apply(&stored.profile, patch).map_err(|error| match error {
        PatchError::Invalid(problems) => AppError::Invalid(problems),
        PatchError::WeakensRestrictions { removed } => AppError::WeakensRestrictions {
            removed: removed.iter().map(ToString::to_string).collect(),
        },
    })?;

    if !db::write_profile(&state.db, user, expected, &next).await? {
        // Someone wrote between our read and our write.
        let latest = db::read_profile(&state.db, user).await?;
        return Err(conflict(expected, latest.version, &latest.profile));
    }
    Ok(Json(ProfileResponse::new(expected + 1, next)))
}

fn conflict(expected: i64, version: i64, profile: &Profile) -> AppError {
    let current =
        serde_json::to_value(ProfileResponse::new(version, profile.clone())).unwrap_or(Value::Null);
    AppError::VersionConflict { expected, current }
}
