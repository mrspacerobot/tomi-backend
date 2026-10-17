//! Plans a graph recipe for tonight using the graph's safe choices.

use super::{AppliedSwap, Candidate, Tonight};
use crate::graph::serving::{CookFacts, RecipePlan, ServedRecipe};
use crate::graph::Graph;
use crate::kernel::ids::SavedId;
use crate::library::LocalizedText;
use serde_json::Value;

#[cfg(test)]
mod tests;

pub const INGREDIENT_PREFIX: &str = "ingredient/";

/// The facts the graph checks and the rest of tonight's data.
pub struct Kitchen<'a> {
    pub graph: &'a Graph,
    pub facts: &'a CookFacts,
    pub tonight: &'a Tonight<'a>,
}

pub struct RecipeSource {
    pub id: String,
    pub value: Value,
    pub title: LocalizedText,
    pub skill: SavedId,
    pub level: u8,
}

/// A graph recipe with its safe plan, or `None` when a required line has no safe way.
pub fn cook(source: &RecipeSource, kitchen: &Kitchen) -> Option<Candidate> {
    let plan = kitchen.graph.serving.plan(&source.id, kitchen.facts).ok()??;
    let recipe = kitchen.graph.serving.recipes.get(&source.id)?;
    let planned = Planned::read(recipe, &plan);
    Some(planned.into_candidate(source, kitchen.tonight))
}

#[derive(Default)]
struct Planned {
    ingredients: Vec<String>,
    swaps: Vec<AppliedSwap>,
    omitted: Vec<String>,
}

impl Planned {
    fn read(recipe: &ServedRecipe, plan: &RecipePlan) -> Planned {
        let mut planned = Planned::default();
        for line in &recipe.lines {
            if plan.left_out.contains(&line.part) {
                planned.omitted.push(line.part.clone());
                continue;
            }
            match plan.swaps.iter().find(|swap| swap.original == line.thing) {
                Some(swap) => planned.swap(&line.thing, swap),
                None => planned.add(&line.thing),
            }
        }
        planned
    }

    fn add(&mut self, thing: &str) {
        self.ingredients.push(thing.to_string());
    }

    fn swap(&mut self, original: &str, swap: &crate::graph::serving::PlannedSwap) {
        self.swaps.push(AppliedSwap {
            removed: original.to_string(),
            added: swap.instead.clone(),
            ratio: swap.ratio.clone(),
            caveat: swap.caveat.clone(),
        });
        self.add(&swap.instead)
    }

    fn into_candidate(self, source: &RecipeSource, tonight: &Tonight) -> Candidate {
        let missing = self.missing(tonight);
        let equipment = equipment(&source.value);
        Candidate {
            recipe_id: source.id.clone(),
            source_recipe: source.value.clone(),
            title: source.title.clone(),
            skill: source.skill.clone(),
            level: source.level,
            missing,
            ingredients: self.ingredients,
            swaps: self.swaps,
            omitted: self.omitted,
            equipment,
        }
    }

    fn missing(&self, tonight: &Tonight) -> Vec<String> {
        let available: std::collections::BTreeSet<String> =
            tonight.available.iter().map(|id| format!("{INGREDIENT_PREFIX}{id}")).collect();
        self.ingredients.iter().filter(|id| !available.contains(*id)).cloned().collect()
    }
}

fn equipment(source: &Value) -> Vec<String> {
    source
        .get("equipment")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(str::to_string)
        .collect()
}
