//! The Tomi API: the one public service the iPhone app talks to.

pub mod ai;
pub mod auth;
pub mod clock;
pub mod config;
pub mod db;
pub mod error;
pub mod graph;
pub mod http;
pub mod ingredients;
pub mod kernel;
pub mod library;
pub mod meta;
pub mod options;
pub mod profile;
pub mod progress;
pub mod recipe;
pub mod scans;
pub mod storage;
pub mod suggestions;

use ai::Ai;
use auth::Verifier;
use axum::extract::DefaultBodyLimit;
use axum::extract::Request;
use axum::http::HeaderName;
use axum::Router;
use clock::Clock;
use error::AppError;
use graph::{Foods, Graph};
use library::Library;
use sqlx::PgPool;
use std::sync::{Arc, LazyLock};
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
    /// What time it is. Routes read it once per request and pass it into the rules.
    pub clock: Arc<dyn Clock>,
}

impl AppState {
    /// The graph, or the error to show when this server runs without one.
    pub fn graph(&self) -> Result<&Graph, AppError> {
        self.graph.as_deref().ok_or(AppError::GraphUnavailable)
    }

    /// Every food the graph knows, none when this server runs without a graph.
    pub fn foods(&self) -> &Foods {
        static NONE: LazyLock<Foods> = LazyLock::new(Foods::empty);
        match &self.graph {
            Some(graph) => &graph.foods,
            None => &NONE,
        }
    }
}

pub fn app(state: AppState) -> Router {
    http::router()
        .with_state(state)
        .layer(DefaultBodyLimit::max(BODY_LIMIT_BYTES))
        // The default trace logs method, path, status and latency. It never
        // logs bodies, which may hold restrictions or a pantry.
        .layer(TraceLayer::new_for_http().make_span_with(request_span))
        .layer(PropagateRequestIdLayer::new(REQUEST_ID))
        .layer(SetRequestIdLayer::new(REQUEST_ID, MakeRequestUuid))
}

fn request_span(request: &Request) -> tracing::Span {
    let request_id = request.headers().get(&REQUEST_ID).and_then(|value| value.to_str().ok()).unwrap_or_default();
    tracing::info_span!("request", request_id, method = %request.method(), path = request.uri().path())
}
