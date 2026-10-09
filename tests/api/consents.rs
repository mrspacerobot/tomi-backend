use crate::fakes::FakeStore;
use crate::helpers::*;
use axum::http::{Method, StatusCode};
use serde_json::json;
use sqlx::PgPool;
use std::sync::Arc;
use tomi_api::storage::NoStore;
use uuid::Uuid;

#[sqlx::test(migrations = "./migrations")]
async fn a_new_cook_has_agreed_to_nothing(pool: PgPool) {
    let reply = get(&server(pool), "/v1/consents", Some(&new_cook())).await;
    assert_eq!(reply.status, StatusCode::OK);
    let expected = json!([
        {"purpose": "photo-scan", "currentVersion": 1, "agreed": false},
        {"purpose": "photo-dataset", "currentVersion": 1, "agreed": false},
    ]);
    assert_eq!(reply.body, expected);
}

#[sqlx::test(migrations = "./migrations")]
async fn agreeing_to_the_current_text_is_recorded(pool: PgPool) {
    let (app, cook) = (server(pool), new_cook());
    assert_eq!(agree(&app, &cook, "photo-scan").await.status, StatusCode::NO_CONTENT);
    let reply = get(&app, "/v1/consents", Some(&cook)).await;
    assert_eq!(reply.body[0]["agreed"], true);
    assert_eq!(reply.body[1]["agreed"], false);
}

#[sqlx::test(migrations = "./migrations")]
async fn an_old_text_or_an_unknown_purpose_is_refused(pool: PgPool) {
    let (app, cook) = (server(pool), new_cook());
    let old = Some(("application/json", br#"{"version": 0}"#.to_vec()));
    let call_old = Call { method: Method::PUT, path: "/v1/consents/photo-scan", token: Some(&cook), body: old };
    assert_eq!(call(&app, call_old).await.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(agree(&app, &cook, "selling-my-soul").await.status, StatusCode::NOT_FOUND);
}

#[sqlx::test(migrations = "./migrations")]
async fn the_dataset_cannot_be_joined_without_photo_storage(pool: PgPool) {
    let app = server_with(pool, Setup { photos: Arc::new(NoStore), ..Setup::default() });
    let reply = agree(&app, &new_cook(), "photo-dataset").await;
    assert_eq!(reply.status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(reply.body["error"]["code"], "storage_unavailable");
}

#[sqlx::test(migrations = "./migrations")]
async fn leaving_the_dataset_deletes_every_kept_photo(pool: PgPool) {
    let store = Arc::new(FakeStore::default());
    let app = server_with(pool.clone(), Setup { photos: store.clone(), ..Setup::default() });
    let subject = Uuid::new_v4();
    let cook = token_with(&claims(subject));
    agree(&app, &cook, "photo-scan").await;
    agree(&app, &cook, "photo-dataset").await;
    scan(&app, &cook, jpeg()).await;
    assert_eq!(store.files.lock().unwrap().len(), 1);

    let leave = Call { method: Method::DELETE, path: "/v1/consents/photo-dataset", token: Some(&cook), body: None };
    assert_eq!(call(&app, leave).await.status, StatusCode::NO_CONTENT);
    assert!(store.files.lock().unwrap().is_empty());
    let rows = "select count(*) from tomi.fridge_photos where user_id = $1";
    let user = user_id(&pool, subject).await;
    let count: i64 = sqlx::query_scalar(rows).bind(user).fetch_one(&pool).await.unwrap();
    assert_eq!(count, 0);
}
