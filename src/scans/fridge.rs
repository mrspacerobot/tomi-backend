//! What a model says it saw in a fridge, checked before anyone trusts it.
//!
//! The model may only name ingredients the graph knows. Anything else stays
//! a plain label for the cook to confirm, type differently, or skip.

use crate::graph::Foods;
use crate::kernel::display::{Named, Namer};
use crate::kernel::ids::SavedId;
use serde::{Deserialize, Serialize};

#[cfg(test)]
mod tests;

/// More than any fridge holds; a longer answer is cut, not trusted.
pub const MAX_ITEMS: usize = 60;
const MAX_LABEL_CHARACTERS: usize = 80;

/// One item exactly as the model returned it.
#[derive(Debug, Clone, Deserialize)]
pub struct ModelItem {
    pub label: String,
    pub ingredient: Option<String>,
    pub confidence: f64,
}

/// One item after checking, as the app receives it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SeenItem {
    pub label: String,
    /// An ingredient the graph knows, or none.
    pub ingredient: Option<SavedId>,
    /// From 0 to 1.
    pub confidence: f64,
}

/// One item as the app shows it, with the ingredient's name.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ShownItem {
    pub label: String,
    pub ingredient: Option<Named>,
    pub confidence: f64,
}

impl SeenItem {
    pub fn shown(&self, namer: &Namer) -> ShownItem {
        let ingredient = self.ingredient.as_ref().map(|id| namer.ingredient(id));
        ShownItem { label: self.label.clone(), ingredient, confidence: self.confidence }
    }
}

pub fn checked(items: Vec<ModelItem>, foods: &Foods) -> Vec<SeenItem> {
    let mut seen = Vec::new();
    for item in items.into_iter().take(MAX_ITEMS) {
        let label: String = item.label.trim().chars().take(MAX_LABEL_CHARACTERS).collect();
        if label.is_empty() {
            continue;
        }
        let ingredient = known_ingredient(item.ingredient.as_deref(), foods);
        seen.push(SeenItem { label, ingredient, confidence: clamped(item.confidence) });
    }
    seen
}

fn known_ingredient(raw: Option<&str>, foods: &Foods) -> Option<SavedId> {
    let id = SavedId::parse(raw?).ok()?;
    if foods.knows(&id) {
        return Some(id);
    }
    None
}

fn clamped(confidence: f64) -> f64 {
    if confidence.is_nan() {
        return 0.0;
    }
    confidence.clamp(0.0, 1.0)
}
