//! `POST /v1/ingredients/resolve`: what the cook typed, as ingredients.

use crate::error::AppError;
use crate::http::{json_body, Caller, CookLanguage};
use crate::ingredients::{resolve, Resolved};
use crate::kernel::display::Namer;
use crate::AppState;
use axum::extract::rejection::JsonRejection;
use axum::routing::post;
use axum::{Json, Router};
use serde::{Deserialize, Serialize};

const MAX_LABELS: usize = 50;
const MAX_LABEL_CHARACTERS: usize = 80;

pub fn router() -> Router<AppState> {
    Router::new().route("/v1/ingredients/resolve", post(resolve_labels))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ResolveRequest {
    labels: Vec<String>,
}

#[derive(Serialize)]
struct ResolveResponse {
    results: Vec<Resolved>,
}

/// Needs sign-in, so only cooks can probe the table this way.
async fn resolve_labels(
    caller: Caller,
    CookLanguage(language): CookLanguage,
    body: Result<Json<ResolveRequest>, JsonRejection>,
) -> Result<Json<ResolveResponse>, AppError> {
    let request = json_body(body)?;
    check(&request)?;
    let namer = Namer { library: &caller.state.library, foods: caller.state.foods(), language: &language };
    let results = request.labels.iter().map(|label| resolve(label, &namer)).collect();
    Ok(Json(ResolveResponse { results }))
}

fn check(request: &ResolveRequest) -> Result<(), AppError> {
    if request.labels.len() > MAX_LABELS {
        return Err(AppError::Invalid(vec![format!("send at most {MAX_LABELS} labels")]));
    }
    if request.labels.iter().any(|label| label.chars().count() > MAX_LABEL_CHARACTERS) {
        return Err(AppError::Invalid(vec![format!("a label has more than {MAX_LABEL_CHARACTERS} characters")]));
    }
    Ok(())
}
