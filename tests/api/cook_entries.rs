//! The cook log: `POST /v1/cook-entries` stores each cook once, per cook, whatever order or how often it arrives.

use crate::helpers::*;
use axum::http::Method;
use axum::Router;
use serde_json::{json, Value};
use sqlx::PgPool;
use uuid::Uuid;

mod idempotency;
mod problems;
mod request;

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

/// One entry, sent with the cook's calendar.
fn request(entry: &Value) -> Value {
    json!({"calendar": {"timeZone": "Europe/Amsterdam", "firstWeekday": 2}, "entry": entry})
}

async fn send(app: &Router, token: Option<&str>, body: &Value) -> Reply {
    let body = Some(("application/json", body.to_string().into_bytes()));
    call(app, Call { method: Method::POST, path: "/v1/cook-entries", token, body }).await
}

/// Sends each entry separately and returns the results in order.
async fn results(app: &Router, token: &str, entries: &[Value]) -> Vec<Value> {
    let mut results = Vec::new();
    for entry in entries {
        let reply = send(app, Some(token), &request(entry)).await;
        assert_eq!(reply.status, axum::http::StatusCode::OK, "{}", reply.body);
        results.push(reply.body["result"].clone());
    }
    results
}

/// Sends one entry and returns its result.
async fn result(app: &Router, token: &str, entry: &Value) -> Value {
    results(app, token, std::slice::from_ref(entry)).await[0].clone()
}

async fn stored_rows(pool: &PgPool) -> i64 {
    sqlx::query_scalar("select count(*) from tomi.cook_entries").fetch_one(pool).await.unwrap()
}
