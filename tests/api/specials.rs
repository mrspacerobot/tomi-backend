use crate::fakes::{FakeAi, FakeStore};
use crate::helpers::*;
use axum::http::StatusCode;
use serde_json::{json, Value};
use sqlx::PgPool;
use std::sync::Arc;
use tomi_api::domain::ids::SavedId;
use tomi_api::domain::special::Ranked;

const OMELETTE: &str = "spinach-hard-cheese-omelette";
const PAN_SAUCE: &str = "seared-chicken-lemon-butter-pan-sauce";

fn ranking_omelette_first() -> Setup {
    let reason = Some("Ten minutes and you have the eggs.".to_string());
    let ranked = Ranked { lesson_id: SavedId::parse(OMELETTE).unwrap(), reason };
    let ai = FakeAi { ranking: Some(vec![ranked]), ..FakeAi::default() };
    Setup { ai: Arc::new(ai), ..Setup::default() }
}

#[sqlx::test(migrations = "./migrations")]
async fn a_vegetarian_gets_only_the_omelette(pool: PgPool) {
    let (app, cook) = (server(pool), new_cook());
    patch_profile(&app, &cook, json!({"version": 0, "diet": "vegetarian"})).await;
    let reply = special(&app, &cook, json!({"ingredients": ["egg"]})).await;
    assert_eq!(reply.status, StatusCode::OK, "{}", reply.body);
    assert_eq!(reply.body["special"]["lessonId"], OMELETTE);
    assert_eq!(reply.body["alternatives"], json!([]));
    assert_eq!(reply.body["rankedBy"], "overlap");
}

#[sqlx::test(migrations = "./migrations")]
async fn the_model_orders_the_short_list(pool: PgPool) {
    let app = server_with(pool, ranking_omelette_first());
    let reply = special(&app, &new_cook(), json!({"ingredients": ["chicken-thigh"]})).await;
    assert_eq!(reply.body["rankedBy"], "model");
    assert_eq!(reply.body["special"]["lessonId"], OMELETTE);
    assert_eq!(reply.body["special"]["reason"], "Ten minutes and you have the eggs.");
    assert_eq!(reply.body["alternatives"][0]["lessonId"], PAN_SAUCE);
}

#[sqlx::test(migrations = "./migrations")]
async fn without_a_model_the_order_follows_what_the_cook_has(pool: PgPool) {
    let reply = special(&server(pool), &new_cook(), json!({"ingredients": ["chicken-thigh", "garlic"]})).await;
    assert_eq!(reply.body["rankedBy"], "overlap");
    assert_eq!(reply.body["special"]["lessonId"], PAN_SAUCE);
}

#[sqlx::test(migrations = "./migrations")]
async fn a_listed_swap_is_applied_and_reported(pool: PgPool) {
    let (app, cook) = (server(pool), new_cook());
    patch_profile(&app, &cook, json!({"version": 0, "avoid": ["alcohol"], "diet": "pescatarian"})).await;
    let reply = special(&app, &cook, json!({"ingredients": []})).await;
    assert_eq!(reply.body["special"]["lessonId"], OMELETTE);
    patch_profile(&app, &cook, json!({"version": 1, "diet": null, "allowWeakerRestrictions": true})).await;
    let at_home = json!({"ingredients": ["chicken-thigh", "garlic", "lemon", "sunflower-oil"]});
    let reply = special(&app, &cook, at_home).await;
    assert_eq!(reply.body["special"]["lessonId"], PAN_SAUCE);
    let swaps = &reply.body["special"]["swaps"];
    assert_eq!(swaps, &json!([{"replace": "white-wine", "with": "chicken-stock"}]));
}

#[sqlx::test(migrations = "./migrations")]
async fn an_unknown_restriction_offers_nothing(pool: PgPool) {
    let (app, cook) = (server(pool), new_cook());
    patch_profile(&app, &cook, json!({"version": 0, "avoid": ["mango"]})).await;
    let reply = special(&app, &cook, json!({"ingredients": ["egg"]})).await;
    assert_eq!(reply.body["special"], Value::Null);
}

#[sqlx::test(migrations = "./migrations")]
async fn a_kept_photo_learns_what_the_cook_confirmed(pool: PgPool) {
    let store = Arc::new(FakeStore::default());
    let app = server_with(pool.clone(), Setup { photos: store, ..Setup::default() });
    let cook = new_cook();
    agree(&app, &cook, "photo-scan").await;
    agree(&app, &cook, "photo-dataset").await;
    let scanned = scan(&app, &cook, jpeg()).await;
    let body = json!({"ingredients": ["egg", "spinach"], "scanId": scanned.body["scanId"]});
    special(&app, &cook, body).await;
    let confirmed: Value =
        sqlx::query_scalar("select confirmed from tomi.fridge_photos").fetch_one(&pool).await.unwrap();
    assert_eq!(confirmed, json!(["egg", "spinach"]));
}
