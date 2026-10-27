//! The documented quiz answer is what the server really sends for the documented pick.

use super::*;

#[sqlx::test(migrations = "./migrations")]
async fn the_example_matches_the_real_answer(pool: PgPool) {
    let example: Value = serde_json::from_str(include_str!("../../../docs/examples/quiz-answer.json")).unwrap();
    let mut request = example["request"].clone();
    request["graphVersion"] = json!("fixture");
    let reply = answer(&server(pool), &new_cook(), &request).await;
    assert_eq!(reply.status, StatusCode::OK, "{}", reply.body);
    assert_eq!(reply.body, example["answer"]);
}
