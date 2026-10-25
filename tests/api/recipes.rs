use crate::helpers::*;
use crate::suggestions::{comfortable_cook, open_server};
use axum::http::{Method, StatusCode};
use axum::Router;
use serde_json::{json, Value};
use sqlx::PgPool;

#[sqlx::test(migrations = "./migrations")]
async fn a_graph_recipe_is_offered_without_a_copied_lesson(pool: PgPool) {
    let app = open_server(pool);
    let cook = comfortable_cook(&app, json!({})).await;
    let reply = suggest(&app, &cook, json!({"ingredients": ["egg"]})).await;
    let offer = &reply.body["special"];
    assert_eq!(offer["recipeId"], "recipe/spinach-and-cheese-omelette");
    assert!(!offer.as_object().unwrap().contains_key("lessonId"));
    assert!(!offer.as_object().unwrap().contains_key("minutes"));
    assert!(offer["sourceRecipe"]["steps"].is_array());
    assert_source_recipe_has_amounts(offer);
    assert_eq!(reply.body["graphVersion"], "fixture");
    assert_eq!(reply.body["graphChannel"], "draft");
}

fn assert_source_recipe_has_amounts(offer: &Value) {
    let ingredients = offer["sourceRecipe"]["ingredients"].as_array().unwrap();
    assert!(ingredients.iter().any(|line| line["amount"].is_object()));
    assert!(ingredients.iter().all(|line| line["thing"].is_string()));
}

#[sqlx::test(migrations = "./migrations")]
async fn an_unknown_ingredient_is_refused_before_a_scan_is_confirmed(pool: PgPool) {
    let app = server(pool.clone());
    let cook = new_cook();
    let scanned = scan(&app, &cook, jpeg()).await;
    let body = json!({"ingredients": ["made-up"], "scanId": scanned.body["scanId"]});
    let reply = suggest(&app, &cook, body).await;
    assert_eq!(reply.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(reply.body["error"]["code"], "invalid_request");
    let confirmed: Option<Value> =
        sqlx::query_scalar("select confirmed from tomi.fridge_photos").fetch_one(&pool).await.unwrap();
    assert!(confirmed.is_none());
}

#[sqlx::test(migrations = "./migrations")]
async fn the_old_special_alias_is_gone(pool: PgPool) {
    let app = open_server(pool);
    let token = new_cook();
    assert_path_is_gone(&app, &token, "/v1/specials").await;
}

async fn assert_path_is_gone(app: &Router, token: &str, path: &str) {
    let body = Some(("application/json", json!({"ingredients": ["egg"]}).to_string().into_bytes()));
    let reply = call(app, Call { method: Method::POST, path, token: Some(token), body }).await;
    assert_eq!(reply.status, StatusCode::NOT_FOUND);
}
