//! Validate pinned graph references and classify photo work.

use super::*;
use crate::config::GraphConfig;
use crate::graph::files;
use serde_json::json;

const DISH: &str = "recipe/spinach-and-cheese-omelette";
const FOOD: &str = "ingredient/egg";
const ORPHAN: &str = "recipe/missing";

fn fixture() -> Graph {
    let config = GraphConfig { dir: Some("tests/fixtures/graph".into()), allow_draft: true, required: true };
    let lock = include_str!("../../tests/fixtures/graph/graph.lock");
    files::start(lock, &config).unwrap().unwrap()
}

fn record(recipe: &str, food: &str, hash: &str) -> Value {
    let file = format!("{}.webp", "b".repeat(64));
    json!({
        "recipe": recipe, "shows": [food], "recipeFingerprint": hash, "file": file,
        "status": "generated", "model": "test", "prompt": "test"
    })
}

#[test]
fn a_recipe_or_food_missing_from_the_graph_fails_validation() {
    let graph = fixture();
    let hash = "a".repeat(64);
    let orphan = Manifest::parse(&json!([record(ORPHAN, FOOD, &hash)]).to_string()).unwrap();
    assert!(matches!(orphan.validate(&graph), Err(ManifestError::UnknownRecipe(_))));
    let unknown = Manifest::parse(&json!([record(DISH, "ingredient/missing", &hash)]).to_string()).unwrap();
    assert!(matches!(unknown.validate(&graph), Err(ManifestError::UnknownFood { .. })));
}

#[test]
fn the_plan_classifies_missing_stale_and_orphaned_records() {
    let graph = fixture();
    let hash = "a".repeat(64);
    let input = json!([record(DISH, FOOD, &hash), record(ORPHAN, FOOD, &hash)]);
    let manifest = Manifest::parse(&input.to_string()).unwrap();
    let plan = serde_json::to_value(manifest.plan(&graph)).unwrap();
    assert_eq!(plan["stale"][0]["recipe"], DISH);
    assert_eq!(plan["orphaned"], json!([ORPHAN]));
    let missing = plan["missing"].as_array().unwrap();
    assert!(!missing.is_empty());
    assert!(missing.iter().all(|dish| dish["recipe"] != DISH && dish["recipe"] != ORPHAN));
}

#[test]
fn an_unreadable_or_ambiguous_record_is_refused() {
    let hash = "a".repeat(64);
    let mut photo = record(DISH, FOOD, &hash);
    assert!(Manifest::parse(&json!([photo, photo]).to_string()).is_err());
    photo["status"] = json!("approved");
    assert!(Manifest::parse(&json!([photo]).to_string()).is_err());
    photo["file"] = json!("../image.webp");
    assert!(Manifest::parse(&json!([photo]).to_string()).is_err());
}
