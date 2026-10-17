//! `POST /v2/recipes/recommend`: graph recipes the cook can make tonight.
//!
//! Code decides what may be offered. The model only reorders the short
//! list; without it, or when it fails, the order is by what the cook has.

use crate::ai::RankQuestion;
use crate::error::AppError;
use crate::graph::Graph;
use crate::http::{json_body, Caller, CookLanguage};
use crate::kernel::ids::SavedId;
use crate::kernel::language::Language;
use crate::profile::{read as read_profile, Profile};
use crate::scans::{self, Confirmation};
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
    Router::new().route("/v2/recipes/recommend", post(special))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct SpecialRequest {
    /// What the cook confirmed or typed. Cupboard staples are added here.
    ingredients: BTreeSet<SavedId>,
    /// The scan these came from, if any, so a kept photo learns the answer.
    scan_id: Option<Uuid>,
    /// Staples the cook does not have tonight, though they usually do. Never removes something they confirmed.
    #[serde(default)]
    staples_left_out: BTreeSet<SavedId>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SpecialResponse {
    graph_version: String,
    graph_channel: &'static str,
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
    validate_size(&request)?;
    let graph = state.graph()?;
    validate_ingredients(graph, &request)?;
    confirm_scan(&state, user_id, &request).await?;
    let profile = read_profile(&state.db, user_id).await?.profile;
    let input = CandidateInput { request: &request, profile: &profile };
    let by_overlap = candidates(&state, graph, input);
    let (model, ranked_by) = model_order(&state, &by_overlap, &profile).await;
    let order = ranked(by_overlap, &model);
    Ok(Json(response(graph, order, Answer { ranked_by, profile: &profile, language: &language })))
}

fn validate_size(request: &SpecialRequest) -> Result<(), AppError> {
    if request.ingredients.len() <= MAX_INGREDIENTS {
        return Ok(());
    }
    Err(AppError::Invalid(vec![format!("send at most {MAX_INGREDIENTS} ingredients")]))
}

fn validate_ingredients(graph: &Graph, request: &SpecialRequest) -> Result<(), AppError> {
    let mut listed = request.ingredients.iter().chain(&request.staples_left_out);
    if listed.all(|id| graph.foods.knows(id)) {
        return Ok(());
    }
    Err(AppError::Invalid(vec!["ingredients must be listed by the cooking graph".into()]))
}

/// Every recipe the cook may make tonight, fewest to buy first. Until the
/// server keeps the cook log, only experience grants open levels.
struct CandidateInput<'a> {
    request: &'a SpecialRequest,
    profile: &'a Profile,
}

fn candidates(state: &AppState, graph: &Graph, input: CandidateInput) -> Vec<Candidate> {
    let open = open_levels(&state.library.catalog, input.profile.experience, &CookCounts::default());
    let ingredients = available(state, input.request, input.profile);
    let tonight = Tonight { profile: input.profile, available: ingredients, open: &open };
    overlap_order(special::candidates(&state.library, graph, &tonight))
}

/// A kept photo learns which ingredients the cook confirmed for it.
async fn confirm_scan(state: &AppState, user_id: Uuid, request: &SpecialRequest) -> Result<(), AppError> {
    let Some(scan_id) = request.scan_id else {
        return Ok(());
    };
    let ingredients: Vec<SavedId> = request.ingredients.iter().cloned().collect();
    let confirmation = Confirmation { user_id, scan_id, ingredients: &ingredients };
    scans::confirm(&state.db, &confirmation).await
}

/// What the cook has tonight: what they confirmed, and their staples less any they leave out. Until they answer
/// the cupboard question the staples a kitchen is taken to have stand in for theirs.
fn available(state: &AppState, request: &SpecialRequest, profile: &Profile) -> BTreeSet<SavedId> {
    let usual =
        if profile.cupboard_answered { profile.cupboard_staples.clone() } else { state.library.staples.assumed() };
    let at_home = usual.into_iter().filter(|staple| !request.staples_left_out.contains(staple));
    request.ingredients.iter().cloned().chain(at_home).collect()
}

/// The model's order over the short list, or none when there is nothing to
/// choose between or the model is unavailable.
async fn model_order(state: &AppState, by_overlap: &[Candidate], profile: &Profile) -> (Vec<Ranked>, &'static str) {
    if by_overlap.len() < 2 {
        return (Vec::new(), "overlap");
    }
    let short_list = &by_overlap[..by_overlap.len().min(MODEL_SHORT_LIST)];
    let question = RankQuestion { candidates: short_list, profile };
    match state.ai.rank_recipes(question).await {
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

fn response(graph: &Graph, order: Vec<(Candidate, Option<String>)>, answer: Answer) -> SpecialResponse {
    let context = OfferContext { graph, profile: answer.profile, language: answer.language };
    let to_offer = |(candidate, reason): (Candidate, Option<String>)| Offer::new(candidate, reason, &context);
    let mut offers = order.into_iter().take(OFFERS).filter_map(to_offer);
    SpecialResponse {
        graph_version: graph.version.clone(),
        graph_channel: graph.channel.as_str(),
        ranked_by: answer.ranked_by,
        special: offers.next(),
        alternatives: offers.collect(),
    }
}
