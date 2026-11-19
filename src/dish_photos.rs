//! Matches each published dish photo to the graph and the cook's adapted recipe.

use crate::graph::{sha256_hex, Graph};
use crate::recipe::PreparedRecipe;
use std::collections::BTreeMap;

mod db;
pub use db::published;

use db::PublishedPhoto;

#[derive(Default)]
pub struct DishPhotos(BTreeMap<String, CheckedPhoto>);

struct CheckedPhoto {
    url: String,
    shows: Vec<String>,
}

impl DishPhotos {
    pub fn checked(rows: Vec<PublishedPhoto>, graph: &Graph, project_url: &str) -> Self {
        let root = project_url.trim_end_matches('/');
        let photos = rows.into_iter().filter_map(|row| row.check(graph, root));
        Self(photos.collect())
    }

    pub fn url(&self, recipe: &str, adapted: &PreparedRecipe) -> Option<String> {
        let photo = self.0.get(recipe)?;
        photo.shows.iter().all(|food| adapted.lines.iter().any(|line| &line.food == food)).then(|| photo.url.clone())
    }
}

impl PublishedPhoto {
    fn check(self, graph: &Graph, root: &str) -> Option<(String, CheckedPhoto)> {
        let source = graph.thing(&self.recipe_id)?;
        if self.recipe_fingerprint != sha256_hex(&source.to_string()) || self.shows.is_empty() {
            return None;
        }
        if !self.shows.iter().all(|food| known_food(graph, food)) || !hashed_webp(&self.object_name) {
            return None;
        }
        let url = format!("{root}/storage/v1/object/public/dish-photos/{}", self.object_name);
        Some((self.recipe_id, CheckedPhoto { url, shows: self.shows }))
    }
}

fn known_food(graph: &Graph, id: &str) -> bool {
    let is_food = id.starts_with("ingredient/") || id.starts_with("preparation/");
    is_food && graph.thing(id).is_some()
}

fn hashed_webp(name: &str) -> bool {
    name.strip_suffix(".webp")
        .is_some_and(|hash| hash.len() == 64 && hash.bytes().all(|byte| matches!(byte, b'0'..=b'9' | b'a'..=b'f')))
}
