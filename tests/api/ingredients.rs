use crate::helpers::*;
use axum::http::{Method, StatusCode};
use axum::Router;
use serde_json::{json, Value};
use sqlx::PgPool;

async fn resolve(app: &Router, token: Option<&str>, labels: Value) -> Reply {
    let body = Some(("application/json", json!({ "labels": labels }).to_string().into_bytes()));
    call(app, Call { method: Method::POST, path: "/v1/ingredients/resolve", token, body }).await
}

#[sqlx::test(migrations = "./migrations")]
async fn typed_items_become_ingredients_or_choices(pool: PgPool) {
    let (app, cook) = (server(pool), new_cook());
    let reply = resolve(&app, Some(&cook), json!(["Eggs", "oil", "dragon fruit"])).await;
    assert_eq!(reply.status, StatusCode::OK);
    let results = &reply.body["results"];
    assert_eq!(results[0], json!({"label": "Eggs", "basis": "exact", "candidates": [{"id": "egg", "name": "Egg"}]}));
    assert_eq!(results[1]["basis"], "ambiguous");
    assert_eq!(results[2], json!({"label": "dragon fruit", "basis": "none", "candidates": []}));
}

#[sqlx::test(migrations = "./migrations")]
async fn resolving_needs_sign_in_and_sane_input(pool: PgPool) {
    let app = server(pool);
    assert_eq!(resolve(&app, None, json!(["egg"])).await.status, StatusCode::UNAUTHORIZED);
    let cook = new_cook();
    let too_many = Value::from(vec!["egg"; 51]);
    assert_eq!(resolve(&app, Some(&cook), too_many).await.status, StatusCode::UNPROCESSABLE_ENTITY);
    let too_long = json!(["x".repeat(81)]);
    assert_eq!(resolve(&app, Some(&cook), too_long).await.status, StatusCode::UNPROCESSABLE_ENTITY);
}
