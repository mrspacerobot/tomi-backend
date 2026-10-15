//! What the graph says about each recipe for a server: the facts every line and swap is unsafe for.
//!
//! The graph repository works these out (`serving` in the artifact); this side only compares sets.
//! A fact or recipe it does not list is an error, never a pass.

use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Serving {
    pub facts: BTreeSet<String>,
    pub recipes: BTreeMap<String, ServedRecipe>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ServedRecipe {
    pub makes: String,
    pub tier: String,
    pub techniques: Vec<String>,
    pub ingredients: Vec<String>,
    pub equipment: Vec<String>,
    pub unsafe_for: BTreeSet<String>,
    pub optional_parts: Vec<OptionalPart>,
    pub lines: Vec<ServedLine>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OptionalPart {
    pub part: String,
    pub unsafe_for: BTreeSet<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ServedLine {
    pub thing: String,
    pub part: String,
    pub optional: bool,
    pub unsafe_for: BTreeSet<String>,
    pub swaps: Vec<ServedSwap>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ServedSwap {
    pub thing: String,
    pub unsafe_for: BTreeSet<String>,
    pub basis: String,
    pub ratio: Option<Ratio>,
    pub caveat: Option<String>,
    pub tier: String,
}

/// How much of the swap replaces how much of the original, by weight, volume or count.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Ratio {
    pub stand_in: f64,
    pub original: f64,
    pub by: String,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ServingError {
    #[error("the graph does not know the fact `{0}`")]
    UnknownFact(String),
    #[error("the graph does not know the recipe `{0}`")]
    UnknownRecipe(String),
}

/// A swap to apply to one line, as the graph allows it.
#[derive(Debug, Clone, PartialEq)]
pub struct PlannedSwap {
    pub original: String,
    pub instead: String,
    pub ratio: Option<Ratio>,
    pub caveat: Option<String>,
}

/// How a recipe is made for one cook: the swaps to apply and the optional parts to leave out.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct RecipePlan {
    pub swaps: Vec<PlannedSwap>,
    pub left_out: Vec<String>,
}

impl Serving {
    /// The recipe made for a cook who avoids these facts, or `None` when a required line has no safe way.
    pub fn plan(&self, recipe: &str, avoided: &BTreeSet<String>) -> Result<Option<RecipePlan>, ServingError> {
        if let Some(unknown) = avoided.iter().find(|fact| !self.facts.contains(*fact)) {
            return Err(ServingError::UnknownFact(unknown.clone()));
        }
        let Some(served) = self.recipes.get(recipe) else {
            return Err(ServingError::UnknownRecipe(recipe.to_string()));
        };
        Ok(served.plan(avoided))
    }
}

impl ServedRecipe {
    fn plan(&self, avoided: &BTreeSet<String>) -> Option<RecipePlan> {
        let mut plan = RecipePlan::default();
        for line in self.lines.iter().filter(|line| !line.is_safe(avoided)) {
            match line.first_safe_swap(avoided) {
                Some(swap) => plan.swaps.push(swap.replacing(&line.thing)),
                None if line.optional => plan.left_out.push(line.part.clone()),
                None => return None,
            }
        }
        plan.left_out.dedup();
        Some(plan)
    }
}

impl ServedLine {
    fn is_safe(&self, avoided: &BTreeSet<String>) -> bool {
        self.unsafe_for.is_disjoint(avoided)
    }

    fn first_safe_swap(&self, avoided: &BTreeSet<String>) -> Option<&ServedSwap> {
        self.swaps.iter().find(|swap| swap.unsafe_for.is_disjoint(avoided))
    }
}

impl ServedSwap {
    fn replacing(&self, original: &str) -> PlannedSwap {
        PlannedSwap {
            original: original.to_string(),
            instead: self.thing.clone(),
            ratio: self.ratio.clone(),
            caveat: self.caveat.clone(),
        }
    }
}
