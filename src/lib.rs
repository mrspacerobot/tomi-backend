//! The Tomi API: the one public service the iPhone app talks to.

pub mod ai;
pub mod auth;
pub mod config;
pub mod db;
pub mod domain;
pub mod error;
pub mod graph_files;
pub mod library_files;
pub mod routes;
pub mod storage;

use ai::Ai;
use auth::Verifier;
use axum::extract::DefaultBodyLimit;
use axum::http::HeaderName;
use axum::Router;
use domain::graph::Graph;
use domain::library::Library;
use sqlx::PgPool;
use std::sync::Arc;
use storage::PhotoStore;
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
    pub library: Arc<Library>,
    /// The pinned cooking graph, or none when this build pins none.
    pub graph: Option<Arc<Graph>>,
    pub ai: Arc<dyn Ai>,
    pub photos: Arc<dyn PhotoStore>,
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
