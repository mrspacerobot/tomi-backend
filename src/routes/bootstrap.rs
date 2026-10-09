//! What the app needs to know before anything else.

use crate::domain::profile::PROFILE_SCHEMA_VERSION;
use crate::AppState;
use axum::extract::State;
use axum::routing::get;
use axum::{Json, Router};
use serde_json::{json, Value};

pub fn router() -> Router<AppState> {
    Router::new().route("/v1/bootstrap", get(bootstrap))
}

/// No reviewed library, safety snapshot or graph artifact exists yet, so
/// those are null. The app must read null as "nothing to offer", never as
/// "no restrictions apply".
async fn bootstrap(State(state): State<AppState>) -> Json<Value> {
    Json(json!({
        "apiVersions": ["v1"],
        "profileSchemaVersion": PROFILE_SCHEMA_VERSION,
        "minAppVersion": state.min_app_version,
        "libraryVersion": null,
        "safetyVersion": null,
        "graphArtifactVersion": null,
    }))
}
