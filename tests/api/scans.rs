use crate::fakes::FakeStore;
use crate::helpers::*;
use axum::http::StatusCode;
use serde_json::{json, Value};
use sqlx::PgPool;
use std::sync::Arc;
use tomi_api::ai::NoAi;
use uuid::Uuid;

#[sqlx::test(migrations = "./migrations")]
async fn a_scan_needs_the_scan_consent(pool: PgPool) {
    let reply = scan(&server(pool), &new_cook(), jpeg()).await;
    assert_eq!(reply.status, StatusCode::FORBIDDEN);
    assert_eq!(reply.body["error"]["code"], "consent_required");
    assert_eq!(reply.body["error"]["details"], json!({"purpose": "photo-scan", "version": 1}));
}

#[sqlx::test(migrations = "./migrations")]
async fn a_scan_returns_known_ingredients_and_plain_labels(pool: PgPool) {
    let (app, cook) = (server(pool), new_cook());
    agree(&app, &cook, "photo-scan").await;
    let reply = scan(&app, &cook, jpeg()).await;
    assert_eq!(reply.status, StatusCode::OK, "{}", reply.body);
    let items = &reply.body["items"];
    assert_eq!(items[0], json!({"label": "eggs", "ingredient": {"id": "egg", "name": "Egg"}, "confidence": 0.9}));
    assert_eq!(items[1]["ingredient"], Value::Null);
    assert_eq!(reply.body["kept"], false);
}

#[sqlx::test(migrations = "./migrations")]
async fn without_the_dataset_consent_nothing_is_kept(pool: PgPool) {
    let store = Arc::new(FakeStore::default());
    let app = server_with(pool, Setup { photos: store.clone(), ..Setup::default() });
    let cook = new_cook();
    agree(&app, &cook, "photo-scan").await;
    scan(&app, &cook, jpeg()).await;
    assert!(store.files.lock().unwrap().is_empty());
}

#[sqlx::test(migrations = "./migrations")]
async fn with_the_dataset_consent_the_photo_is_kept_without_its_location(pool: PgPool) {
    let store = Arc::new(FakeStore::default());
    let app = server_with(pool.clone(), Setup { photos: store.clone(), ..Setup::default() });
    let subject = Uuid::new_v4();
    let cook = token_with(&claims(subject));
    agree(&app, &cook, "photo-scan").await;
    agree(&app, &cook, "photo-dataset").await;
    let reply = scan(&app, &cook, jpeg()).await;
    assert_eq!(reply.body["kept"], true);

    let user = user_id(&pool, subject).await;
    let files = store.files.lock().unwrap().clone();
    let (path, bytes) = files.iter().next().unwrap();
    assert_eq!(path, &format!("{user}/{}.jpg", reply.body["scanId"].as_str().unwrap()));
    assert!(!bytes.windows(3).any(|window| window == b"GPS"));
    let seen: Value = sqlx::query_scalar("select seen from tomi.fridge_photos").fetch_one(&pool).await.unwrap();
    assert_eq!(seen[0]["ingredient"], "egg");
}

#[sqlx::test(migrations = "./migrations")]
async fn photos_of_other_types_or_sizes_are_refused(pool: PgPool) {
    let (app, cook) = (server(pool), new_cook());
    agree(&app, &cook, "photo-scan").await;
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
    agree(&app, &cook, "photo-scan").await;
    let reply = scan(&app, &cook, jpeg()).await;
    assert_eq!(reply.status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(reply.body["error"]["code"], "ai_unavailable");
}
