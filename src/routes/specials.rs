//! `POST /v1/specials`: tonight's special and up to two alternatives.
//!
//! Code decides what may be offered. The model only reorders the short
//! list; without it, or when it fails, the order is by what the cook has.

use super::{json_body, CurrentUser};
use crate::ai::RankQuestion;
use crate::db::fridge_photos::{self, Confirmation};
use crate::db::profile as store;
use crate::domain::ids::SavedId;
use crate::domain::profile::Profile;
use crate::domain::special::{self, overlap_order, ranked, Candidate, Ranked, Tonight, MODEL_SHORT_LIST};
use crate::error::AppError;
use crate::AppState;
use axum::extract::rejection::JsonRejection;
use axum::extract::State;
use axum::routing::post;
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use uuid::Uuid;

/// A special and two alternatives, as on Tomi's home screen.
const OFFERS: usize = 3;
const MAX_INGREDIENTS: usize = 200;

pub fn router() -> Router<AppState> {
    Router::new().route("/v1/specials", post(special))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct SpecialRequest {
    /// What the cook confirmed or typed. Cupboard staples are added here.
    ingredients: BTreeSet<SavedId>,
    /// The scan these came from, if any, so a kept photo learns the answer.
    scan_id: Option<Uuid>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SpecialResponse {
    library_version: i64,
    /// `model` or `overlap`, so the app can tell how the order was made.
    ranked_by: &'static str,
    special: Option<Offer>,
    alternatives: Vec<Offer>,
}

#[derive(Serialize)]
struct Offer {
    #[serde(flatten)]
    candidate: Candidate,
    reason: Option<String>,
}

async fn special(
    State(state): State<AppState>,
    CurrentUser(user_id): CurrentUser,
    body: Result<Json<SpecialRequest>, JsonRejection>,
) -> Result<Json<SpecialResponse>, AppError> {
    let request = json_body(body)?;
    if request.ingredients.len() > MAX_INGREDIENTS {
        return Err(AppError::Invalid(vec![format!("send at most {MAX_INGREDIENTS} ingredients")]));
    }
    confirm_scan(&state, user_id, &request).await?;
    let profile = store::read(&state.db, user_id).await?.profile;
    let tonight = Tonight { profile: &profile, available: available(&request, &profile) };
    let by_overlap = overlap_order(special::candidates(&state.library, &tonight));
    let (model, ranked_by) = model_order(&state, &by_overlap, &profile).await;
    Ok(Json(response(&state, ranked(by_overlap, &model), ranked_by)))
}

/// A kept photo learns which ingredients the cook confirmed for it.
async fn confirm_scan(state: &AppState, user_id: Uuid, request: &SpecialRequest) -> Result<(), AppError> {
    let Some(scan_id) = request.scan_id else {
        return Ok(());
    };
    let ingredients: Vec<SavedId> = request.ingredients.iter().cloned().collect();
    let confirmation = Confirmation { user_id, scan_id, ingredients: &ingredients };
    fridge_photos::confirm(&state.db, &confirmation).await
}

fn available(request: &SpecialRequest, profile: &Profile) -> BTreeSet<SavedId> {
    let staples = profile.cupboard_staples.iter().cloned();
    request.ingredients.iter().cloned().chain(staples).collect()
}

/// The model's order over the short list, or none when there is nothing to
/// choose between or the model is unavailable.
async fn model_order(state: &AppState, by_overlap: &[Candidate], profile: &Profile) -> (Vec<Ranked>, &'static str) {
    if by_overlap.len() < 2 {
        return (Vec::new(), "overlap");
    }
    let short_list = &by_overlap[..by_overlap.len().min(MODEL_SHORT_LIST)];
    let question = RankQuestion { candidates: short_list, profile };
    match state.ai.rank_lessons(question).await {
        Ok(model) => (model, "model"),
        Err(_) => (Vec::new(), "overlap"),
    }
}

fn response(state: &AppState, order: Vec<(Candidate, Option<String>)>, ranked_by: &'static str) -> SpecialResponse {
    let mut offers = order.into_iter().take(OFFERS).map(|(candidate, reason)| Offer { candidate, reason });
    SpecialResponse {
        library_version: state.library.version,
        ranked_by,
        special: offers.next(),
        alternatives: offers.collect(),
    }
}
