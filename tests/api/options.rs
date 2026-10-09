use crate::helpers::*;
use axum::http::{Method, StatusCode};
use sqlx::PgPool;

#[sqlx::test(migrations = "./migrations")]
async fn the_lists_need_no_sign_in_and_come_with_words(pool: PgPool) {
    let reply = get(&server(pool), "/v1/options", None).await;
    assert_eq!(reply.status, StatusCode::OK);
    assert_eq!(reply.body["avoidItems"][0]["name"], "Nuts");
    assert_eq!(reply.body["courses"][0]["skills"][0]["name"], "Knife cuts");
}

#[sqlx::test(migrations = "./migrations")]
async fn an_unknown_language_gets_english(pool: PgPool) {
    let app = server(pool);
    let request = Call { method: Method::GET, path: "/v1/options", token: None, body: None };
    let reply = call_in(&app, request, "ja-JP,ja;q=0.9").await;
    assert_eq!(reply.body["diets"][0]["name"], "Vegetarian");
}
