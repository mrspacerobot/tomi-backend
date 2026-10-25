//! Selection uses the offered graph skill level and its actual technique variants.

use super::*;

#[sqlx::test(migrations = "./migrations")]
async fn a_primary_technique_without_teaching_is_not_replaced_by_a_secondary_one(pool: PgPool) {
    let graph = fixtures::changed(|body| {
        fixtures::multiple_techniques(body);
        body["things"]["technique/whisk"]["intro"] = body["things"][TECHNIQUE]["intro"].clone();
        body["things"][TECHNIQUE]["intro"] = Value::Null;
    });
    assert_eq!(offer(pool, graph).await["teaching"], Value::Null);
}

#[sqlx::test(migrations = "./migrations")]
async fn teaching_follows_the_actual_recipe_variant(pool: PgPool) {
    let graph = fixtures::changed(fixtures::variant);
    let teaching = &offer(pool, graph).await["teaching"];
    assert_eq!(teaching["technique"]["id"], "technique/scramble-variant");
    assert_eq!(teaching["quiz"][0]["id"], "technique/scramble-variant/quiz/purpose");
}

#[sqlx::test(migrations = "./migrations")]
async fn an_unknown_technique_tier_refuses_teaching(pool: PgPool) {
    let graph = fixtures::changed(|body| body["things"][TECHNIQUE]["tier"] = json!("future"));
    assert_eq!(offer(pool, graph).await["teaching"], Value::Null);
}

#[sqlx::test(migrations = "./migrations")]
async fn an_invalid_question_identity_makes_the_quiz_unavailable(pool: PgPool) {
    let graph = fixtures::changed(|body| body["things"][TECHNIQUE]["quiz"][0]["id"] = json!("not/a/slug"));
    let teaching = &offer(pool, graph).await["teaching"];
    assert!(teaching["introduction"].is_object());
    assert_eq!(teaching["quiz"], Value::Null);
}
