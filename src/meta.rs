//! What the app needs before anything else: whether the process and the database
//! are up, and which versions this server speaks.

use crate::AppState;
use axum::Router;

mod bootstrap;
mod health;

pub fn router() -> Router<AppState> {
    Router::new().merge(health::router()).merge(bootstrap::router())
}
