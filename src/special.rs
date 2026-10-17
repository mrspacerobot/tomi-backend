//! Graph recipes a cook may make tonight with their ingredients and restrictions.
//!
//! Code decides what may be offered; a model may only reorder that list.
//! The steps follow Tomi's plan: filter by open skill level, cuisine,
//! kitchen, diet and avoid list, apply only graph-listed swaps, check
//! the final dish again, then rank.

use crate::graph::serving::CookFacts;
use crate::graph::Graph;
use crate::kernel::ids::SavedId;
use crate::library::{EquipmentList, Library, LocalizedText, Restrictions};
use crate::profile::Profile;
use crate::special::progress::OpenLevels;
use serde::Serialize;
use serde_json::Value;
use std::collections::BTreeSet;

pub mod game;
mod offer;
mod plan;
pub mod progress;
mod ranking;
mod routes;
#[cfg(test)]
mod tests;

pub use routes::router;

pub use offer::{Offer, OfferContext};
pub use ranking::{overlap_order, ranked, Ranked, MODEL_SHORT_LIST};

/// A graph recipe that the cook may make tonight.
#[derive(Debug, Clone)]
pub struct Candidate {
    pub recipe_id: String,
    pub source_recipe: Value,
    pub title: LocalizedText,
    pub skill: SavedId,
    pub level: u8,
    /// The graph's food things after its allowed swaps and omissions.
    pub ingredients: Vec<String>,
    pub swaps: Vec<AppliedSwap>,
    pub omitted: Vec<String>,
    pub missing: Vec<String>,
    pub equipment: Vec<String>,
}

/// A listed swap applied tonight, named as in the app's `AppliedSwap`.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AppliedSwap {
    pub removed: String,
    pub added: String,
    pub ratio: Option<crate::graph::serving::Ratio>,
    pub caveat: Option<String>,
}

/// What the cook brings tonight.
pub struct Tonight<'a> {
    pub profile: &'a Profile,
    /// Confirmed from the scan or typed, plus the cupboard staples.
    pub available: BTreeSet<SavedId>,
    /// The skill levels the cook has reached; nothing locked is offered.
    pub open: &'a OpenLevels,
}

/// Every graph recipe this cook may be offered tonight.
pub fn candidates(library: &Library, graph: &Graph, tonight: &Tonight) -> Vec<Candidate> {
    let Ok(avoided) = library.restrictions.facts_to_avoid(&restrictions(tonight.profile)) else {
        return Vec::new();
    };
    let facts = CookFacts { avoided, at_hand: graph_ids(&tonight.available) };
    let kitchen = plan::Kitchen { graph, facts: &facts, tonight };
    let found =
        graph.serving.recipes.iter().filter_map(|(recipe_id, recipe)| candidate((recipe_id, recipe), graph, &kitchen));
    found
        .filter(|candidate| {
            fits_cuisine(graph, candidate, tonight.profile)
                && has_tracked_equipment(&library.equipment, candidate, tonight.profile)
        })
        .collect()
}

fn candidate(
    recipe: (&str, &crate::graph::serving::ServedRecipe),
    graph: &Graph,
    kitchen: &plan::Kitchen,
) -> Option<Candidate> {
    let (recipe_id, recipe) = recipe;
    let (skill, level) = eligible_level(graph, recipe, kitchen.tonight)?;
    let source = graph.thing(recipe_id)?;
    let title = serde_json::from_value::<LocalizedText>(source.get("names")?.clone()).ok()?;
    let source = plan::RecipeSource { id: recipe_id.to_string(), value: source.clone(), title, skill, level };
    plan::cook(&source, kitchen)
}

fn eligible_level(
    graph: &Graph,
    recipe: &crate::graph::serving::ServedRecipe,
    tonight: &Tonight,
) -> Option<(SavedId, u8)> {
    graph.program.levels_for(&recipe.techniques).into_iter().find_map(|(skill, level)| {
        let skill = SavedId::parse(&skill).ok()?;
        tonight.open.contains(&skill, level).then_some((skill, level))
    })
}

fn has_tracked_equipment(equipment: &EquipmentList, candidate: &Candidate, profile: &Profile) -> bool {
    let available = profile.equipment.as_ref().unwrap_or(&equipment.standard_kitchen);
    candidate
        .equipment
        .iter()
        .filter_map(|id| id.strip_prefix("equipment/"))
        .filter_map(|id| SavedId::parse(id).ok())
        .filter(|id| equipment.knows(id))
        .all(|id| available.contains(&id))
}

/// What the cook has, named as the graph names it.
fn graph_ids(available: &BTreeSet<SavedId>) -> BTreeSet<String> {
    available.iter().map(|id| format!("{}{id}", plan::INGREDIENT_PREFIX)).collect()
}

fn fits_cuisine(graph: &Graph, candidate: &Candidate, profile: &Profile) -> bool {
    let Some(makes) = candidate.source_recipe.get("makes").and_then(Value::as_str) else {
        return false;
    };
    let Some(cuisines) = graph.thing(makes).and_then(|dish| dish.get("belongs_to")).and_then(Value::as_array) else {
        return true;
    };
    !cuisines
        .iter()
        .filter_map(|link| link.get("thing").and_then(Value::as_str))
        .filter_map(|id| id.strip_prefix("cuisine/"))
        .filter_map(|id| SavedId::parse(id).ok())
        .any(|id| profile.cuisines_never.contains(&id))
}

/// The cook's restrictions as the ingredient table reads them.
pub fn restrictions(profile: &Profile) -> Restrictions<'_> {
    Restrictions { diet: profile.diet.as_ref(), avoid: &profile.avoid }
}
