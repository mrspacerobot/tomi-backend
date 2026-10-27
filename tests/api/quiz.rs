//! `POST /v1/quiz-answers`: one pick in, right or wrong out, and the first pick of each question is the score.

use crate::helpers::*;
use axum::http::{Method, StatusCode};
use serde_json::{json, Value};
use sqlx::PgPool;
use uuid::Uuid;

mod accounts;
mod example;
mod refusals;

pub const RECIPE: &str = "recipe/soft-scrambled-eggs";
/// The fixture graph's answer to this question is the first choice.
pub const QUESTION: &str = "technique/scramble/quiz/purpose";
pub const RIGHT: usize = 0;
pub const WRONG: usize = 1;

/// A pick for the soft scramble quiz under the pinned fixture graph.
pub fn pick(entry_id: Uuid, question_id: &str, choice: usize) -> Value {
    json!({"entryId": entry_id, "recipeId": RECIPE, "graphVersion": "fixture", "questionId": question_id,
        "choice": choice})
}

pub async fn answer(app: &axum::Router, token: &str, body: &Value) -> Reply {
    let body = Some(("application/json", body.to_string().into_bytes()));
    call(app, Call { method: Method::POST, path: "/v1/quiz-answers", token: Some(token), body }).await
}

/// The graph's explanation of a scramble question, which may leave the server only after a right pick.
pub fn explanation(index: usize) -> String {
    let graph = fixture_graph();
    let quiz = &graph.thing("technique/scramble").unwrap()["quiz"];
    quiz[index]["because"].as_str().unwrap().to_string()
}

/// The stored first answers of every cook, as `(entry id, question id, choice, correct)`.
pub async fn stored(pool: &PgPool) -> Vec<(Uuid, String, i32, bool)> {
    let query = "select entry_id, question_id, choice, correct from tomi.quiz_answers order by answered_at";
    sqlx::query_as(query).fetch_all(pool).await.unwrap()
}

#[sqlx::test(migrations = "./migrations")]
async fn a_right_pick_is_scored_and_explained(pool: PgPool) {
    let app = server(pool);
    let reply = answer(&app, &new_cook(), &pick(Uuid::new_v4(), QUESTION, RIGHT)).await;
    assert_eq!(reply.status, StatusCode::OK);
    let expected = json!({"questionId": QUESTION, "correct": true, "scored": true, "explanation": explanation(0)});
    assert_eq!(reply.body, expected);
}

#[sqlx::test(migrations = "./migrations")]
async fn a_wrong_pick_carries_neither_the_explanation_nor_the_answer(pool: PgPool) {
    let app = server(pool);
    let reply = answer(&app, &new_cook(), &pick(Uuid::new_v4(), QUESTION, WRONG)).await;
    assert_eq!(reply.status, StatusCode::OK);
    let expected = json!({"questionId": QUESTION, "correct": false, "scored": true, "explanation": null});
    assert_eq!(reply.body, expected);
    assert!(!reply.body.to_string().contains(&explanation(0)));
}

#[sqlx::test(migrations = "./migrations")]
async fn a_right_retry_teaches_and_the_wrong_first_pick_stays_the_score(pool: PgPool) {
    let app = server(pool.clone());
    let (token, entry) = (new_cook(), Uuid::new_v4());
    let first = answer(&app, &token, &pick(entry, QUESTION, WRONG)).await;
    assert_eq!(first.body["scored"], true);
    let retry = answer(&app, &token, &pick(entry, QUESTION, RIGHT)).await;
    let expected = json!({"questionId": QUESTION, "correct": true, "scored": false, "explanation": explanation(0)});
    assert_eq!(retry.body, expected);
    assert_eq!(stored(&pool).await, vec![(entry, QUESTION.to_string(), 1, false)]);
}

#[sqlx::test(migrations = "./migrations")]
async fn the_same_first_pick_sent_twice_is_stored_once_and_answered_the_same(pool: PgPool) {
    let app = server(pool.clone());
    let (token, entry) = (new_cook(), Uuid::new_v4());
    let first = answer(&app, &token, &pick(entry, QUESTION, WRONG)).await;
    let again = answer(&app, &token, &pick(entry, QUESTION, WRONG)).await;
    assert_eq!(first.body, again.body);
    assert_eq!(again.body["scored"], true);
    assert_eq!(stored(&pool).await.len(), 1);
}

#[sqlx::test(migrations = "./migrations")]
async fn each_question_and_each_cook_entry_keeps_its_own_first_answer(pool: PgPool) {
    let app = server(pool.clone());
    let token = new_cook();
    let (entry, next_cook) = (Uuid::new_v4(), Uuid::new_v4());
    answer(&app, &token, &pick(entry, QUESTION, WRONG)).await;
    let other_question = answer(&app, &token, &pick(entry, "technique/scramble/quiz/method", 1)).await;
    let next_cook = answer(&app, &token, &pick(next_cook, QUESTION, RIGHT)).await;
    assert_eq!(other_question.body["scored"], true);
    assert_eq!(other_question.body["correct"], true);
    assert_eq!(next_cook.body["scored"], true);
    assert_eq!(stored(&pool).await.len(), 3);
}

#[sqlx::test(migrations = "./migrations")]
async fn the_first_answer_keeps_the_whole_quiz_it_belongs_to(pool: PgPool) {
    let app = server(pool.clone());
    answer(&app, &new_cook(), &pick(Uuid::new_v4(), QUESTION, RIGHT)).await;
    let query = "select recipe_id, graph_version, quiz_question_ids from tomi.quiz_answers";
    let row: (String, String, Vec<String>) = sqlx::query_as(query).fetch_one(&pool).await.unwrap();
    let quiz = ["purpose", "method", "observation", "correction"].map(|id| format!("technique/scramble/quiz/{id}"));
    assert_eq!(row, (RECIPE.to_string(), "fixture".to_string(), quiz.to_vec()));
}

#[sqlx::test(migrations = "./migrations")]
async fn quiz_answers_are_hidden_from_supabase_data_api(pool: PgPool) {
    let query = "select relrowsecurity from pg_class where oid = 'tomi.quiz_answers'::regclass";
    let protected: bool = sqlx::query_scalar(query).fetch_one(&pool).await.unwrap();
    assert!(protected);
}
