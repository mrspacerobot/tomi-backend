//! The Tomi API: the one public service the iPhone app talks to.

pub mod auth;
pub mod config;
pub mod db;
pub mod domain;
pub mod error;
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
const REQUEST_ID: HeaderName = HeaderName::from_static("x-request-id");

/// What every handler can reach. Cheap to clone: each field is shared.
#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub verifier: Arc<Verifier>,
    pub min_app_version: String,
}

pub fn app(state: AppState) -> Router {
    routes::router()
        .with_state(state)
        .layer(DefaultBodyLimit::max(BODY_LIMIT_BYTES))
        // The default trace logs method, path, status and latency. It never
        // logs bodies, which may hold restrictions or a pantry.
        .layer(TraceLayer::new_for_http())
        .layer(PropagateRequestIdLayer::new(REQUEST_ID))
        .layer(SetRequestIdLayer::new(REQUEST_ID, MakeRequestUuid))
}
