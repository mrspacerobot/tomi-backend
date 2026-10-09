//! The HTTP routes, one module per feature, merged into one router.
//!
//! Handlers only translate: they read the request, call the domain rules
//! and the database, and shape the answer. No rule is decided here.

use crate::auth::{bearer, AuthError};
use crate::db;
use crate::error::AppError;
use crate::AppState;
use axum::extract::FromRequestParts;
use axum::http::header::AUTHORIZATION;
use axum::http::request::Parts;
use axum::Router;
use uuid::Uuid;

mod bootstrap;
mod health;
mod profile;

pub fn router() -> Router<AppState> {
    Router::new()
        .merge(health::router())
        .merge(bootstrap::router())
        .merge(profile::router())
}

/// The signed-in cook, as Tomi's own user id. A handler that takes this
/// argument can only be reached with a valid token.
pub struct CurrentUser(pub Uuid);

impl FromRequestParts<AppState> for CurrentUser {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, AppError> {
        let header = authorization_header(parts)?;
        let subject = state.verifier.verify(bearer(header)?).await?;
        let user_id = db::users::for_subject(&state.db, subject).await?;
        Ok(CurrentUser(user_id))
    }
}

fn authorization_header(parts: &Parts) -> Result<Option<&str>, AuthError> {
    let Some(value) = parts.headers.get(AUTHORIZATION) else {
        return Ok(None);
    };
    match value.to_str() {
        Ok(text) => Ok(Some(text)),
        Err(_) => Err(AuthError::Invalid),
    }
}
