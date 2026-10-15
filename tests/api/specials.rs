use crate::fakes::{FakeAi, FakeStore};
use crate::helpers::*;
use axum::http::StatusCode;
use axum::Router;
use serde_json::{json, Value};
use sqlx::PgPool;
use std::sync::Arc;
use tomi_api::kernel::ids::SavedId;
use tomi_api::special::Ranked;

const OMELETTE: &str = "spinach-hard-cheese-omelette";
const PAN_SAUCE: &str = "seared-chicken-lemon-butter-pan-sauce";

/// A server where a comfortable cook can reach every lesson, with this model.
fn open_server(pool: PgPool, ai: FakeAi) -> Router {
    server_with(pool, Setup { ai: Arc::new(ai), library: library_without_prerequisites(), ..Setup::default() })
}

fn ranking_omelette_first() -> FakeAi {
    let reason = Some("Ten minutes and you have the eggs.".to_string());
    let ranked = Ranked { lesson_id: SavedId::parse(OMELETTE).unwrap(), reason };
    FakeAi { ranking: Some(vec![ranked]), ..FakeAi::default() }
}

/// A new cook who answered "comfortable", with these other answers.
async fn comfortable_cook(app: &Router, answers: Value) -> String {
    let cook = new_cook();
    let mut profile = json!({"version": 0, "experience": "comfortable"});
    profile.as_object_mut().unwrap().extend(answers.as_object().unwrap().clone());
    assert_eq!(patch_profile(app, &cook, profile).await.status, StatusCode::OK);
    cook
}

#[sqlx::test(migrations = "./migrations")]
async fn a_new_cook_is_offered_nothing_above_their_level(pool: PgPool) {
    let app = server(pool);
    let reply = special(&app, &new_cook(), json!({"ingredients": ["egg"]})).await;
    assert_eq!(reply.body["special"], Value::Null, "the only lessons are eggs 3 and pan sauce 1");
    let comfortable = comfortable_cook(&app, json!({})).await;
    let reply = special(&app, &comfortable, json!({"ingredients": ["egg"]})).await;
    assert_eq!(reply.body["special"]["lessonId"], OMELETTE);
    assert_eq!(reply.body["alternatives"], json!([]), "pan sauce needs searing first");
}

#[sqlx::test(migrations = "./migrations")]
async fn an_offer_carries_the_words_the_app_shows(pool: PgPool) {
    let app = open_server(pool, FakeAi::default());
    let cook = comfortable_cook(&app, json!({"diet": "vegetarian", "avoid": ["nuts"]})).await;
    let reply = special(&app, &cook, json!({"ingredients": ["egg"]})).await;
    let offer = &reply.body["special"];
    assert_eq!(offer["title"], "Spinach and hard cheese omelette");
    assert_eq!(offer["skill"], json!({"id": "eggs", "name": "Eggs"}));
    assert_eq!(offer["level"], json!({"number": 3, "name": "Rolled omelette"}));
    assert_eq!(offer["ingredients"][0], json!({"id": "egg", "name": "Egg"}));
    assert_eq!(offer["checked"], json!({"diet": "vegetarian", "avoid": ["nuts"]}));
}

#[sqlx::test(migrations = "./migrations")]
async fn the_model_orders_the_short_list(pool: PgPool) {
    let app = open_server(pool, ranking_omelette_first());
    let cook = comfortable_cook(&app, json!({})).await;
    let reply = special(&app, &cook, json!({"ingredients": ["chicken-thigh"]})).await;
    assert_eq!(reply.body["rankedBy"], "model");
    assert_eq!(reply.body["special"]["lessonId"], OMELETTE);
    assert_eq!(reply.body["special"]["reason"], "Ten minutes and you have the eggs.");
    assert_eq!(reply.body["alternatives"][0]["lessonId"], PAN_SAUCE);
}

#[sqlx::test(migrations = "./migrations")]
async fn without_a_model_the_order_follows_what_the_cook_has(pool: PgPool) {
    let app = open_server(pool, FakeAi::default());
    let cook = comfortable_cook(&app, json!({})).await;
    let reply = special(&app, &cook, json!({"ingredients": ["chicken-thigh", "garlic"]})).await;
    assert_eq!(reply.body["rankedBy"], "overlap");
    assert_eq!(reply.body["special"]["lessonId"], PAN_SAUCE);
}

#[sqlx::test(migrations = "./migrations")]
async fn a_listed_swap_is_applied_and_named(pool: PgPool) {
    let app = open_server(pool, FakeAi::default());
    let cook = comfortable_cook(&app, json!({"avoid": ["alcohol"]})).await;
    let at_home = json!({"ingredients": ["chicken-thigh", "garlic", "lemon", "sunflower-oil"]});
    let reply = special(&app, &cook, at_home).await;
    assert_eq!(reply.body["special"]["lessonId"], PAN_SAUCE);
    let swap = json!({"removed": {"id": "white-wine", "name": "White wine"}, "added": {"id": "chicken-stock", "name": "Chicken stock"}});
    assert_eq!(reply.body["special"]["swaps"], json!([swap]));
}

#[sqlx::test(migrations = "./migrations")]
async fn an_allergen_only_the_graph_knows_is_honoured(pool: PgPool) {
    // Sulphites had no avoid item before the graph added one: the wine is out, the stock stands in.
    let app = open_server(pool, FakeAi::default());
    let cook = comfortable_cook(&app, json!({"avoid": ["sulphites"]})).await;
    let at_home = json!({"ingredients": ["chicken-thigh", "garlic", "lemon", "sunflower-oil"]});
    let reply = special(&app, &cook, at_home).await;
    assert_eq!(reply.body["special"]["lessonId"], PAN_SAUCE);
    assert_eq!(reply.body["special"]["swaps"][0]["added"]["id"], "chicken-stock");
    assert_eq!(reply.body["special"]["checked"]["avoid"], json!(["sulphites"]));
}

#[sqlx::test(migrations = "./migrations")]
async fn two_graph_allergens_together_leave_no_safe_way(pool: PgPool) {
    // Wine holds sulphites and every stock holds celery, so a cook avoiding both gets no pan sauce.
    let app = open_server(pool, FakeAi::default());
    let cook = comfortable_cook(&app, json!({"avoid": ["sulphites", "celery"]})).await;
    let reply = special(&app, &cook, json!({"ingredients": ["chicken-thigh"]})).await;
    assert_ne!(reply.body["special"]["lessonId"], PAN_SAUCE);
    assert!(reply.body["alternatives"].as_array().unwrap().iter().all(|offer| offer["lessonId"] != PAN_SAUCE));
}

#[sqlx::test(migrations = "./migrations")]
async fn saved_nuts_still_rule_out_peanuts_and_tree_nuts_together(pool: PgPool) {
    let app = open_server(pool, FakeAi::default());
    let cook = comfortable_cook(&app, json!({"avoid": ["nuts"]})).await;
    let reply = special(&app, &cook, json!({"ingredients": ["egg"]})).await;
    assert_eq!(reply.body["special"]["checked"]["avoid"], json!(["nuts"]));
}

#[sqlx::test(migrations = "./migrations")]
async fn a_vegetarian_never_gets_the_chicken(pool: PgPool) {
    let app = open_server(pool, FakeAi::default());
    let cook = comfortable_cook(&app, json!({"diet": "vegetarian"})).await;
    let reply = special(&app, &cook, json!({"ingredients": ["chicken-thigh"]})).await;
    assert_eq!(reply.body["special"]["lessonId"], OMELETTE);
    assert_eq!(reply.body["alternatives"], json!([]));
}

#[sqlx::test(migrations = "./migrations")]
async fn an_unknown_restriction_offers_nothing(pool: PgPool) {
    let app = open_server(pool, FakeAi::default());
    let cook = comfortable_cook(&app, json!({"avoid": ["mango"]})).await;
    let reply = special(&app, &cook, json!({"ingredients": ["egg"]})).await;
    assert_eq!(reply.body["special"], Value::Null);
}

#[sqlx::test(migrations = "./migrations")]
async fn a_scanned_photo_learns_what_the_cook_confirmed(pool: PgPool) {
    let store = Arc::new(FakeStore::default());
    let app = server_with(pool.clone(), Setup { photos: store, ..Setup::default() });
    let cook = new_cook();
    let scanned = scan(&app, &cook, jpeg()).await;
    let body = json!({"ingredients": ["egg", "spinach"], "scanId": scanned.body["scanId"]});
    special(&app, &cook, body).await;
    let confirmed: Value =
        sqlx::query_scalar("select confirmed from tomi.fridge_photos").fetch_one(&pool).await.unwrap();
    assert_eq!(confirmed, json!(["egg", "spinach"]));
}

#[sqlx::test(migrations = "./migrations")]
async fn the_old_specials_path_is_gone(pool: PgPool) {
    let app = open_server(pool, FakeAi::default());
    let token = new_cook();
    let body = Some(("application/json", json!({ "ingredients": ["egg"] }).to_string().into_bytes()));
    let reply =
        call(&app, Call { method: axum::http::Method::POST, path: "/v1/specials", token: Some(&token), body }).await;
    assert_eq!(reply.status, StatusCode::NOT_FOUND);
}

#[sqlx::test(migrations = "./migrations")]
async fn without_a_graph_a_suggestion_says_so_and_offers_nothing(pool: PgPool) {
    let app = server_with(pool, Setup { graph: None, ..Setup::default() });
    let reply = special(&app, &new_cook(), json!({"ingredients": ["egg"]})).await;
    assert_eq!(reply.status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(reply.body["error"]["code"], "graph_unavailable");
}
