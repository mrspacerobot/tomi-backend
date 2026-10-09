//! The ingredient table: what each ingredient contains, and which facts
//! each avoid item and each diet rules out.
//!
//! This is the only source of allergen and diet answers. Anything it does
//! not list refuses, never passes.

use super::{Choice, LocalizedText};
use crate::domain::ids::SavedId;
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};

mod validation;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct TableFile {
    schema_version: i64,
    facts: BTreeSet<String>,
    avoid_items: Vec<AvoidItem>,
    diets: Vec<Diet>,
    ingredients: Vec<Ingredient>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct AvoidItem {
    id: SavedId,
    name: LocalizedText,
    triggered_by: BTreeSet<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Diet {
    id: SavedId,
    name: LocalizedText,
    summary: LocalizedText,
    excludes: BTreeSet<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Ingredient {
    id: SavedId,
    name: LocalizedText,
    contains: BTreeSet<String>,
}

pub struct IngredientTable {
    /// What the app shows, in the file's order.
    pub listed: Listed,
    names: BTreeMap<SavedId, String>,
    contains: BTreeMap<SavedId, BTreeSet<String>>,
    avoid_items: BTreeMap<SavedId, BTreeSet<String>>,
    diets: BTreeMap<SavedId, BTreeSet<String>>,
}

/// The diets, avoid items and ingredients with their names, for display.
pub struct Listed {
    pub diets: Vec<Choice>,
    pub avoid_items: Vec<Choice>,
    pub ingredients: Vec<Choice>,
}

/// What the cook may not eat, from their profile.
pub struct Restrictions<'a> {
    pub diet: Option<&'a SavedId>,
    pub avoid: &'a BTreeSet<SavedId>,
}

/// Why an ingredient may not be offered.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Refusal {
    UnknownIngredient(SavedId),
    UnknownDiet(SavedId),
    UnknownAvoidItem(SavedId),
    RuledOutByDiet { ingredient: SavedId, diet: SavedId },
    Avoided { ingredient: SavedId, item: SavedId },
}

const SCHEMA_VERSION: i64 = 1;

impl IngredientTable {
    pub fn parse(text: &str) -> Result<Self, String> {
        let file: TableFile = serde_json::from_str(text).map_err(|error| error.to_string())?;
        if file.schema_version != SCHEMA_VERSION {
            return Err(format!("schema version {} is not {SCHEMA_VERSION}", file.schema_version));
        }
        validation::check_declared(&file)?;
        Ok(IngredientTable::from(file))
    }

    pub fn knows(&self, ingredient: &SavedId) -> bool {
        self.contains.contains_key(ingredient)
    }

    /// Every ingredient id with its English name, in id order.
    pub fn names(&self) -> &BTreeMap<SavedId, String> {
        &self.names
    }

    /// Whether this cook may be offered this ingredient.
    pub fn check(&self, ingredient: &SavedId, cook: &Restrictions) -> Result<(), Refusal> {
        let Some(facts) = self.contains.get(ingredient) else {
            return Err(Refusal::UnknownIngredient(ingredient.clone()));
        };
        let food = Food { ingredient, facts };
        if let Some(diet) = cook.diet {
            self.check_diet(&food, diet)?;
        }
        for item in cook.avoid {
            self.check_avoid_item(&food, item)?;
        }
        Ok(())
    }

    fn check_diet(&self, food: &Food, diet: &SavedId) -> Result<(), Refusal> {
        let Some(excluded) = self.diets.get(diet) else {
            return Err(Refusal::UnknownDiet(diet.clone()));
        };
        if food.facts.is_disjoint(excluded) {
            return Ok(());
        }
        Err(Refusal::RuledOutByDiet { ingredient: food.ingredient.clone(), diet: diet.clone() })
    }

    fn check_avoid_item(&self, food: &Food, item: &SavedId) -> Result<(), Refusal> {
        let Some(triggers) = self.avoid_items.get(item) else {
            return Err(Refusal::UnknownAvoidItem(item.clone()));
        };
        if food.facts.is_disjoint(triggers) {
            return Ok(());
        }
        Err(Refusal::Avoided { ingredient: food.ingredient.clone(), item: item.clone() })
    }
}

/// An ingredient with the facts the table lists for it.
struct Food<'a> {
    ingredient: &'a SavedId,
    facts: &'a BTreeSet<String>,
}

impl From<TableFile> for IngredientTable {
    fn from(file: TableFile) -> Self {
        IngredientTable {
            listed: Listed::from(&file),
            names: file.ingredients.iter().map(|item| (item.id.clone(), item.name.english().to_string())).collect(),
            contains: file.ingredients.into_iter().map(|item| (item.id, item.contains)).collect(),
            avoid_items: file.avoid_items.into_iter().map(|item| (item.id, item.triggered_by)).collect(),
            diets: file.diets.into_iter().map(|diet| (diet.id, diet.excludes)).collect(),
        }
    }
}

impl From<&TableFile> for Listed {
    fn from(file: &TableFile) -> Self {
        let diets = file.diets.iter().map(|diet| Choice::described(&diet.id, &diet.name, &diet.summary));
        let avoid_items = file.avoid_items.iter().map(|item| Choice::named(&item.id, &item.name));
        let ingredients = file.ingredients.iter().map(|item| Choice::named(&item.id, &item.name));
        Listed { diets: diets.collect(), avoid_items: avoid_items.collect(), ingredients: ingredients.collect() }
    }
}

impl Listed {
    pub fn ingredient(&self, id: &SavedId) -> Option<&Choice> {
        self.ingredients.iter().find(|choice| &choice.id == id)
    }
}
