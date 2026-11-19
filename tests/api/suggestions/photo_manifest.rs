//! Test data for a photo of the original chicken dish.

use super::photos::{FILE, RECIPE};
use crate::helpers::PROJECT;
use serde_json::json;
use tomi_api::dish_photos::{fingerprint, DishPhotos, Manifest};
use tomi_api::graph::Graph;

pub fn dish_photos(graph: &Graph, status: &str, stale: bool) -> DishPhotos {
    let hash = if stale { "b".repeat(64) } else { fingerprint(graph.thing(RECIPE).unwrap()) };
    let record = json!({
        "recipe": RECIPE, "shows": ["ingredient/chicken-thigh", "ingredient/white-wine", "ingredient/spinach"],
        "recipeFingerprint": hash, "file": FILE, "model": "test", "prompt": "test",
        "status": status, "approvedBy": "owner", "approvedOn": "2026-11-19"
    });
    let manifest = Manifest::parse(&json!([record]).to_string()).unwrap();
    manifest.for_serving(graph, PROJECT)
}
