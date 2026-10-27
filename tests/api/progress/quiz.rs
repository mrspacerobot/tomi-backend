//! Each cook's quiz result is derived from the first answers recorded under its entry id, in whatever order the
//! answers and the entry arrive, and both the snapshot and the cook-entries answer show it.

use super::*;
use crate::quiz::{answer, pick};
use axum::http::StatusCode;

/// The fixture graph's soft scramble quiz, each question with its right and a wrong choice.
const SCRAMBLE_QUIZ: [(&str, usize, usize); 4] = [
    ("technique/scramble/quiz/purpose", 0, 1),
    ("technique/scramble/quiz/method", 1, 0),
    ("technique/scramble/quiz/observation", 2, 0),
    ("technique/scramble/quiz/correction", 0, 1),
];

/// A pick of each question of the quiz for this entry, right unless its index is in `wrong`.
fn picks(entry_id: Uuid, wrong: &[usize]) -> Vec<Value> {
    let mut picks = Vec::new();
    for (index, (question, right_choice, wrong_choice)) in SCRAMBLE_QUIZ.into_iter().enumerate() {
        let choice = if wrong.contains(&index) { wrong_choice } else { right_choice };
        picks.push(pick(entry_id, question, choice));
    }
    picks
}

/// Sends each pick in order, checking that each is answered.
async fn answer_the_quiz(app: &Router, cook: &str, picks: &[Value]) {
    for body in picks {
        let reply = answer(app, cook, body).await;
        assert_eq!(reply.status, StatusCode::OK, "{}", reply.body);
    }
}

/// The entry's id, as the phone made it when cooking began.
fn id_of(entry: &Value) -> Uuid {
    entry["id"].as_str().unwrap().parse().unwrap()
}

/// Posts the entry and returns the XP of the snapshot in the cook-entries answer.
async fn post_and_read_xp(app: &Router, cook: &str, entry: &Value) -> Value {
    let reply = send(app, Some(cook), &batch(std::slice::from_ref(entry))).await;
    assert_eq!(reply.body["results"][0]["status"], "accepted", "{}", reply.body);
    reply.body["progress"]["xp"].clone()
}

async fn xp(app: &Router, cook: &str) -> Value {
    let reply = read_progress(app, Some(cook), AMSTERDAM).await;
    assert_eq!(reply.status, StatusCode::OK, "{}", reply.body);
    reply.body["xp"].clone()
}

#[sqlx::test(migrations = "./migrations")]
async fn answers_before_or_after_the_entry_give_the_same_xp(pool: PgPool) {
    let app = server_at_request_time(pool);
    let (quiz_first, entry_first) = (new_cook(), new_cook());

    let entry = entry();
    answer_the_quiz(&app, &quiz_first, &picks(id_of(&entry), &[])).await;
    assert_eq!(post_and_read_xp(&app, &quiz_first, &entry).await, 60, "the cook-entries answer");

    let entry = super::entry();
    assert_eq!(post_and_read_xp(&app, &entry_first, &entry).await, 50, "no answer recorded yet");
    answer_the_quiz(&app, &entry_first, &picks(id_of(&entry), &[])).await;

    assert_eq!(xp(&app, &quiz_first).await, 60);
    assert_eq!(xp(&app, &entry_first).await, 60);
}

#[sqlx::test(migrations = "./migrations")]
async fn a_wrong_first_answer_keeps_the_bonus_away_even_after_a_right_retry(pool: PgPool) {
    let app = server_at_request_time(pool);
    let (cook, entry) = (new_cook(), entry());
    answer_the_quiz(&app, &cook, &picks(id_of(&entry), &[2])).await;
    let (question, right_choice, _) = SCRAMBLE_QUIZ[2];
    let retry = answer(&app, &cook, &pick(id_of(&entry), question, right_choice)).await;
    assert_eq!((&retry.body["correct"], &retry.body["scored"]), (&json!(true), &json!(false)));
    assert_eq!(post_and_read_xp(&app, &cook, &entry).await, 50);
    assert_eq!(xp(&app, &cook).await, 50);
}

#[sqlx::test(migrations = "./migrations")]
async fn a_question_left_unanswered_earns_no_bonus(pool: PgPool) {
    let app = server_at_request_time(pool);
    let (cook, entry) = (new_cook(), entry());
    for (question, right_choice, _) in &SCRAMBLE_QUIZ[..3] {
        answer(&app, &cook, &pick(id_of(&entry), question, *right_choice)).await;
    }
    assert_eq!(post_and_read_xp(&app, &cook, &entry).await, 50);
}

#[sqlx::test(migrations = "./migrations")]
async fn answers_with_no_entry_or_on_a_stopped_cook_earn_nothing(pool: PgPool) {
    let app = server_at_request_time(pool);
    let cook = new_cook();
    answer_the_quiz(&app, &cook, &picks(Uuid::new_v4(), &[])).await;
    assert_eq!(xp(&app, &cook).await, 0, "answers whose entry never came");

    let stopped = with(&entry(), "outcome", json!("stoppedEarly"));
    answer_the_quiz(&app, &cook, &picks(id_of(&stopped), &[])).await;
    assert_eq!(post_and_read_xp(&app, &cook, &stopped).await, 0);
}

#[sqlx::test(migrations = "./migrations")]
async fn another_cooks_answers_under_the_same_entry_id_earn_this_cook_nothing(pool: PgPool) {
    let app = server_at_request_time(pool);
    let (cook, someone_else, entry) = (new_cook(), new_cook(), entry());
    answer_the_quiz(&app, &someone_else, &picks(id_of(&entry), &[])).await;
    assert_eq!(post_and_read_xp(&app, &cook, &entry).await, 50);
}
