use crate::fakes::FakeStore;
use crate::helpers::*;
use axum::http::StatusCode;
use axum::Router;
use serde_json::{json, Value};
use sqlx::PgPool;
use std::sync::Arc;

pub const OMELETTE: &str = "recipe/spinach-and-cheese-omelette";
const PAN_SAUCE: &str = "recipe/seared-chicken-thighs-with-lemon-butter-pan-sauce";

/// A server where a comfortable cook can reach every skill level.
pub fn open_server(pool: PgPool) -> Router {
    server_with(pool, Setup { library: library_without_prerequisites(), ..Setup::default() })
}

/// A new cook who answered "comfortable", with these other answers.
pub async fn comfortable_cook(app: &Router, answers: Value) -> String {
    let cook = new_cook();
    let mut profile = json!({"version": 0, "experience": "comfortable"});
    profile.as_object_mut().unwrap().extend(answers.as_object().unwrap().clone());
    assert_eq!(patch_profile(app, &cook, profile).await.status, StatusCode::OK);
    cook
}

#[sqlx::test(migrations = "./migrations")]
async fn a_new_cook_is_offered_nothing_above_their_level(pool: PgPool) {
    let app = server(pool);
    let reply = suggest(&app, &new_cook(), json!({"ingredients": ["egg"]})).await;
    assert_eq!(reply.body["special"]["level"]["number"], 1);
    let comfortable = comfortable_cook(&app, json!({})).await;
    let reply = suggest(&app, &comfortable, json!({"ingredients": ["egg"]})).await;
    assert!(offers(&reply.body).iter().any(|offer| offer["recipeId"] == OMELETTE));
}

#[sqlx::test(migrations = "./migrations")]
async fn an_offer_carries_the_words_the_app_shows(pool: PgPool) {
    let app = open_server(pool);
    let cook = comfortable_cook(&app, json!({"diet": "vegetarian", "avoid": ["nuts"]})).await;
    let reply = suggest(&app, &cook, json!({"ingredients": ["egg"]})).await;
    let offer = &reply.body["special"];
    assert_eq!(offer["title"], "Spinach and cheese omelette");
    assert_eq!(offer["skill"], json!({"id": "eggs", "name": "Eggs"}));
    assert_eq!(offer["level"], json!({"number": 3, "name": "Rolled omelette"}));
    assert_eq!(offer["safePlan"]["ingredients"][0], json!({"id": "ingredient/egg", "name": "Egg"}));
    assert_eq!(offer["checked"], json!({"diet": "vegetarian", "avoid": ["nuts"]}));
}

#[sqlx::test(migrations = "./migrations")]
async fn recommendations_use_the_deterministic_overlap_order(pool: PgPool) {
    let app = open_server(pool);
    let cook = comfortable_cook(&app, json!({})).await;
    let pan_sauce = ["chicken-thigh", "garlic", "lemon", "white-wine", "butter", "spinach"];
    let reply = suggest(&app, &cook, json!({"ingredients": pan_sauce})).await;
    assert_eq!(reply.body["rankedBy"], "overlap");
    assert_eq!(reply.body["special"]["recipeId"], PAN_SAUCE);
}

#[sqlx::test(migrations = "./migrations")]
async fn a_listed_swap_is_applied_and_named(pool: PgPool) {
    let app = open_server(pool);
    let cook = comfortable_cook(&app, json!({"avoid": ["alcohol"]})).await;
    let at_home = json!({"ingredients": ["chicken-thigh", "garlic", "lemon", "sunflower-oil"]});
    let reply = suggest(&app, &cook, at_home).await;
    assert_eq!(reply.body["special"]["recipeId"], PAN_SAUCE);
    let swap = json!({"removed": {"id": "ingredient/white-wine", "name": "White wine"}, "added": {"id": "ingredient/chicken-stock", "name": "Chicken stock"}, "ratio": {"by": "volume", "original": 1.0, "stand_in": 1.0}, "caveat": "Stock has none of the sharpness of wine. A squeeze of lemon at the end brings some of it back."});
    assert_eq!(reply.body["special"]["safePlan"]["substitutions"], json!([swap]));
    assert!(reply.body["special"]["sourceRecipe"]["ingredients"]
        .as_array()
        .unwrap()
        .iter()
        .any(|line| line["thing"] == "ingredient/white-wine"));
    assert!(reply.body["special"]["safePlan"]["ingredients"]
        .as_array()
        .unwrap()
        .iter()
        .any(|line| line["id"] == "ingredient/chicken-stock"));
    assert!(!reply.body["special"]["safePlan"]["ingredients"]
        .as_array()
        .unwrap()
        .iter()
        .any(|line| line["id"] == "ingredient/white-wine"));
}

#[sqlx::test(migrations = "./migrations")]
async fn an_allergen_only_the_graph_knows_is_honoured(pool: PgPool) {
    // Sulphites had no avoid item before the graph added one: the wine is out, the stock stands in.
    let app = open_server(pool);
    let cook = comfortable_cook(&app, json!({"avoid": ["sulphites"]})).await;
    let at_home = json!({"ingredients": ["chicken-thigh", "garlic", "lemon", "sunflower-oil"]});
    let reply = suggest(&app, &cook, at_home).await;
    assert_eq!(reply.body["special"]["recipeId"], PAN_SAUCE);
    assert_eq!(reply.body["special"]["safePlan"]["substitutions"][0]["added"]["id"], "ingredient/chicken-stock");
    assert_eq!(reply.body["special"]["checked"]["avoid"], json!(["sulphites"]));
}

#[sqlx::test(migrations = "./migrations")]
async fn two_graph_allergens_together_leave_no_safe_way(pool: PgPool) {
    // Wine holds sulphites and every stock holds celery, so a cook avoiding both gets no pan sauce.
    let app = open_server(pool);
    let cook = comfortable_cook(&app, json!({"avoid": ["sulphites", "celery"]})).await;
    let reply = suggest(&app, &cook, json!({"ingredients": ["chicken-thigh"]})).await;
    assert_ne!(reply.body["special"]["recipeId"], PAN_SAUCE);
    assert!(reply.body["alternatives"].as_array().unwrap().iter().all(|offer| offer["recipeId"] != PAN_SAUCE));
}

#[sqlx::test(migrations = "./migrations")]
async fn saved_nuts_still_rule_out_peanuts_and_tree_nuts_together(pool: PgPool) {
    let app = open_server(pool);
    let cook = comfortable_cook(&app, json!({"avoid": ["nuts"]})).await;
    let reply = suggest(&app, &cook, json!({"ingredients": ["egg"]})).await;
    assert_eq!(reply.body["special"]["checked"]["avoid"], json!(["nuts"]));
}

#[sqlx::test(migrations = "./migrations")]
async fn a_vegetarian_never_gets_the_chicken(pool: PgPool) {
    let app = open_server(pool);
    let cook = comfortable_cook(&app, json!({"diet": "vegetarian"})).await;
    let reply = suggest(&app, &cook, json!({"ingredients": ["chicken-thigh"]})).await;
    assert!(offers(&reply.body).iter().all(|offer| offer["recipeId"] != PAN_SAUCE));
    assert!(offers(&reply.body).iter().any(|offer| offer["recipeId"] == OMELETTE));
}

fn offers(response: &Value) -> Vec<&Value> {
    response["alternatives"]
        .as_array()
        .into_iter()
        .flatten()
        .chain(response["special"].as_object().map(|_| &response["special"]))
        .collect()
}

#[sqlx::test(migrations = "./migrations")]
async fn an_unknown_restriction_offers_nothing(pool: PgPool) {
    let app = open_server(pool);
    let cook = comfortable_cook(&app, json!({"avoid": ["mango"]})).await;
    let reply = suggest(&app, &cook, json!({"ingredients": ["egg"]})).await;
    assert_eq!(reply.body["special"], Value::Null);
}

#[sqlx::test(migrations = "./migrations")]
async fn a_scanned_photo_learns_what_the_cook_confirmed(pool: PgPool) {
    let store = Arc::new(FakeStore::default());
    let app = server_with(pool.clone(), Setup { photos: store, ..Setup::default() });
    let cook = new_cook();
    let scanned = scan(&app, &cook, jpeg()).await;
    let body = json!({"ingredients": ["egg", "spinach"], "scanId": scanned.body["scanId"]});
    suggest(&app, &cook, body).await;
    let confirmed: Value =
        sqlx::query_scalar("select confirmed from tomi.fridge_photos").fetch_one(&pool).await.unwrap();
    assert_eq!(confirmed, json!(["egg", "spinach"]));
}

#[sqlx::test(migrations = "./migrations")]
async fn without_a_graph_a_suggestion_says_so_and_offers_nothing(pool: PgPool) {
    let app = server_with(pool, Setup { graph: None, ..Setup::default() });
    let reply = suggest(&app, &new_cook(), json!({"ingredients": ["egg"]})).await;
    assert_eq!(reply.status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(reply.body["error"]["code"], "graph_unavailable");
}
