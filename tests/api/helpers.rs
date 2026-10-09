//! A test server, signed tokens and a small client to call it with.

use axum::body::Body;
use axum::http::{HeaderMap, Method, Request, StatusCode};
use axum::Router;
use http_body_util::BodyExt;
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use serde_json::{json, Value};
use sqlx::PgPool;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tomi_api::auth::Verifier;
use tomi_api::{app, AppState};
use tower::ServiceExt;
use uuid::Uuid;

pub const PROJECT: &str = "https://test-project.supabase.co";
const SECRET: &str = "test-only-shared-secret-not-used-anywhere-else";

/// A server that accepts tokens signed with the test secret.
pub fn server(pool: PgPool) -> Router {
    let verifier = Verifier::new(PROJECT, Some(SECRET)).unwrap();
    server_with(pool, verifier)
}

pub fn server_with(pool: PgPool, verifier: Verifier) -> Router {
    let state = AppState {
        db: pool,
        verifier: Arc::new(verifier),
        min_app_version: "1.2.0".into(),
    };
    app(state)
}

pub fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

/// The claims Supabase puts on a signed-in user's token.
pub fn claims(subject: Uuid) -> Value {
    json!({
        "sub": subject,
        "aud": "authenticated",
        "role": "authenticated",
        "iss": format!("{PROJECT}/auth/v1"),
        "exp": now() + 600,
        "is_anonymous": false,
    })
}

pub fn token_with(claims: &Value) -> String {
    let key = EncodingKey::from_secret(SECRET.as_bytes());
    encode(&Header::new(Algorithm::HS256), claims, &key).unwrap()
}

/// A valid token for a new cook.
pub fn new_cook() -> String {
    token_with(&claims(Uuid::new_v4()))
}

pub struct Reply {
    pub status: StatusCode,
    pub headers: HeaderMap,
    pub body: Value,
}

pub async fn get(app: &Router, path: &str, token: Option<&str>) -> Reply {
    let request = builder(Method::GET, path, token)
        .body(Body::empty())
        .unwrap();
    send(app, request).await
}

pub async fn patch_profile(app: &Router, token: &str, body: Value) -> Reply {
    let builder = builder(Method::PATCH, "/v1/profile", Some(token));
    let request = builder
        .header("content-type", "application/json")
        .body(Body::from(body.to_string()));
    send(app, request.unwrap()).await
}

fn builder(method: Method, path: &str, token: Option<&str>) -> axum::http::request::Builder {
    let builder = Request::builder().method(method).uri(path);
    match token {
        Some(token) => builder.header("authorization", format!("Bearer {token}")),
        None => builder,
    }
}

async fn send(app: &Router, request: Request<Body>) -> Reply {
    let response = app.clone().oneshot(request).await.unwrap();
    let status = response.status();
    let headers = response.headers().clone();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let body = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
    Reply {
        status,
        headers,
        body,
    }
}
