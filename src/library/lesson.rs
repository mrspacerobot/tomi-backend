//! What the server needs to know about a lesson to offer it.
//!
//! Only the fields the server decides with are read; the app's validator
//! owns the full lesson schema. Its skill, level, cuisine and equipment must be in
//! the lists, or the lesson is left out. Its foods are checked against the graph.

use super::{Library, LocalizedText};
use crate::kernel::ids::SavedId;
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
}

#[derive(Debug, Clone, Deserialize)]
pub struct LessonIngredient {
    pub ingredient: SavedId,
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

    fn check_against(&self, library: &Library) -> Result<(), String> {
        if !library.catalog.has_level(&self.skill, self.level) {
            return Err(format!("skill `{}` has no level {}", self.skill, self.level));
        }
        if !library.choices.has_cuisine(&self.cuisine) {
            return Err(format!("unknown cuisine `{}`", self.cuisine));
        }
        self.check_ingredients()?;
        match self.equipment.iter().find(|item| !library.equipment.knows(item)) {
            Some(item) => Err(format!("unknown equipment `{item}`")),
            None => Ok(()),
        }
    }

    fn check_ingredients(&self) -> Result<(), String> {
        let mut seen = BTreeSet::new();
        for id in self.ingredient_ids() {
            if !seen.insert(id) {
                return Err(format!("ingredient `{id}` appears twice"));
            }
        }
        if seen.is_empty() {
            return Err("the lesson has no ingredients".to_string());
        }
        Ok(())
    }
}
