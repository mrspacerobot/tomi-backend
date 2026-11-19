//! The reviewed dish photo manifest and matching against a cook's final recipe.

use crate::graph::{sha256_hex, Graph};
use crate::recipe::PreparedRecipe;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

mod plan;
#[cfg(test)]
mod tests;

pub const SHIPPED: &str = include_str!("../dish-photos/manifest.json");

pub struct Manifest(BTreeMap<String, Record>);

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Record {
    recipe: String,
    shows: BTreeSet<String>,
    recipe_fingerprint: String,
    file: String,
    model: String,
    prompt: String,
    status: Status,
    approved_by: Option<String>,
    approved_on: Option<jiff::civil::Date>,
}

#[derive(Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
enum Status {
    Generated,
    Approved,
    Rejected,
}

#[derive(Debug, thiserror::Error)]
pub enum ManifestError {
    #[error("dish photo manifest is unreadable")]
    Unreadable(#[from] serde_json::Error),
    #[error("invalid dish photo record for {0}")]
    Invalid(String),
    #[error("duplicate dish photo record for {0}")]
    Duplicate(String),
    #[error("the graph has no dish recipe {0}")]
    UnknownRecipe(String),
    #[error("dish photo for {recipe} shows unknown graph food {food}")]
    UnknownFood { recipe: String, food: String },
}

#[derive(Default)]
pub struct DishPhotos(BTreeMap<String, PublishedPhoto>);

struct PublishedPhoto {
    url: String,
    shows: BTreeSet<String>,
}

impl Manifest {
    pub fn parse(text: &str) -> Result<Self, ManifestError> {
        let mut by_recipe = BTreeMap::new();
        for record in serde_json::from_str::<Vec<Record>>(text)? {
            let recipe = record.recipe.clone();
            if !record.well_formed() {
                return Err(ManifestError::Invalid(recipe));
            }
            if by_recipe.insert(recipe.clone(), record).is_some() {
                return Err(ManifestError::Duplicate(recipe));
            }
        }
        Ok(Self(by_recipe))
    }

    pub fn validate(&self, graph: &Graph) -> Result<(), ManifestError> {
        self.0.values().try_for_each(|record| record.validate(graph))
    }

    pub fn for_serving(&self, graph: &Graph, project_url: &str) -> DishPhotos {
        let root = project_url.trim_end_matches('/');
        let published = self.0.values().filter_map(|record| record.publish(graph, root));
        DishPhotos(published.collect())
    }
}

impl Record {
    fn well_formed(&self) -> bool {
        !self.shows.is_empty()
            && !self.model.trim().is_empty()
            && !self.prompt.trim().is_empty()
            && is_hash(&self.recipe_fingerprint)
            && self.file.strip_suffix(".webp").is_some_and(is_hash)
            && self.has_review_if_approved()
    }

    fn has_review_if_approved(&self) -> bool {
        self.status != Status::Approved
            || (self.approved_by.as_ref().is_some_and(|name| !name.trim().is_empty()) && self.approved_on.is_some())
    }

    fn validate(&self, graph: &Graph) -> Result<(), ManifestError> {
        if dish_source(graph, &self.recipe).is_none() {
            return Err(ManifestError::UnknownRecipe(self.recipe.clone()));
        }
        for food in &self.shows {
            if !known_food(graph, food) {
                return Err(ManifestError::UnknownFood { recipe: self.recipe.clone(), food: food.clone() });
            }
        }
        Ok(())
    }

    fn publish(&self, graph: &Graph, root: &str) -> Option<(String, PublishedPhoto)> {
        if self.status != Status::Approved || self.validate(graph).is_err() {
            return None;
        }
        let source = dish_source(graph, &self.recipe)?;
        if self.recipe_fingerprint != fingerprint(source) {
            return None;
        }
        let url = format!("{root}/storage/v1/object/public/dish-photos/{}", self.file);
        Some((self.recipe.clone(), PublishedPhoto { url, shows: self.shows.clone() }))
    }
}

impl DishPhotos {
    pub fn url(&self, recipe: &str, adapted: &PreparedRecipe) -> Option<String> {
        let photo = self.0.get(recipe)?;
        let matches = photo.shows.iter().all(|food| adapted.lines.iter().any(|line| &line.food == food));
        matches.then(|| photo.url.clone())
    }
}

fn known_food(graph: &Graph, id: &str) -> bool {
    let is_food = id.starts_with("ingredient/") || id.starts_with("preparation/");
    is_food && graph.thing(id).is_some()
}

fn is_hash(text: &str) -> bool {
    text.len() == 64 && text.bytes().all(|byte| matches!(byte, b'0'..=b'9' | b'a'..=b'f'))
}

fn dish_source<'a>(graph: &'a Graph, recipe: &str) -> Option<&'a Value> {
    let served = graph.serving.recipes.get(recipe)?;
    if !crate::graph::serving::is_dish(&served.makes) {
        return None;
    }
    graph.thing(recipe)
}

/// Hash the compact source recipe JSON, including its ordered ingredient lines and steps.
pub fn fingerprint(source: &Value) -> String {
    sha256_hex(&source.to_string())
}
