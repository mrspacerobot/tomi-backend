//! Plans a graph recipe for tonight using the graph's safe choices, then adapts it for the cook.

use super::adapted::{adapt, Adapted, Basis};
use super::{AppliedSwap, Candidate, Tonight};
use crate::graph::serving::{CookFacts, PlannedSwap};
use crate::graph::Graph;
use crate::kernel::ids::SavedId;
use crate::library::{EquipmentList, LocalizedText};
use serde_json::Value;

#[cfg(test)]
mod tests;

pub const INGREDIENT_PREFIX: &str = "ingredient/";

/// The facts the graph checks and the rest of tonight's data.
pub struct Kitchen<'a> {
    pub graph: &'a Graph,
    pub facts: &'a CookFacts,
    pub tonight: &'a Tonight<'a>,
    pub equipment: &'a EquipmentList,
}

pub struct RecipeSource {
    pub id: String,
    pub value: Value,
    pub title: LocalizedText,
    pub skill: SavedId,
    pub level: u8,
}

/// A graph recipe with its safe plan, adapted for the cook. `None` when a required line has no safe way, or when
/// the recipe cannot be adapted cleanly: a recipe that cannot be written for the cook is not offered.
pub fn cook(source: &RecipeSource, kitchen: &Kitchen) -> Option<Candidate> {
    let plan = kitchen.graph.serving.plan(&source.id, kitchen.facts).ok()??;
    let basis = Basis { graph: kitchen.graph, avoided: &kitchen.facts.avoided };
    match adapt(&source.id, &plan, &basis) {
        Ok(adapted) => Some(into_candidate(source, adapted, kitchen.tonight)),
        Err(error) => {
            log_not_adapted(&source.id, error.reason());
            None
        }
    }
}

/// Only the recipe and why: never a food, a restriction or the cook.
#[expect(clippy::cognitive_complexity, reason = "tracing's macro expands into branches no reader sees")]
fn log_not_adapted(recipe: &str, reason: &str) {
    tracing::warn!(recipe, reason, "a recipe could not be adapted and is not offered");
}

fn into_candidate(source: &RecipeSource, adapted: Adapted, tonight: &Tonight) -> Candidate {
    let ingredients = food_ids(&adapted);
    Candidate {
        recipe_id: source.id.clone(),
        source_recipe: source.value.clone(),
        title: source.title.clone(),
        skill: source.skill.clone(),
        level: source.level,
        missing: missing(&ingredients, tonight),
        swaps: adapted.swaps.iter().map(applied).collect(),
        omitted: adapted.left_out.iter().map(|left_out| left_out.part.clone()).collect(),
        equipment: equipment(&source.value),
        ingredients,
        adapted,
    }
}

fn food_ids(adapted: &Adapted) -> Vec<String> {
    adapted.lines.iter().map(|line| line.food.clone()).collect()
}

fn applied(swap: &PlannedSwap) -> AppliedSwap {
    AppliedSwap {
        removed: swap.original.clone(),
        added: swap.instead.clone(),
        ratio: swap.ratio.clone(),
        caveat: swap.caveat.clone(),
    }
}

fn missing(ingredients: &[String], tonight: &Tonight) -> Vec<String> {
    let available: std::collections::BTreeSet<String> =
        tonight.available.iter().map(|id| format!("{INGREDIENT_PREFIX}{id}")).collect();
    ingredients.iter().filter(|id| !available.contains(*id)).cloned().collect()
}

pub fn equipment(source: &Value) -> Vec<String> {
    source
        .get("equipment")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(str::to_string)
        .collect()
}
