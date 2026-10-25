//! Draft policy and adaptation compatibility are enforced before teaching reaches the app.

use super::*;

#[sqlx::test(migrations = "./migrations")]
async fn a_production_header_does_not_promote_draft_teaching(pool: PgPool) {
    let graph = fixtures::changed(|body| body["channel"] = json!("production"));
    assert_eq!(offer(pool, graph).await["teaching"], Value::Null);
}

#[sqlx::test(migrations = "./migrations")]
async fn reviewed_teaching_can_be_offered_on_production(pool: PgPool) {
    let graph = fixtures::changed(fixtures::reviewed);
    let teaching = &offer(pool, graph).await["teaching"];
    assert_eq!(teaching["graphChannel"], "production");
    assert_eq!(teaching["introduction"]["tier"], "reviewed");
}

#[sqlx::test(migrations = "./migrations")]
async fn an_unknown_intro_tier_refuses_teaching(pool: PgPool) {
    let graph = fixtures::changed(|body| body["things"][TECHNIQUE]["intro"]["tier"] = json!("future"));
    assert_eq!(offer(pool, graph).await["teaching"], Value::Null);
}

#[sqlx::test(migrations = "./migrations")]
async fn a_production_quiz_below_reviewed_is_unavailable(pool: PgPool) {
    let graph = fixtures::changed(|body| {
        fixtures::reviewed(body);
        body["things"][TECHNIQUE]["quiz"][0]["tier"] = json!("sourced");
    });
    let teaching = &offer(pool, graph).await["teaching"];
    assert!(teaching["introduction"].is_object());
    assert_eq!(teaching["quiz"], Value::Null);
}

#[sqlx::test(migrations = "./migrations")]
async fn an_adapted_recipe_requires_reviewed_teaching_compatibility(pool: PgPool) {
    let app = server(pool);
    let foods = json!({"ingredients": ["egg", "butter", "creme-fraiche", "salt"]});
    let reply = suggest(&app, &new_cook(), foods).await;
    let offer = &reply.body["special"];
    assert_eq!(offer["recipeId"], RECIPE);
    assert!(!offer["changes"]["swaps"].as_array().unwrap().is_empty());
    assert_eq!(offer["teaching"], Value::Null);
}

#[sqlx::test(migrations = "./migrations")]
async fn each_production_teaching_dependency_must_be_reviewed(pool: PgPool) {
    for path in fixtures::tier_paths() {
        let graph = fixtures::changed(|body| fixtures::below_reviewed(body, &path));
        assert_eq!(offer(pool.clone(), graph).await["teaching"], Value::Null, "{path:?}");
    }
}
