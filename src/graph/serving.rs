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
    pub step_text: StepText,
    pub recipes: BTreeMap<String, ServedRecipe>,
}

/// What writing a step needs besides the foods.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StepText {
    /// Words that keep their capital inside a sentence, such as Dijon.
    pub proper_name_words: Vec<String>,
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
    pub ratio: Option<SwapRatio>,
    pub caveat: Option<String>,
    pub tier: String,
}

/// How much of the swap replaces how much of the original, by weight, volume or count.
#[derive(Debug, Clone, PartialEq, Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct SwapRatio {
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
    pub part: String,
    pub original: String,
    pub instead: String,
    pub ratio: Option<SwapRatio>,
    pub caveat: Option<String>,
}

/// How a recipe is made for one cook: the swaps to apply and the optional parts to leave out.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct RecipePlan {
    pub swaps: Vec<PlannedSwap>,
    pub left_out: Vec<String>,
}

/// What a cook avoids, in graph facts, and which foods (as graph ids) they have tonight.
#[derive(Debug, Clone, Default)]
pub struct CookFacts {
    pub avoided: BTreeSet<String>,
    pub at_hand: BTreeSet<String>,
}

enum Decision<'a> {
    Keep,
    Swap(&'a ServedSwap),
    LeaveOut,
    Refuse,
}

impl Serving {
    /// The recipe made for this cook, or `None` when a required line has no safe way.
    pub fn plan(&self, recipe: &str, cook: &CookFacts) -> Result<Option<RecipePlan>, ServingError> {
        if let Some(unknown) = cook.avoided.iter().find(|fact| !self.facts.contains(*fact)) {
            return Err(ServingError::UnknownFact(unknown.clone()));
        }
        let Some(served) = self.recipes.get(recipe) else {
            return Err(ServingError::UnknownRecipe(recipe.to_string()));
        };
        Ok(served.plan(cook))
    }
}

impl ServedRecipe {
    fn plan(&self, cook: &CookFacts) -> Option<RecipePlan> {
        let mut plan = RecipePlan::default();
        for line in &self.lines {
            match line.decide(cook) {
                Decision::Keep => {}
                Decision::Swap(swap) => plan.swaps.push(swap.replacing(line)),
                Decision::LeaveOut => plan.left_out.push(line.part.clone()),
                Decision::Refuse => return None,
            }
        }
        plan.left_out.dedup();
        Some(plan)
    }
}

impl ServedLine {
    /// A safe original the cook has is kept. Otherwise a safe swap, the one they have first. A safe original
    /// they lack is replaced only by a swap they have.
    fn decide(&self, cook: &CookFacts) -> Decision<'_> {
        let safe = self.unsafe_for.is_disjoint(&cook.avoided);
        if safe && cook.at_hand.contains(&self.thing) {
            return Decision::Keep;
        }
        match self.best_swap(cook, safe) {
            Some(swap) => Decision::Swap(swap),
            None if safe => Decision::Keep,
            None if self.optional => Decision::LeaveOut,
            None => Decision::Refuse,
        }
    }

    fn best_swap(&self, cook: &CookFacts, original_is_safe: bool) -> Option<&ServedSwap> {
        let mut safe = self.swaps.iter().filter(|swap| swap.unsafe_for.is_disjoint(&cook.avoided));
        let at_hand = safe.clone().find(|swap| cook.at_hand.contains(&swap.thing));
        if original_is_safe {
            return at_hand;
        }
        at_hand.or_else(|| safe.next())
    }
}

impl ServedSwap {
    fn replacing(&self, line: &ServedLine) -> PlannedSwap {
        PlannedSwap {
            part: line.part.clone(),
            original: line.thing.clone(),
            instead: self.thing.clone(),
            ratio: self.ratio.clone(),
            caveat: self.caveat.clone(),
        }
    }
}
