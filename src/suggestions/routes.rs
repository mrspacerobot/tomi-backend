//! `POST /v1/lessons/suggest`: graph recipes the cook can make tonight.
//!
//! Code selects and ranks eligible dishes from the cook's confirmed foods and pantry.

use crate::cook_entries::cook_log;
use crate::error::AppError;
use crate::graph::Graph;
use crate::http::{json_body, Caller, CookLanguage};
use crate::kernel::ids::SavedId;
use crate::kernel::language::Language;
use crate::profile::{read as read_profile, Profile};
use crate::progress::{open_levels, CookCounts, OpenLevels, StoredEntry};
use crate::scans::{self, Confirmation};
use crate::suggestions::{self, overlap_order, Candidate, Suggestion, SuggestionContext, Tonight};
use crate::AppState;
use axum::extract::rejection::JsonRejection;
use axum::routing::post;
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use uuid::Uuid;

/// A special and two alternatives, as on Tomi's home screen.
const MAX_SUGGESTIONS: usize = 3;
const MAX_INGREDIENTS: usize = 200;

pub fn router() -> Router<AppState> {
    Router::new().route("/v1/lessons/suggest", post(suggest))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct SuggestionsRequest {
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
struct SuggestionsResponse {
    graph_version: String,
    graph_channel: &'static str,
    /// Kept for API compatibility. This value is always `overlap`.
    ranked_by: &'static str,
    special: Option<Suggestion>,
    alternatives: Vec<Suggestion>,
}

async fn suggest(
    caller: Caller,
    CookLanguage(language): CookLanguage,
    body: Result<Json<SuggestionsRequest>, JsonRejection>,
) -> Result<Json<SuggestionsResponse>, AppError> {
    let Caller { state, user_id } = caller;
    let request = json_body(body)?;
    validate_size(&request)?;
    let graph = state.graph()?;
    validate_ingredients(graph, &request)?;
    confirm_scan(&state, user_id, &request).await?;
    let profile = read_profile(&state.db, user_id).await?.profile;
    let open = cook_open_levels(&state, user_id, &profile).await?;
    let input = CandidateInput { request: &request, profile: &profile, open: &open };
    let by_overlap = candidates(&state, graph, input);
    Ok(Json(response(graph, by_overlap, Answer { profile: &profile, language: &language })))
}

fn validate_size(request: &SuggestionsRequest) -> Result<(), AppError> {
    if request.ingredients.len() <= MAX_INGREDIENTS {
        return Ok(());
    }
    Err(AppError::Invalid(vec![format!("send at most {MAX_INGREDIENTS} ingredients")]))
}

fn validate_ingredients(graph: &Graph, request: &SuggestionsRequest) -> Result<(), AppError> {
    let mut listed = request.ingredients.iter().chain(&request.staples_left_out);
    if listed.all(|id| graph.foods.knows(id)) {
        return Ok(());
    }
    Err(AppError::Invalid(vec!["ingredients must be listed by the cooking graph".into()]))
}

/// Every eligible dish, deterministically ordered by confirmed ingredients and pantry overlap.
struct CandidateInput<'a> {
    request: &'a SuggestionsRequest,
    profile: &'a Profile,
    open: &'a OpenLevels,
}

/// The levels this cook has opened with their own finished cooks and their experience answer. Progress only
/// decides which levels are offered; the cook's restrictions still decide every food.
async fn cook_open_levels(state: &AppState, user_id: Uuid, profile: &Profile) -> Result<OpenLevels, AppError> {
    let stored = cook_log(&state.db, user_id).await?;
    let log = stored.iter().map(StoredEntry::read).collect::<Result<Vec<_>, _>>()?;
    let counts = CookCounts::from_log(&log);
    Ok(open_levels(&state.library.catalog, profile.experience, &counts))
}

fn candidates(state: &AppState, graph: &Graph, input: CandidateInput) -> Vec<Candidate> {
    let ingredients = available(state, input.request, input.profile);
    let tonight = Tonight {
        profile: input.profile,
        available: ingredients,
        confirmed: input.request.ingredients.clone(),
        open: input.open,
    };
    overlap_order(suggestions::candidates(&state.library, graph, &tonight))
}

/// A kept photo learns which ingredients the cook confirmed for it.
async fn confirm_scan(state: &AppState, user_id: Uuid, request: &SuggestionsRequest) -> Result<(), AppError> {
    let Some(scan_id) = request.scan_id else {
        return Ok(());
    };
    let ingredients: Vec<SavedId> = request.ingredients.iter().cloned().collect();
    let confirmation = Confirmation { user_id, scan_id, ingredients: &ingredients };
    scans::confirm(&state.db, &confirmation).await
}

/// What the cook has tonight: what they confirmed, and their staples less any they leave out. Until they answer
/// the cupboard question the staples a kitchen is taken to have stand in for theirs.
fn available(state: &AppState, request: &SuggestionsRequest, profile: &Profile) -> BTreeSet<SavedId> {
    let usual =
        if profile.cupboard_answered { profile.cupboard_staples.clone() } else { state.library.staples.assumed() };
    let at_home = usual.into_iter().filter(|staple| !request.staples_left_out.contains(staple));
    request.ingredients.iter().cloned().chain(at_home).collect()
}

/// How the answer was made, and for whom.
struct Answer<'a> {
    profile: &'a Profile,
    language: &'a Language,
}

fn response(graph: &Graph, order: Vec<Candidate>, answer: Answer) -> SuggestionsResponse {
    let context = SuggestionContext { graph, profile: answer.profile, language: answer.language };
    let to_suggestion = |candidate: Candidate| Suggestion::new(candidate, None, &context);
    let mut suggestions = order.into_iter().filter_map(to_suggestion).take(MAX_SUGGESTIONS);
    SuggestionsResponse {
        graph_version: graph.version.clone(),
        graph_channel: graph.channel.as_str(),
        ranked_by: "overlap",
        special: suggestions.next(),
        alternatives: suggestions.collect(),
    }
}
