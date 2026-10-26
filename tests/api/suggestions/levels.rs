//! Suggestions open levels from the signed-in cook's own finished cooks, never anyone else's.

use super::offers;
use crate::helpers::*;
use crate::progress::{entry, results, with};
use crate::teaching::fixtures::changed;
use axum::Router;
use serde_json::{json, Value};
use sqlx::PgPool;

/// The fixture graph has no eggs level 2 recipe, so here the omelette teaches level 2 as well as level 3.
fn omelette_at_level_two(body: &mut Value) {
    let program = body["things"].as_object_mut().unwrap().values_mut().find(|thing| thing["courses"].is_array());
    let skills = program.unwrap()["courses"][0]["skills"].as_array_mut().unwrap();
    let eggs = skills.iter_mut().find(|skill| skill["slug"] == "eggs").unwrap();
    eggs["levels"][1]["teaches"] = json!([{"thing": "technique/roll-an-omelette", "tier": "drafted"}]);
}

fn server(pool: PgPool) -> Router {
    server_with(pool, Setup { graph: Some(changed(omelette_at_level_two)), ..Setup::default() })
}

/// The omelette offers made to this cook tonight, with eggs and spinach at home.
async fn omelettes(app: &Router, cook: &str) -> Vec<Value> {
    let reply = suggest(app, cook, json!({"ingredients": ["egg", "spinach"]})).await;
    assert_eq!(reply.status, axum::http::StatusCode::OK, "{}", reply.body);
    let all = offers(&reply.body).into_iter().cloned();
    all.filter(|offer| offer["recipeId"] == super::OMELETTE).collect()
}

/// Sends these entries for the cook and checks each was stored.
async fn log(app: &Router, cook: &str, entries: &[Value]) {
    for result in results(app, cook, entries).await {
        assert_eq!(result["status"], "accepted", "{result}");
    }
}

fn stopped_early() -> Value {
    with(&entry(), "outcome", json!("stoppedEarly"))
}

#[sqlx::test(migrations = "./migrations")]
async fn two_finished_level_one_cooks_open_the_skills_level_two_recipes(pool: PgPool) {
    let app = server(pool);
    let cook = new_cook();
    assert!(omelettes(&app, &cook).await.is_empty());
    log(&app, &cook, &[entry(), entry()]).await;
    let offered = omelettes(&app, &cook).await;
    assert_eq!(offered.len(), 1);
    assert_eq!(offered[0]["skill"]["id"], "eggs");
    assert_eq!(offered[0]["level"]["number"], 2);
}

#[sqlx::test(migrations = "./migrations")]
async fn a_stopped_early_cook_opens_nothing(pool: PgPool) {
    let app = server(pool);
    let cook = new_cook();
    log(&app, &cook, &[entry(), stopped_early(), stopped_early()]).await;
    assert!(omelettes(&app, &cook).await.is_empty());
}

#[sqlx::test(migrations = "./migrations")]
async fn a_cook_with_no_entries_gets_the_suggestions_of_a_new_cook(pool: PgPool) {
    let app = server(pool);
    let reply = suggest(&app, &new_cook(), json!({"ingredients": ["egg", "spinach"]})).await;
    assert_eq!(reply.body["special"]["recipeId"], "recipe/soft-scrambled-eggs");
    assert_eq!(reply.body["special"]["level"]["number"], 1);
    assert_eq!(reply.body["alternatives"], json!([]));
}

#[sqlx::test(migrations = "./migrations")]
async fn another_cooks_entries_open_nothing_for_this_cook(pool: PgPool) {
    let app = server(pool);
    let (other, cook) = (new_cook(), new_cook());
    log(&app, &other, &[entry(), entry()]).await;
    assert!(omelettes(&app, &cook).await.is_empty());
    assert_eq!(omelettes(&app, &other).await.len(), 1);
}
