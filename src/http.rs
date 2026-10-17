//! The HTTP pieces every feature shares, and the router that merges each feature's routes.
//!
//! Handlers only translate: they read the request, call the feature's rules
//! and the database, and shape the answer. No rule is decided here.

use crate::auth::{bearer, users, AuthError};
use crate::error::AppError;
use crate::kernel::language::Language;
use crate::AppState;
use axum::extract::rejection::JsonRejection;
use axum::extract::FromRequestParts;
use axum::http::header::{ACCEPT_LANGUAGE, AUTHORIZATION};
use axum::http::request::Parts;
use axum::{Json, Router};
use std::convert::Infallible;
use uuid::Uuid;

pub fn router() -> Router<AppState> {
    Router::new()
        .merge(crate::meta::router())
        .merge(crate::options::router())
        .merge(crate::ingredients::router())
        .merge(crate::profile::router())
        .merge(crate::scans::router())
        .merge(crate::special::router())
}

/// A JSON body, or the reason it could not be read as a refusal.
pub fn json_body<T>(body: Result<Json<T>, JsonRejection>) -> Result<T, AppError> {
    match body {
        Ok(Json(value)) => Ok(value),
        Err(rejection) => Err(AppError::Invalid(vec![rejection.body_text()])),
    }
}

/// The signed-in cook, as Tomi's own user id. A handler that takes this
/// argument can only be reached with a valid token.
pub struct CurrentUser(pub Uuid);

impl FromRequestParts<AppState> for CurrentUser {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, AppError> {
        let header = authorization_header(parts)?;
        let subject = state.verifier.verify(bearer(header)?).await?;
        let user_id = users::for_subject(&state.db, subject).await?;
        Ok(CurrentUser(user_id))
    }
}

/// The signed-in cook together with the app state, for handlers that also
/// take a path and a body and would otherwise need four parameters.
pub struct Caller {
    pub state: AppState,
    pub user_id: Uuid,
}

impl FromRequestParts<AppState> for Caller {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, AppError> {
        let CurrentUser(user_id) = CurrentUser::from_request_parts(parts, state).await?;
        Ok(Caller { state: state.clone(), user_id })
    }
}

/// The language the app asks for, English when it asks for none.
pub struct CookLanguage(pub Language);

impl<S: Send + Sync> FromRequestParts<S> for CookLanguage {
    type Rejection = Infallible;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Infallible> {
        let header = parts.headers.get(ACCEPT_LANGUAGE).and_then(|value| value.to_str().ok());
        Ok(CookLanguage(Language::from_accept_language(header)))
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
