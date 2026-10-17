//! `GET /v1/options`: every list the app offers, with its words.
//!
//! No sign-in needed: onboarding shows these before the cook has an account.

use crate::http::CookLanguage;
use crate::options::{options, Options};
use crate::AppState;
use axum::extract::State;
use axum::routing::get;
use axum::{Json, Router};

pub fn router() -> Router<AppState> {
    Router::new().route("/v1/options", get(list))
}

async fn list(State(state): State<AppState>, CookLanguage(language): CookLanguage) -> Json<Options> {
    Json(options(&state.library, state.foods(), &language))
}
