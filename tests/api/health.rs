use crate::helpers::{get, server};
use axum::http::StatusCode;
use serde_json::Value;
use sqlx::PgPool;

#[sqlx::test(migrations = "./migrations")]
async fn health_checks_need_no_sign_in(pool: PgPool) {
    let app = server(pool);
    assert_eq!(get(&app, "/healthz", None).await.status, StatusCode::OK);
    assert_eq!(get(&app, "/readyz", None).await.status, StatusCode::OK);
}

#[sqlx::test(migrations = "./migrations")]
async fn bootstrap_offers_no_content_until_some_is_published(pool: PgPool) {
    let reply = get(&server(pool), "/v1/bootstrap", None).await;
    assert_eq!(reply.status, StatusCode::OK);
    assert_eq!(reply.body["minAppVersion"], "1.2.0");
    assert_eq!(reply.body["libraryVersion"], Value::Null);
    assert_eq!(reply.body["safetyVersion"], Value::Null);
}

#[sqlx::test(migrations = "./migrations")]
async fn every_reply_carries_a_request_id(pool: PgPool) {
    let reply = get(&server(pool), "/v1/bootstrap", None).await;
    assert!(reply.headers.contains_key("x-request-id"));
}
