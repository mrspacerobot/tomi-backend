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

/// The library this server serves is named. There is no safety snapshot yet,
/// and a build that pins no graph has none, so those are null. The app must
/// read null as "nothing to offer", never as "no restrictions apply".
async fn bootstrap(State(state): State<AppState>) -> Json<Value> {
    let (graph_version, graph_channel) = graph_labels(&state);
    Json(json!({
        "apiVersions": ["v1"],
        "profileSchemaVersion": PROFILE_SCHEMA_VERSION,
        "minAppVersion": state.min_app_version,
        "libraryVersion": state.library.version,
        "safetyVersion": null,
        "graphArtifactVersion": graph_version,
        "graphChannel": graph_channel,
    }))
}

/// The loaded graph's version and channel, both null without a graph.
fn graph_labels(state: &AppState) -> (Value, Value) {
    match &state.graph {
        Some(graph) => (json!(graph.version), json!(graph.channel.as_str())),
        None => (Value::Null, Value::Null),
    }
}
