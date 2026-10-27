//! Starts the Tomi API: reads settings, connects, migrates, serves.

use sqlx::PgPool;
use std::path::Path;
use std::sync::Arc;
use tokio::net::TcpListener;
use tomi_api::ai::{Ai, Gemini, NoAi};
use tomi_api::auth::Verifier;
use tomi_api::clock::SystemClock;
use tomi_api::config::{AiConfig, Config, PhotoConfig};
use tomi_api::library::Library;
use tomi_api::storage::{NoStore, PhotoStore, SupabaseStore};
use tomi_api::{app, db, graph, library, AppState};
use tracing_subscriber::EnvFilter;

/// The graph this build is pinned to, written by `scripts/fetch-graph.sh`.
const GRAPH_LOCK: &str = include_str!("../graph.lock");

#[tokio::main]
async fn main() {
    start_logging();
    if let Err(error) = run().await {
        log_start_failure(&error);
        std::process::exit(1);
    }
}

/// JSON lines, which Cloud Logging reads as structured entries.
fn start_logging() {
    let filter = match EnvFilter::try_from_default_env() {
        Ok(filter) => filter,
        Err(_) => EnvFilter::new("info"),
    };
    tracing_subscriber::fmt().json().with_env_filter(filter).init();
}

async fn run() -> Result<(), String> {
    let config = Config::from_env()?;
    let pool = database(&config).await?;
    let state = state(&config, pool)?;
    let listener = TcpListener::bind(("0.0.0.0", config.port)).await;
    let listener = listener.map_err(|error| format!("port {}: {error}", config.port))?;
    log_listening(config.port);
    let server = axum::serve(listener, app(state)).with_graceful_shutdown(shutdown_signal());
    server.await.map_err(|error| error.to_string())
}

#[expect(clippy::cognitive_complexity, reason = "tracing's macro expands into branches no reader sees")]
fn log_start_failure(error: &str) {
    tracing::error!(%error, "could not start");
}

#[expect(clippy::cognitive_complexity, reason = "tracing's macro expands into branches no reader sees")]
fn log_listening(port: u16) {
    tracing::info!(port, "listening");
}

async fn database(config: &Config) -> Result<PgPool, String> {
    let pool = db::connect(&config.database_url, config.db_max_connections).await;
    let pool = pool.map_err(|error| format!("database: {error}"))?;
    if config.run_migrations {
        let migrated = db::migrate(&pool).await;
        migrated.map_err(|error| format!("migrations: {error}"))?;
    }
    Ok(pool)
}

fn state(config: &Config, pool: PgPool) -> Result<AppState, String> {
    let verifier = verifier(config)?;
    let mut library = library(&config.library_dir)?;
    let graph = graph::files::start(GRAPH_LOCK, &config.graph)?;
    agree(&mut library, graph.as_ref())?;
    Ok(AppState {
        db: pool,
        verifier: Arc::new(verifier),
        min_app_version: config.min_app_version.clone(),
        library: Arc::new(library),
        graph: graph.map(Arc::new),
        ai: ai(&config.ai)?,
        photos: photos(config.supabase_url.as_str(), &config.photos)?,
        clock: Arc::new(SystemClock),
    })
}

fn verifier(config: &Config) -> Result<Verifier, String> {
    let secret = config.supabase_jwt_secret.as_deref();
    Verifier::new(&config.supabase_url, secret).map_err(|error| format!("auth: {error}"))
}

/// The library's restrictions may only name facts the graph has, or there is no start.
fn agree(library: &mut Library, graph: Option<&graph::Graph>) -> Result<(), String> {
    let Some(graph) = graph else {
        return Ok(());
    };
    library.agree_with(graph)?;
    Ok(())
}

/// The library, or no start at all: without it nothing can be offered.
fn library(folder: &str) -> Result<Library, String> {
    let files = library::files::read(Path::new(folder));
    let files = files.map_err(|error| format!("library in {folder}: {error}"))?;
    Library::parse(&files).map_err(|error| format!("library: {error}"))
}

fn ai(config: &AiConfig) -> Result<Arc<dyn Ai>, String> {
    let Some(key) = &config.gemini_api_key else {
        return Ok(Arc::new(NoAi));
    };
    let gemini = Gemini::new(key.clone(), config.gemini_model.clone());
    let gemini = gemini.map_err(|error| format!("ai: {error}"))?;
    Ok(Arc::new(gemini))
}

fn photos(project_url: &str, config: &PhotoConfig) -> Result<Arc<dyn PhotoStore>, String> {
    let Some(key) = &config.supabase_secret_key else {
        return Ok(Arc::new(NoStore));
    };
    let store = SupabaseStore::new(project_url, &config.bucket, key.clone());
    let store = store.map_err(|error| format!("photo storage: {error}"))?;
    Ok(Arc::new(store))
}

/// Cloud Run sends SIGTERM before stopping an instance; Ctrl-C locally.
async fn shutdown_signal() {
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {},
        () = terminate() => {},
    }
}

#[cfg(unix)]
async fn terminate() {
    use tokio::signal::unix::{signal, SignalKind};
    if let Ok(mut terminate) = signal(SignalKind::terminate()) {
        terminate.recv().await;
    }
}

#[cfg(not(unix))]
async fn terminate() {
    std::future::pending::<()>().await;
}
