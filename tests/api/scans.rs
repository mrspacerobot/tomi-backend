use crate::fakes::FakeStore;
use crate::helpers::*;
use axum::http::StatusCode;
use serde_json::{json, Value};
use sqlx::PgPool;
use std::sync::Arc;
use tomi_api::ai::NoAi;
use tomi_api::storage::NoStore;
use uuid::Uuid;

#[sqlx::test(migrations = "./migrations")]
async fn the_consent_routes_are_gone_until_consent_is_built_properly(pool: PgPool) {
    let (app, cook) = (server(pool), new_cook());
    assert_eq!(get(&app, "/v1/consents", Some(&cook)).await.status, StatusCode::NOT_FOUND);
    let put = Call { method: axum::http::Method::PUT, path: "/v1/consents/photo-scan", token: Some(&cook), body: None };
    assert_eq!(call(&app, put).await.status, StatusCode::NOT_FOUND);
}

#[sqlx::test(migrations = "./migrations")]
async fn a_scan_returns_known_ingredients_and_plain_labels(pool: PgPool) {
    let (app, cook) = (server(pool), new_cook());
    let reply = scan(&app, &cook, jpeg()).await;
    assert_eq!(reply.status, StatusCode::OK, "{}", reply.body);
    let items = &reply.body["items"];
    assert_eq!(items[0], json!({"label": "eggs", "ingredient": {"id": "egg", "name": "Egg"}, "confidence": 0.9}));
    assert_eq!(items[1]["ingredient"], Value::Null);
}

#[sqlx::test(migrations = "./migrations")]
async fn every_scanned_photo_is_kept_as_it_arrived(pool: PgPool) {
    let store = Arc::new(FakeStore::default());
    let app = server_with(pool.clone(), Setup { photos: store.clone(), ..Setup::default() });
    let subject = Uuid::new_v4();
    let cook = token_with(&claims(subject));
    let upload = jpeg();
    let sent = upload.1.clone();
    let reply = scan(&app, &cook, upload).await;
    assert_eq!(reply.body["kept"], true);

    let user = user_id(&pool, subject).await;
    let files = store.files.lock().unwrap().clone();
    let (path, bytes) = files.iter().next().unwrap();
    assert_eq!(path, &format!("{user}/{}.jpg", reply.body["scanId"].as_str().unwrap()));
    assert_eq!(bytes, &sent);
    let seen: Value = sqlx::query_scalar("select seen from tomi.fridge_photos").fetch_one(&pool).await.unwrap();
    assert_eq!(seen[0]["ingredient"], "egg");
}

#[sqlx::test(migrations = "./migrations")]
async fn without_photo_storage_the_scan_still_works_and_keeps_nothing(pool: PgPool) {
    let app = server_with(pool.clone(), Setup { photos: Arc::new(NoStore), ..Setup::default() });
    let reply = scan(&app, &new_cook(), jpeg()).await;
    assert_eq!(reply.status, StatusCode::OK);
    assert_eq!(reply.body["kept"], false);
    let rows: i64 = sqlx::query_scalar("select count(*) from tomi.fridge_photos").fetch_one(&pool).await.unwrap();
    assert_eq!(rows, 0);
}

#[sqlx::test(migrations = "./migrations")]
async fn a_real_sized_photo_is_accepted(pool: PgPool) {
    let mut photo = jpeg_with_location();
    photo.extend(vec![0x11; 1024 * 1024]);
    let reply = scan(&server(pool), &new_cook(), ("image/jpeg", photo)).await;
    assert_eq!(reply.status, StatusCode::OK, "{}", reply.body);
}

#[sqlx::test(migrations = "./migrations")]
async fn photos_of_other_types_or_sizes_are_refused(pool: PgPool) {
    let (app, cook) = (server(pool), new_cook());
    let heic = scan(&app, &cook, ("image/heic", jpeg_with_location())).await;
    assert_eq!(heic.status, StatusCode::UNSUPPORTED_MEDIA_TYPE);
    let huge = scan(&app, &cook, ("image/jpeg", vec![0xFF; 11 * 1024 * 1024])).await;
    assert_eq!(huge.status, StatusCode::PAYLOAD_TOO_LARGE);
    let fake = scan(&app, &cook, ("image/jpeg", b"not a photo".to_vec())).await;
    assert_eq!(fake.status, StatusCode::UNPROCESSABLE_ENTITY);
}

#[sqlx::test(migrations = "./migrations")]
async fn without_a_model_the_scan_says_so(pool: PgPool) {
    let app = server_with(pool, Setup { ai: Arc::new(NoAi), ..Setup::default() });
    let cook = new_cook();
    let reply = scan(&app, &cook, jpeg()).await;
    assert_eq!(reply.status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(reply.body["error"]["code"], "ai_unavailable");
}

#[sqlx::test(migrations = "./migrations")]
async fn without_a_graph_a_scan_says_so_and_never_guesses(pool: PgPool) {
    let app = server_with(pool, Setup { graph: None, ..Setup::default() });
    let reply = scan(&app, &new_cook(), jpeg()).await;
    assert_eq!(reply.status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(reply.body["error"]["code"], "graph_unavailable");
}
