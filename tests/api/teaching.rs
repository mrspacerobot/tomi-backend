//! Teaching is additive: malformed content refuses the launch, never the checked recipe.

use crate::helpers::*;
use serde_json::{json, Value};
use sqlx::PgPool;

mod fixtures;
mod policy;
mod selection;

const TECHNIQUE: &str = "technique/scramble";
const RECIPE: &str = "recipe/soft-scrambled-eggs";

async fn offer(pool: PgPool, graph: tomi_api::graph::Graph) -> Value {
    let app = server_with(pool, Setup { graph: Some(graph), ..Setup::default() });
    let foods = json!({"ingredients": ["egg", "butter", "cream", "salt"]});
    let reply = suggest(&app, &new_cook(), foods).await;
    assert_eq!(reply.body["special"]["recipeId"], RECIPE);
    reply.body["special"].clone()
}

#[sqlx::test(migrations = "./migrations")]
async fn an_unchanged_recipe_carries_pinned_graph_teaching(pool: PgPool) {
    let offer = offer(pool, fixture_graph()).await;
    let teaching = &offer["teaching"];
    assert_eq!(teaching["schemaVersion"], 1);
    assert_eq!(teaching["graphVersion"], "fixture");
    assert_eq!(teaching["graphChannel"], "draft");
    assert_eq!(teaching["language"], "en");
    assert_eq!(teaching["technique"]["id"], TECHNIQUE);
    assert_eq!(teaching["introduction"]["tier"], "drafted");
    assert_eq!(teaching, &fixtures::expected());
}

#[sqlx::test(migrations = "./migrations")]
async fn a_missing_introduction_keeps_browsing_and_refuses_teaching(pool: PgPool) {
    let graph = fixtures::changed(|body| body["things"][TECHNIQUE]["intro"] = Value::Null);
    let offer = offer(pool, graph).await;
    assert!(offer["recipe"]["steps"].is_array());
    assert_eq!(offer["teaching"], Value::Null);
}

#[sqlx::test(migrations = "./migrations")]
async fn blank_introduction_text_refuses_teaching(pool: PgPool) {
    for field in ["idea", "what", "why"] {
        let graph = fixtures::changed(|body| body["things"][TECHNIQUE]["intro"][field] = json!(" "));
        assert_eq!(offer(pool.clone(), graph).await["teaching"], Value::Null, "{field}");
    }
}

#[sqlx::test(migrations = "./migrations")]
async fn missing_quiz_keeps_the_introduction(pool: PgPool) {
    let graph = fixtures::changed(|body| body["things"][TECHNIQUE]["quiz"] = Value::Null);
    let teaching = &offer(pool, graph).await["teaching"];
    assert!(teaching["introduction"].is_object());
    assert_eq!(teaching["quiz"], Value::Null);
}

#[sqlx::test(migrations = "./migrations")]
async fn an_invalid_answer_index_makes_the_quiz_unavailable(pool: PgPool) {
    let graph = fixtures::changed(|body| body["things"][TECHNIQUE]["quiz"][0]["answer"] = json!(999));
    let teaching = &offer(pool, graph).await["teaching"];
    assert!(teaching["introduction"].is_object());
    assert_eq!(teaching["quiz"], Value::Null);
}

#[sqlx::test(migrations = "./migrations")]
async fn repeated_question_ids_make_the_quiz_unavailable(pool: PgPool) {
    let graph = fixtures::changed(fixtures::duplicate_question);
    let teaching = &offer(pool, graph).await["teaching"];
    assert!(teaching["introduction"].is_object());
    assert_eq!(teaching["quiz"], Value::Null);
}

#[sqlx::test(migrations = "./migrations")]
async fn actual_recipe_techniques_are_selected_independent_of_order(pool: PgPool) {
    let graph = fixtures::changed(fixtures::multiple_techniques);
    assert_eq!(offer(pool, graph).await["teaching"]["technique"]["id"], TECHNIQUE);
}

#[sqlx::test(migrations = "./migrations")]
async fn malformed_quiz_fields_leave_a_valid_introduction(pool: PgPool) {
    for (field, value) in fixtures::invalid_quiz_fields() {
        let graph = fixtures::changed(|body| body["things"][TECHNIQUE]["quiz"][0][field] = value);
        let teaching = &offer(pool.clone(), graph).await["teaching"];
        assert!(teaching["introduction"].is_object(), "{field}");
        assert_eq!(teaching["quiz"], Value::Null, "{field}");
    }
}
