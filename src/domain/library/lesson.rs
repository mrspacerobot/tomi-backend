//! What the server needs to know about a lesson to offer it.
//!
//! Only the fields the server decides with are read; the app's validator
//! owns the full lesson schema. Every id a lesson names must be in the
//! lists, or the lesson is left out.

use super::{Library, LocalizedText};
use crate::domain::ids::SavedId;
use serde::Deserialize;
use std::collections::BTreeSet;

const SCHEMA_VERSION: i64 = 1;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Lesson {
    schema_version: i64,
    pub id: SavedId,
    pub revision: u32,
    pub skill: SavedId,
    pub level: u8,
    pub cuisine: SavedId,
    pub title: LocalizedText,
    pub minutes: u32,
    pub ingredients: Vec<LessonIngredient>,
    pub equipment: Vec<SavedId>,
    pub swaps: Vec<Swap>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LessonIngredient {
    pub ingredient: SavedId,
}

/// A swap the lesson lists: `replace` may give way to `with`.
#[derive(Debug, Clone, Deserialize)]
pub struct Swap {
    pub replace: SavedId,
    pub with: LessonIngredient,
}

impl Lesson {
    pub fn parse(text: &str, library: &Library) -> Result<Self, String> {
        let lesson: Lesson = serde_json::from_str(text).map_err(|error| error.to_string())?;
        if lesson.schema_version != SCHEMA_VERSION {
            return Err(format!("schema version {} is not {SCHEMA_VERSION}", lesson.schema_version));
        }
        lesson.check_against(library)?;
        Ok(lesson)
    }

    pub fn ingredient_ids(&self) -> impl Iterator<Item = &SavedId> {
        self.ingredients.iter().map(|item| &item.ingredient)
    }

    /// The listed swaps for one of the lesson's ingredients.
    pub fn swaps_for<'a>(&'a self, ingredient: &'a SavedId) -> impl Iterator<Item = &'a SavedId> {
        let matching = self.swaps.iter().filter(move |swap| &swap.replace == ingredient);
        matching.map(|swap| &swap.with.ingredient)
    }

    fn check_against(&self, library: &Library) -> Result<(), String> {
        if !library.catalog.has_level(&self.skill, self.level) {
            return Err(format!("skill `{}` has no level {}", self.skill, self.level));
        }
        self.check_ingredients(library)?;
        self.check_swaps(library)?;
        match self.equipment.iter().find(|item| !library.equipment.knows(item)) {
            Some(item) => Err(format!("unknown equipment `{item}`")),
            None => Ok(()),
        }
    }

    fn check_ingredients(&self, library: &Library) -> Result<(), String> {
        let mut seen = BTreeSet::new();
        for id in self.ingredient_ids() {
            if !library.table.knows(id) {
                return Err(format!("unknown ingredient `{id}`"));
            }
            if !seen.insert(id) {
                return Err(format!("ingredient `{id}` appears twice"));
            }
        }
        if seen.is_empty() {
            return Err("the lesson has no ingredients".to_string());
        }
        Ok(())
    }

    /// A swap replaces something the lesson has, with something known that
    /// the lesson does not already have.
    fn check_swaps(&self, library: &Library) -> Result<(), String> {
        let own: BTreeSet<&SavedId> = self.ingredient_ids().collect();
        for swap in &self.swaps {
            let with = &swap.with.ingredient;
            let fits = own.contains(&swap.replace) && !own.contains(with) && library.table.knows(with);
            if !fits {
                return Err(format!("swap of `{}` for `{with}` does not fit", swap.replace));
            }
        }
        Ok(())
    }
}
