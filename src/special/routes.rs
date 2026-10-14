//! `POST /v1/lessons/suggest`: tonight's special and up to two alternatives.
//!
//! Code decides what may be offered. The model only reorders the short
//! list; without it, or when it fails, the order is by what the cook has.

use crate::ai::RankQuestion;
use crate::error::AppError;
use crate::http::{json_body, Caller, CookLanguage};
use crate::kernel::display::Namer;
use crate::kernel::ids::SavedId;
use crate::kernel::language::Language;
use crate::profile::db as store;
use crate::profile::Profile;
use crate::scans::db::{self as fridge_photos, Confirmation};
use crate::special::progress::{open_levels, CookCounts};
use crate::special::{self, overlap_order, ranked, Candidate, Offer, OfferContext, Ranked, Tonight, MODEL_SHORT_LIST};
use crate::AppState;
use axum::extract::rejection::JsonRejection;
use axum::routing::post;
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use uuid::Uuid;

/// A special and two alternatives, as on Tomi's home screen.
const OFFERS: usize = 3;
const MAX_INGREDIENTS: usize = 200;

pub fn router() -> Router<AppState> {
    Router::new().route("/v1/lessons/suggest", post(special))
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

async fn special(
    caller: Caller,
    CookLanguage(language): CookLanguage,
    body: Result<Json<SpecialRequest>, JsonRejection>,
) -> Result<Json<SpecialResponse>, AppError> {
    let Caller { state, user_id } = caller;
    let request = json_body(body)?;
    if request.ingredients.len() > MAX_INGREDIENTS {
        return Err(AppError::Invalid(vec![format!("send at most {MAX_INGREDIENTS} ingredients")]));
    }
    confirm_scan(&state, user_id, &request).await?;
    let profile = store::read(&state.db, user_id).await?.profile;
    let by_overlap = candidates(&state, &request, &profile);
    let (model, ranked_by) = model_order(&state, &by_overlap, &profile).await;
    let order = ranked(by_overlap, &model);
    Ok(Json(response(&state, order, Answer { ranked_by, profile: &profile, language: &language })))
}

/// Every lesson the cook may make tonight, fewest to buy first. Until the
/// server keeps the cook log, only experience grants open levels.
fn candidates(state: &AppState, request: &SpecialRequest, profile: &Profile) -> Vec<Candidate> {
    let open = open_levels(&state.library.catalog, profile.experience, &CookCounts::default());
    let tonight = Tonight { profile, available: available(request, profile), open: &open };
    overlap_order(special::candidates(&state.library, &tonight))
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

/// How the answer was made, and for whom.
struct Answer<'a> {
    ranked_by: &'static str,
    profile: &'a Profile,
    language: &'a Language,
}

fn response(state: &AppState, order: Vec<(Candidate, Option<String>)>, answer: Answer) -> SpecialResponse {
    let namer = Namer { library: &state.library, language: answer.language };
    let context = OfferContext { namer, profile: answer.profile };
    let to_offer = |(candidate, reason): (Candidate, Option<String>)| Offer::new(candidate, reason, &context);
    let mut offers = order.into_iter().take(OFFERS).map(to_offer);
    SpecialResponse {
        library_version: state.library.version,
        ranked_by: answer.ranked_by,
        special: offers.next(),
        alternatives: offers.collect(),
    }
}
