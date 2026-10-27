//! The cook log: `POST /v1/cook-entries` stores each cook once, per cook, whatever order or how often it arrives,
//! and `GET /v1/progress` and the cook-entries answer compute the progress snapshot from it.

use crate::fakes::FixedClock;
use crate::helpers::*;
use axum::http::Method;
use axum::Router;
use jiff::Timestamp;
use serde_json::{json, Value};
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

mod answers;
mod batch;
mod idempotency;
mod problems;
mod snapshot;

/// A recipe of the fixture graph that teaches eggs level 1, and only through its techniques.
const RECIPE: &str = "recipe/soft-scrambled-eggs";
const FIXTURE_GRAPH: &str = "fixture";

/// A finished cook of a fixture graph recipe, with a new id, as the phone sends it.
fn entry() -> Value {
    json!({
        "schemaVersion": 1,
        "id": Uuid::new_v4(),
        "cookedAt": "2026-10-20T18:42:00Z",
        "timeZone": "Europe/Amsterdam",
        "dish": {"recipeId": RECIPE, "graphVersion": FIXTURE_GRAPH},
        "skill": "eggs",
        "level": 1,
        "outcome": "finished",
        "weeklyGoal": 3,
    })
}

/// The same entry with one field set to another value.
fn with(entry: &Value, field: &str, value: Value) -> Value {
    let mut changed = entry.clone();
    changed[field] = value;
    changed
}

/// A batch of entries, sent with the cook's calendar.
fn batch(entries: &[Value]) -> Value {
    json!({"calendar": {"timeZone": "Europe/Amsterdam", "firstWeekday": 2}, "entries": entries})
}

async fn send(app: &Router, token: Option<&str>, body: &Value) -> Reply {
    let body = Some(("application/json", body.to_string().into_bytes()));
    call(app, Call { method: Method::POST, path: "/v1/cook-entries", token, body }).await
}

/// Sends a batch and returns its results, after checking the batch itself was read.
async fn results(app: &Router, token: &str, entries: &[Value]) -> Vec<Value> {
    let reply = send(app, Some(token), &batch(entries)).await;
    assert_eq!(reply.status, axum::http::StatusCode::OK, "{}", reply.body);
    reply.body["results"].as_array().unwrap().clone()
}

/// Sends one entry and returns its result.
async fn result(app: &Router, token: &str, entry: &Value) -> Value {
    results(app, token, std::slice::from_ref(entry)).await[0].clone()
}

async fn stored_rows(pool: &PgPool) -> i64 {
    sqlx::query_scalar("select count(*) from tomi.cook_entries").fetch_one(pool).await.unwrap()
}

/// The moment the progress tests ask at: Tuesday 27 October 2026, 7:43:02 pm in Amsterdam.
fn request_time() -> Timestamp {
    "2026-10-27T18:43:02Z".parse().unwrap()
}

/// The query of a phone in Amsterdam whose weeks start on Monday.
const AMSTERDAM: &str = "timeZone=Europe/Amsterdam&firstWeekday=2";

/// A test server whose clock reads `request_time()`.
fn server_at_request_time(pool: PgPool) -> Router {
    server_with(pool, Setup { clock: Arc::new(FixedClock(request_time())), ..Setup::default() })
}

/// `GET /v1/progress` with this query.
async fn read_progress(app: &Router, token: Option<&str>, query: &str) -> Reply {
    get(app, &format!("/v1/progress?{query}"), token).await
}
