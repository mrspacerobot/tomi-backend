//! Starts the Tomi API: reads settings, connects, migrates, serves.

use sqlx::PgPool;
use std::sync::Arc;
use tokio::net::TcpListener;
use tomi_api::auth::Verifier;
use tomi_api::config::Config;
use tomi_api::{app, db, AppState};
use tracing_subscriber::EnvFilter;

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
    tracing_subscriber::fmt()
        .json()
        .with_env_filter(filter)
        .init();
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

#[expect(
    clippy::cognitive_complexity,
    reason = "tracing's macro expands into branches no reader sees"
)]
fn log_start_failure(error: &str) {
    tracing::error!(%error, "could not start");
}

#[expect(
    clippy::cognitive_complexity,
    reason = "tracing's macro expands into branches no reader sees"
)]
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
    let secret = config.supabase_jwt_secret.as_deref();
    let verifier = Verifier::new(&config.supabase_url, secret);
    let verifier = verifier.map_err(|error| format!("auth: {error}"))?;
    Ok(AppState {
        db: pool,
        verifier: Arc::new(verifier),
        min_app_version: config.min_app_version.clone(),
    })
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
