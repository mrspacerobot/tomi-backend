use std::sync::Arc;
use tomi_api::auth::Verifier;
use tomi_api::config::Config;
use tomi_api::{app, db, AppState};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    // JSON lines, which Cloud Logging reads as structured entries.
    tracing_subscriber::fmt()
        .json()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    if let Err(error) = run().await {
        tracing::error!(%error, "could not start");
        std::process::exit(1);
    }
}

async fn run() -> Result<(), String> {
    let config = Config::from_env()?;
    let pool = db::connect(&config.database_url, config.db_max_connections)
        .await
        .map_err(|error| format!("database: {error}"))?;
    if config.run_migrations {
        db::migrate(&pool)
            .await
            .map_err(|error| format!("migrations: {error}"))?;
    }

    let verifier = Verifier::new(
        &config.supabase_url,
        config.supabase_jwt_secret.as_deref(),
        true,
    );
    let state = AppState {
        db: pool,
        verifier: Arc::new(verifier),
        min_app_version: config.min_app_version.clone(),
    };

    let listener = tokio::net::TcpListener::bind(("0.0.0.0", config.port))
        .await
        .map_err(|error| format!("port {}: {error}", config.port))?;
    tracing::info!(port = config.port, "listening");
    axum::serve(listener, app(state))
        .with_graceful_shutdown(shutdown())
        .await
        .map_err(|error| error.to_string())
}

/// Cloud Run sends SIGTERM before stopping an instance.
async fn shutdown() {
    let ctrl_c = async {
        let _ = tokio::signal::ctrl_c().await;
    };
    #[cfg(unix)]
    let terminate = async {
        if let Ok(mut signal) =
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
        {
            signal.recv().await;
        }
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();
    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
