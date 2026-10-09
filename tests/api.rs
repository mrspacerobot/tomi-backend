//! End-to-end tests of the API against a real Postgres.
//!
//! `sqlx::test` creates a fresh database per test from `DATABASE_URL` and
//! runs the migrations; see `scripts/test.sh`.

use axum::body::Body;
use axum::http::{Method, Request, StatusCode};
use axum::Router;
use http_body_util::BodyExt;
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use p256::pkcs8::EncodePrivateKey;
use serde_json::{json, Value};
use sqlx::PgPool;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tomi_api::auth::Verifier;
use tomi_api::{app, AppState};
use tower::ServiceExt;
use uuid::Uuid;

const PROJECT: &str = "https://test-project.supabase.co";
const SECRET: &str = "test-only-shared-secret-not-used-anywhere-else";

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

fn claims(subject: Uuid) -> Value {
    json!({
        "sub": subject,
        "aud": "authenticated",
        "role": "authenticated",
        "iss": format!("{PROJECT}/auth/v1"),
        "exp": now() + 600,
        "is_anonymous": false,
    })
}

fn token_with(claims: &Value) -> String {
    encode(
        &Header::new(Algorithm::HS256),
        claims,
        &EncodingKey::from_secret(SECRET.as_bytes()),
    )
    .unwrap()
}

fn token(subject: Uuid) -> String {
    token_with(&claims(subject))
}

fn server(pool: PgPool) -> Router {
    server_with(pool, Verifier::new(PROJECT, Some(SECRET), false))
}

fn server_with(pool: PgPool, verifier: Verifier) -> Router {
    app(AppState {
        db: pool,
        verifier: Arc::new(verifier),
        min_app_version: "1.2.0".into(),
    })
}

struct Reply {
    status: StatusCode,
    headers: axum::http::HeaderMap,
    body: Value,
}

async fn send(
    app: &Router,
    method: Method,
    path: &str,
    token: Option<&str>,
    body: Option<Value>,
) -> Reply {
    let mut request = Request::builder().method(method).uri(path);
    if let Some(token) = token {
        request = request.header("authorization", format!("Bearer {token}"));
    }
    let request = match body {
        Some(body) => request
            .header("content-type", "application/json")
            .body(Body::from(body.to_string())),
        None => request.body(Body::empty()),
    }
    .unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    let status = response.status();
    let headers = response.headers().clone();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let body = serde_json::from_slice(&bytes)
        .unwrap_or(Value::String(String::from_utf8_lossy(&bytes).into()));
    Reply {
        status,
        headers,
        body,
    }
}

async fn get(app: &Router, path: &str, token: Option<&str>) -> Reply {
    send(app, Method::GET, path, token, None).await
}

async fn patch(app: &Router, token: &str, body: Value) -> Reply {
    send(app, Method::PATCH, "/v1/profile", Some(token), Some(body)).await
}

#[sqlx::test(migrations = "./migrations")]
async fn health_and_bootstrap_need_no_sign_in(pool: PgPool) {
    let app = server(pool);
    assert_eq!(get(&app, "/healthz", None).await.status, StatusCode::OK);
    assert_eq!(get(&app, "/readyz", None).await.status, StatusCode::OK);

    let reply = get(&app, "/v1/bootstrap", None).await;
    assert_eq!(reply.status, StatusCode::OK);
    assert_eq!(reply.body["min_app_version"], "1.2.0");
    assert_eq!(reply.body["library_version"], Value::Null);
    assert_eq!(reply.body["safety_version"], Value::Null);
    assert!(reply.headers.contains_key("x-request-id"));
}

#[sqlx::test(migrations = "./migrations")]
async fn the_profile_needs_a_valid_token(pool: PgPool) {
    let app = server(pool);
    let user = Uuid::new_v4();

    let reply = get(&app, "/v1/profile", None).await;
    assert_eq!(reply.status, StatusCode::UNAUTHORIZED);
    assert_eq!(reply.body["error"]["code"], "unauthenticated");

    let mut expired = claims(user);
    expired["exp"] = json!(now() - 3600);
    let mut other_project = claims(user);
    other_project["iss"] = json!("https://someone-else.supabase.co/auth/v1");
    let mut anon_key = claims(user);
    anon_key["role"] = json!("anon");
    anon_key["aud"] = json!("anon");
    let wrong_secret = encode(
        &Header::new(Algorithm::HS256),
        &claims(user),
        &EncodingKey::from_secret(b"guessed"),
    )
    .unwrap();

    for bad in [
        token_with(&expired),
        token_with(&other_project),
        token_with(&anon_key),
        wrong_secret,
        "junk".into(),
    ] {
        let reply = get(&app, "/v1/profile", Some(&bad)).await;
        assert_eq!(reply.status, StatusCode::UNAUTHORIZED, "{bad}");
    }

    let mut anonymous = claims(user);
    anonymous["is_anonymous"] = json!(true);
    let reply = get(&app, "/v1/profile", Some(&token_with(&anonymous))).await;
    assert_eq!(reply.status, StatusCode::FORBIDDEN);
    assert_eq!(reply.body["error"]["code"], "anonymous_not_allowed");
}

#[sqlx::test(migrations = "./migrations")]
async fn asymmetric_tokens_are_checked_against_the_key_set(pool: PgPool) {
    let secret = p256::SecretKey::random(&mut rand_core::OsRng);
    let mut jwk: Value = serde_json::from_str(&secret.public_key().to_jwk_string()).unwrap();
    jwk["kid"] = json!("key-1");
    jwk["alg"] = json!("ES256");
    let jwks = serde_json::from_value(json!({ "keys": [jwk] })).unwrap();
    let app = server_with(pool, Verifier::with_keys(PROJECT, &jwks));

    let signing = EncodingKey::from_ec_der(secret.to_pkcs8_der().unwrap().as_bytes());
    let mut header = Header::new(Algorithm::ES256);
    header.kid = Some("key-1".into());
    let good = encode(&header, &claims(Uuid::new_v4()), &signing).unwrap();
    assert_eq!(
        get(&app, "/v1/profile", Some(&good)).await.status,
        StatusCode::OK
    );

    header.kid = Some("key-unknown".into());
    let unknown_kid = encode(&header, &claims(Uuid::new_v4()), &signing).unwrap();
    assert_eq!(
        get(&app, "/v1/profile", Some(&unknown_kid)).await.status,
        StatusCode::UNAUTHORIZED
    );

    // This verifier has no shared secret, so an HS256 token cannot pass.
    assert_eq!(
        get(&app, "/v1/profile", Some(&token(Uuid::new_v4())))
            .await
            .status,
        StatusCode::UNAUTHORIZED
    );
}

#[sqlx::test(migrations = "./migrations")]
async fn a_new_cook_starts_with_version_zero_and_saves_version_one(pool: PgPool) {
    let app = server(pool);
    let token = token(Uuid::new_v4());

    let reply = get(&app, "/v1/profile", Some(&token)).await;
    assert_eq!(reply.status, StatusCode::OK);
    assert_eq!(reply.body["version"], 0);
    assert_eq!(reply.body["profile"]["avoid"], json!([]));

    let reply = patch(
        &app,
        &token,
        json!({"version": 0, "avoid": ["fact/peanuts"], "servings": 4}),
    )
    .await;
    assert_eq!(reply.status, StatusCode::OK, "{}", reply.body);
    assert_eq!(reply.body["version"], 1);

    let reply = get(&app, "/v1/profile", Some(&token)).await;
    assert_eq!(reply.body["version"], 1);
    assert_eq!(reply.body["profile"]["avoid"], json!(["fact/peanuts"]));
    assert_eq!(reply.body["profile"]["servings"], 4);
}

#[sqlx::test(migrations = "./migrations")]
async fn a_stale_version_is_refused_with_the_current_profile(pool: PgPool) {
    let app = server(pool);
    let token = token(Uuid::new_v4());
    patch(&app, &token, json!({"version": 0, "servings": 3})).await;

    for stale in [0, 5] {
        let reply = patch(&app, &token, json!({"version": stale, "servings": 6})).await;
        assert_eq!(reply.status, StatusCode::CONFLICT);
        assert_eq!(reply.body["error"]["code"], "version_conflict");
        assert_eq!(reply.body["error"]["details"]["current"]["version"], 1);
        assert_eq!(
            reply.body["error"]["details"]["current"]["profile"]["servings"],
            3
        );
    }
}

#[sqlx::test(migrations = "./migrations")]
async fn restrictions_are_never_dropped_silently(pool: PgPool) {
    let app = server(pool);
    let token = token(Uuid::new_v4());
    patch(
        &app,
        &token,
        json!({"version": 0, "avoid": ["fact/peanuts", "fact/sesame"]}),
    )
    .await;

    let reply = patch(
        &app,
        &token,
        json!({"version": 1, "avoid": ["fact/sesame"]}),
    )
    .await;
    assert_eq!(reply.status, StatusCode::CONFLICT);
    assert_eq!(reply.body["error"]["code"], "weakens_restrictions");
    assert_eq!(
        reply.body["error"]["details"]["removed"],
        json!(["fact/peanuts"])
    );
    let unchanged = get(&app, "/v1/profile", Some(&token)).await;
    assert_eq!(
        unchanged.body["profile"]["avoid"],
        json!(["fact/peanuts", "fact/sesame"])
    );

    let reply = patch(
        &app,
        &token,
        json!({"version": 1, "avoid": ["fact/sesame"], "allow_weaker_restrictions": true}),
    )
    .await;
    assert_eq!(reply.status, StatusCode::OK);
    assert_eq!(reply.body["profile"]["avoid"], json!(["fact/sesame"]));
}

#[sqlx::test(migrations = "./migrations")]
async fn bad_bodies_are_refused(pool: PgPool) {
    let app = server(pool);
    let token = token(Uuid::new_v4());

    let reply = patch(&app, &token, json!({"version": 0, "pantry": ["fact/milk"]})).await;
    assert_eq!(reply.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(
        reply.body["error"]["details"]["problems"],
        json!(["pantry: `fact/milk` is not of kind ingredient"])
    );

    for body in [
        json!({"version": 0, "xp": 100}),
        json!({"servings": 2}),
        json!({"version": 0, "avoid": ["peanuts"]}),
    ] {
        let reply = patch(&app, &token, body.clone()).await;
        assert_eq!(reply.status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
        assert_eq!(reply.body["error"]["code"], "invalid_request");
    }
}

#[sqlx::test(migrations = "./migrations")]
async fn cooks_see_only_their_own_profile(pool: PgPool) {
    let app = server(pool);
    let ana = token(Uuid::new_v4());
    let ben = token(Uuid::new_v4());
    patch(&app, &ana, json!({"version": 0, "avoid": ["fact/milk"]})).await;

    let reply = get(&app, "/v1/profile", Some(&ben)).await;
    assert_eq!(reply.body["version"], 0);
    assert_eq!(reply.body["profile"]["avoid"], json!([]));
}

#[sqlx::test(migrations = "./migrations")]
async fn a_deleted_account_is_refused(pool: PgPool) {
    let app = server(pool.clone());
    let subject = Uuid::new_v4();
    let token = token(subject);
    assert_eq!(
        get(&app, "/v1/profile", Some(&token)).await.status,
        StatusCode::OK
    );

    sqlx::query("update tomi.users set deleted_at = now() where auth_subject = $1")
        .bind(subject)
        .execute(&pool)
        .await
        .unwrap();
    let reply = get(&app, "/v1/profile", Some(&token)).await;
    assert_eq!(reply.status, StatusCode::FORBIDDEN);
    assert_eq!(reply.body["error"]["code"], "account_deleted");
}
