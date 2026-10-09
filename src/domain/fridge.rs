//! What a model says it saw in a fridge, checked before anyone trusts it.
//!
//! The model may only name ingredients from the table. Anything else stays
//! a plain label for the cook to confirm, type differently, or skip.

use crate::domain::ids::SavedId;
use crate::domain::library::IngredientTable;
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
    /// An ingredient the table knows, or none.
    pub ingredient: Option<SavedId>,
    /// From 0 to 1.
    pub confidence: f64,
}

pub fn checked(items: Vec<ModelItem>, table: &IngredientTable) -> Vec<SeenItem> {
    let mut seen = Vec::new();
    for item in items.into_iter().take(MAX_ITEMS) {
        let label: String = item.label.trim().chars().take(MAX_LABEL_CHARACTERS).collect();
        if label.is_empty() {
            continue;
        }
        let ingredient = known_ingredient(item.ingredient.as_deref(), table);
        seen.push(SeenItem { label, ingredient, confidence: clamped(item.confidence) });
    }
    seen
}

fn known_ingredient(raw: Option<&str>, table: &IngredientTable) -> Option<SavedId> {
    let id = SavedId::parse(raw?).ok()?;
    if table.knows(&id) {
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
