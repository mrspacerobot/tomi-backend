//! A small client that calls the test server the way the app would.

use axum::body::Body;
use axum::http::{HeaderMap, Method, Request, StatusCode};
use axum::Router;
use http_body_util::BodyExt;
use serde_json::Value;
use tower::ServiceExt;

pub struct Reply {
    pub status: StatusCode,
    pub headers: HeaderMap,
    pub body: Value,
}

/// One request: method and path, the cook's token, and an optional body
/// with its content type.
pub struct Call<'a> {
    pub method: Method,
    pub path: &'a str,
    pub token: Option<&'a str>,
    pub body: Option<(&'a str, Vec<u8>)>,
}

pub async fn get(app: &Router, path: &str, token: Option<&str>) -> Reply {
    call(app, Call { method: Method::GET, path, token, body: None }).await
}

pub async fn patch_profile(app: &Router, token: &str, body: Value) -> Reply {
    let json = ("application/json", body.to_string().into_bytes());
    let request = Call { method: Method::PATCH, path: "/v1/profile", token: Some(token), body: Some(json) };
    call(app, request).await
}

pub async fn call(app: &Router, call: Call<'_>) -> Reply {
    call_in(app, call, "en").await
}

/// A call from an app set to this `Accept-Language`.
pub async fn call_in(app: &Router, call: Call<'_>, language: &str) -> Reply {
    let mut builder = Request::builder().method(call.method).uri(call.path).header("accept-language", language);
    if let Some(token) = call.token {
        builder = builder.header("authorization", format!("Bearer {token}"));
    }
    let request = match call.body {
        Some((content_type, bytes)) => builder.header("content-type", content_type).body(Body::from(bytes)),
        None => builder.body(Body::empty()),
    };
    send(app, request.unwrap()).await
}

async fn send(app: &Router, request: Request<Body>) -> Reply {
    let response = app.clone().oneshot(request).await.unwrap();
    let status = response.status();
    let headers = response.headers().clone();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let body = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
    Reply { status, headers, body }
}
