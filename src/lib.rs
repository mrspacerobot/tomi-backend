//! The Tomi API: one public service the iPhone app talks to.

pub mod auth;
pub mod config;
pub mod db;
pub mod error;
pub mod ids;
pub mod profile;
pub mod routes;

use auth::Verifier;
use axum::extract::DefaultBodyLimit;
use axum::http::HeaderName;
use axum::Router;
use sqlx::PgPool;
use std::sync::Arc;
use tower_http::request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer};
use tower_http::trace::TraceLayer;

/// Requests are small JSON; anything bigger is a mistake or an attack.
const BODY_LIMIT_BYTES: usize = 64 * 1024;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub verifier: Arc<Verifier>,
    pub min_app_version: String,
}

pub fn app(state: AppState) -> Router {
    let request_id = HeaderName::from_static("x-request-id");
    routes::router()
        .with_state(state)
        .layer(DefaultBodyLimit::max(BODY_LIMIT_BYTES))
        // The default trace logs method, path, status and latency. It never
        // logs bodies, which may hold restrictions or a pantry.
        .layer(TraceLayer::new_for_http())
        .layer(PropagateRequestIdLayer::new(request_id.clone()))
        .layer(SetRequestIdLayer::new(request_id, MakeRequestUuid))
}
