//! Dish photo eligibility through the authenticated suggestions route.

use super::comfortable_cook;
use crate::helpers::*;
use serde_json::json;
use sqlx::PgPool;
use tomi_api::graph::Graph;

pub(super) const RECIPE: &str = "recipe/seared-chicken-thighs-with-lemon-butter-pan-sauce";
pub(super) const FILE: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa.webp";

#[sqlx::test(migrations = "./migrations")]
async fn a_photo_follows_the_checked_recipe_when_food_is_swapped(pool: PgPool) {
    let app = photo_server(pool, "approved", false);
    let original = comfortable_cook(&app, json!({})).await;
    let swapped = comfortable_cook(&app, json!({"avoid": ["alcohol"]})).await;
    let full = suggest(&app, &original, full_pantry()).await;
    let swap = suggest(&app, &swapped, full_pantry()).await;
    assert_eq!(full.body["special"]["recipeId"], RECIPE);
    assert_eq!(swap.body["special"]["recipeId"], RECIPE);
    assert_eq!(full.body["special"]["photoUrl"], format!("{PROJECT}/storage/v1/object/public/dish-photos/{FILE}"));
    assert_eq!(swap.body["special"]["changes"]["swaps"][0]["removed"]["id"], "ingredient/white-wine");
    assert!(swap.body["special"].get("photoUrl").is_none());
}

#[sqlx::test(migrations = "./migrations")]
async fn a_photo_is_omitted_when_the_checked_recipe_loses_a_part(pool: PgPool) {
    let app = photo_server(pool, "approved", false);
    let cook = comfortable_cook(&app, json!({"avoid": ["egg"]})).await;
    let reply = suggest(&app, &cook, full_pantry()).await;
    let offer = &reply.body["special"];
    assert_eq!(offer["recipeId"], RECIPE);
    assert_eq!(offer["changes"]["leftOut"][0]["ingredients"][0]["id"], "ingredient/spinach");
    assert!(offer.get("photoUrl").is_none());
}

#[sqlx::test(migrations = "./migrations")]
async fn stale_and_unapproved_photos_are_absent(pool: PgPool) {
    for (status, stale) in [("approved", true), ("generated", false), ("rejected", false)] {
        let app = photo_server(pool.clone(), status, stale);
        let cook = comfortable_cook(&app, json!({})).await;
        let reply = suggest(&app, &cook, full_pantry()).await;
        assert_eq!(reply.body["special"]["recipeId"], RECIPE);
        assert!(reply.body["special"].get("photoUrl").is_none(), "{status}, stale: {stale}");
    }
}

fn full_pantry() -> serde_json::Value {
    json!({"ingredients": ["chicken-thigh", "garlic", "lemon", "white-wine", "butter", "spinach"]})
}

fn photo_server(pool: PgPool, status: &str, stale: bool) -> axum::Router {
    let library = library_without_prerequisites();
    let graph = graph_with_an_optional_allergen();
    let dish_photos = super::photo_manifest::dish_photos(&graph, status, stale);
    server_with(pool, Setup { library, dish_photos, graph: Some(graph), ..Setup::default() })
}

/// Give the fixture's optional part a graph allergen and no stand-in so the route must omit it.
fn graph_with_an_optional_allergen() -> Graph {
    let mut graph = fixture_graph();
    let recipe = graph.serving.recipes.get_mut(RECIPE).unwrap();
    let line = recipe.lines.iter_mut().find(|line| line.optional).unwrap();
    line.unsafe_for.insert("fact/egg".into());
    line.swaps.clear();
    graph
}
