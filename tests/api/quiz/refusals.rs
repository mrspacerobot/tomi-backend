//! A pick the pinned graph's teaching does not ask for is refused, stores nothing and gives no answer away.

use super::*;
use crate::teaching::fixtures;

const OMELETTE: &str = "recipe/spinach-and-cheese-omelette";
const OMELETTE_QUESTION: &str = "technique/roll-an-omelette/quiz/purpose";

async fn assert_refused(pool: &PgPool, reply: &Reply, problem: &str) {
    assert_eq!(reply.status, StatusCode::UNPROCESSABLE_ENTITY, "{}", reply.body);
    assert_eq!(reply.body["error"]["code"], "invalid_request");
    assert_eq!(reply.body["error"]["details"]["problems"], json!([problem]));
    assert!(!reply.body.to_string().contains(&explanation(0)));
    assert!(stored(pool).await.is_empty());
}

#[sqlx::test(migrations = "./migrations")]
async fn a_question_of_another_recipes_quiz_is_refused(pool: PgPool) {
    let app = server(pool.clone());
    let reply = answer(&app, &new_cook(), &pick(Uuid::new_v4(), OMELETTE_QUESTION, 1)).await;
    assert_refused(&pool, &reply, "question_not_in_quiz").await;
}

#[sqlx::test(migrations = "./migrations")]
async fn the_same_question_is_accepted_for_the_recipe_that_teaches_it(pool: PgPool) {
    let app = server(pool);
    let mut body = pick(Uuid::new_v4(), OMELETTE_QUESTION, 1);
    body["recipeId"] = json!(OMELETTE);
    let reply = answer(&app, &new_cook(), &body).await;
    assert_eq!(reply.status, StatusCode::OK, "{}", reply.body);
    assert_eq!(reply.body["correct"], true);
}

#[sqlx::test(migrations = "./migrations")]
async fn an_unknown_question_or_recipe_is_refused(pool: PgPool) {
    let app = server(pool.clone());
    let token = new_cook();
    let unknown = answer(&app, &token, &pick(Uuid::new_v4(), "technique/scramble/quiz/unknown", 0)).await;
    assert_refused(&pool, &unknown, "question_not_in_quiz").await;
    let mut body = pick(Uuid::new_v4(), QUESTION, RIGHT);
    body["recipeId"] = json!("recipe/unknown");
    assert_refused(&pool, &answer(&app, &token, &body).await, "question_not_in_quiz").await;
}

#[sqlx::test(migrations = "./migrations")]
async fn a_choice_that_is_not_one_of_the_choices_is_refused(pool: PgPool) {
    let app = server(pool.clone());
    let reply = answer(&app, &new_cook(), &pick(Uuid::new_v4(), QUESTION, 3)).await;
    assert_refused(&pool, &reply, "choice_out_of_range").await;
}

#[sqlx::test(migrations = "./migrations")]
async fn a_quiz_that_is_not_taught_today_is_refused(pool: PgPool) {
    let graph = fixtures::changed(|body| body["things"]["technique/scramble"]["intro"] = Value::Null);
    let app = server_with(pool.clone(), Setup { graph: Some(graph), ..Setup::default() });
    let reply = answer(&app, &new_cook(), &pick(Uuid::new_v4(), QUESTION, RIGHT)).await;
    assert_refused(&pool, &reply, "question_not_in_quiz").await;
}

#[sqlx::test(migrations = "./migrations")]
async fn a_stale_graph_version_is_a_permanent_conflict(pool: PgPool) {
    let app = server(pool.clone());
    let mut body = pick(Uuid::new_v4(), QUESTION, RIGHT);
    body["graphVersion"] = json!("2026.01.01");
    let reply = answer(&app, &new_cook(), &body).await;
    assert_eq!(reply.status, StatusCode::CONFLICT);
    assert_eq!(reply.body["error"]["code"], "graph_version_changed");
    assert!(stored(&pool).await.is_empty());
}

#[sqlx::test(migrations = "./migrations")]
async fn without_a_graph_the_quiz_is_unavailable(pool: PgPool) {
    let app = server_with(pool.clone(), Setup { graph: None, ..Setup::default() });
    let reply = answer(&app, &new_cook(), &pick(Uuid::new_v4(), QUESTION, RIGHT)).await;
    assert_eq!(reply.status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(reply.body["error"]["code"], "graph_unavailable");
}

#[sqlx::test(migrations = "./migrations")]
async fn an_unreadable_pick_is_refused(pool: PgPool) {
    let app = server(pool.clone());
    let token = new_cook();
    let mut extra = pick(Uuid::new_v4(), QUESTION, RIGHT);
    extra["correct"] = json!(true);
    let mut negative = pick(Uuid::new_v4(), QUESTION, RIGHT);
    negative["choice"] = json!(-1);
    let mut no_entry = pick(Uuid::new_v4(), QUESTION, RIGHT);
    no_entry["entryId"] = json!("not-a-uuid");
    for body in [extra, negative, no_entry] {
        let reply = answer(&app, &token, &body).await;
        assert_eq!(reply.status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
    }
    assert!(stored(&pool).await.is_empty());
}
