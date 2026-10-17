//! Checks Cloud Run uses to see that the service is up and can serve.

use crate::AppState;
use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::get;
use axum::Router;

pub fn router() -> Router<AppState> {
    Router::new().route("/health", get(alive)).route("/healthz", get(alive)).route("/readyz", get(ready))
}

/// The process is up.
async fn alive() -> &'static str {
    "ok"
}

/// The process can serve: the database answers.
async fn ready(State(state): State<AppState>) -> StatusCode {
    match sqlx::query("select 1").execute(&state.db).await {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::SERVICE_UNAVAILABLE,
    }
}
