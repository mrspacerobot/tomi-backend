//! What the cook has tonight: their staples, the ones assumed until they answer, and any left out for the night.

use crate::fakes::FakeAi;
use crate::helpers::suggest;
use crate::suggestions::{comfortable_cook, open_server, OMELETTE};
use serde_json::{json, Value};
use sqlx::PgPool;

/// The graph ingredients an omelette still needs, for a cook with these answers and tonight's call.
async fn omelette_missing(pool: PgPool, answers: Value, tonight: Value) -> Vec<String> {
    let app = open_server(pool, FakeAi::default());
    let cook = comfortable_cook(&app, answers).await;
    let reply = suggest(&app, &cook, tonight).await;
    let offer = reply.body["alternatives"].as_array().unwrap().iter().chain([&reply.body["special"]]);
    let omelette = offer.into_iter().find(|offer| offer["recipeId"] == OMELETTE).unwrap();
    omelette["safePlan"]["missing"]
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item["id"].as_str().unwrap().to_string())
        .collect()
}

#[sqlx::test(migrations = "./migrations")]
async fn before_the_cupboard_is_answered_the_assumed_staples_count_as_at_home(pool: PgPool) {
    let missing = omelette_missing(pool, json!({}), json!({"ingredients": ["egg"]})).await;
    assert!(
        !missing.contains(&"ingredient/salt".to_string()) && !missing.contains(&"ingredient/black-pepper".to_string())
    );
    assert!(missing.contains(&"ingredient/spinach".to_string()));
}

#[sqlx::test(migrations = "./migrations")]
async fn once_answered_only_the_cooks_own_staples_count(pool: PgPool) {
    let answers = json!({"cupboardAnswered": true, "cupboardStaples": ["salt"]});
    let missing = omelette_missing(pool, answers, json!({"ingredients": ["egg"]})).await;
    assert!(!missing.contains(&"ingredient/salt".to_string()));
    assert!(missing.contains(&"ingredient/black-pepper".to_string()));
}

#[sqlx::test(migrations = "./migrations")]
async fn a_cook_who_answered_that_they_have_nothing_is_not_assumed_to_have_salt(pool: PgPool) {
    let answers = json!({"cupboardAnswered": true});
    let missing = omelette_missing(pool, answers, json!({"ingredients": ["egg"]})).await;
    assert!(missing.contains(&"ingredient/salt".to_string()));
}

#[sqlx::test(migrations = "./migrations")]
async fn a_staple_left_out_tonight_is_missing_tonight(pool: PgPool) {
    let tonight = json!({"ingredients": ["egg"], "staplesLeftOut": ["salt"]});
    let missing = omelette_missing(pool, json!({}), tonight).await;
    assert!(missing.contains(&"ingredient/salt".to_string()));
    assert!(!missing.contains(&"ingredient/black-pepper".to_string()));
}

#[sqlx::test(migrations = "./migrations")]
async fn leaving_a_staple_out_never_removes_what_the_cook_confirmed_tonight(pool: PgPool) {
    let tonight = json!({"ingredients": ["egg", "salt"], "staplesLeftOut": ["salt"]});
    let missing = omelette_missing(pool, json!({}), tonight).await;
    assert!(!missing.contains(&"ingredient/salt".to_string()));
}
