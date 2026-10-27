//! An entry that breaks a rule is rejected with permanent problem codes and stores nothing.

use super::*;
use serde_json::json;

/// Sends one entry and checks it is rejected with exactly these problems and nothing is stored.
async fn rejected(pool: &PgPool, sent: &Value, problems: &[&str]) {
    let reply = result(&server(pool.clone()), &new_cook(), sent).await;
    assert_eq!(reply, json!({"id": sent["id"], "status": "rejected", "problems": problems}));
    assert_eq!(stored_rows(pool).await, 0);
}

#[sqlx::test(migrations = "./migrations")]
async fn an_unknown_schema_version_is_rejected_before_anything_else_is_read(pool: PgPool) {
    let sent = json!({"schemaVersion": 2, "id": Uuid::new_v4(), "somethingNew": true});
    rejected(&pool, &sent, &["unsupported_schema_version"]).await;
}

#[sqlx::test(migrations = "./migrations")]
async fn a_skill_that_is_not_an_id_is_rejected(pool: PgPool) {
    rejected(&pool, &with(&entry(), "skill", json!("Eggs!")), &["invalid_skill_id"]).await;
}

#[sqlx::test(migrations = "./migrations")]
async fn a_level_that_is_not_a_level_number_is_rejected(pool: PgPool) {
    for level in [0, -1, 256] {
        let sent = with(&entry(), "level", json!(level));
        rejected(&pool, &sent, &["level_out_of_range"]).await;
    }
}

#[sqlx::test(migrations = "./migrations")]
async fn a_weekly_goal_outside_the_game_range_is_rejected(pool: PgPool) {
    for goal in [0, 8] {
        rejected(&pool, &with(&entry(), "weeklyGoal", json!(goal)), &["weekly_goal_out_of_range"]).await;
    }
}

#[sqlx::test(migrations = "./migrations")]
async fn a_cook_far_in_the_future_is_rejected(pool: PgPool) {
    rejected(&pool, &with(&entry(), "cookedAt", json!("2999-01-01T12:00:00Z")), &["cooked_in_future"]).await;
}

#[sqlx::test(migrations = "./migrations")]
async fn a_time_zone_that_is_not_an_iana_zone_is_rejected(pool: PgPool) {
    for zone in ["Mars/Olympus_Mons", "+02:00", ""] {
        rejected(&pool, &with(&entry(), "timeZone", json!(zone)), &["invalid_time_zone"]).await;
    }
}

#[sqlx::test(migrations = "./migrations")]
async fn a_recipe_of_the_pinned_graph_that_does_not_teach_the_level_is_rejected(pool: PgPool) {
    let sent = with(&entry(), "level", json!(3));
    rejected(&pool, &sent, &["recipe_does_not_teach_level"]).await;
    let other_skill = with(&entry(), "skill", json!("knife-cuts"));
    rejected(&pool, &other_skill, &["recipe_does_not_teach_level"]).await;
}

#[sqlx::test(migrations = "./migrations")]
async fn a_recipe_of_another_graph_version_or_unknown_to_the_graph_is_not_judged(pool: PgPool) {
    let (app, cook) = (server(pool), new_cook());
    let older_graph = json!({"recipeId": RECIPE, "graphVersion": "2026.01.01"});
    let unknown_recipe = json!({"recipeId": "recipe/not-in-this-graph", "graphVersion": FIXTURE_GRAPH});
    for dish in [older_graph, unknown_recipe] {
        let sent = with(&with(&entry(), "level", json!(3)), "dish", dish);
        assert_eq!(result(&app, &cook, &sent).await["status"], "accepted");
    }
}

#[sqlx::test(migrations = "./migrations")]
async fn every_problem_of_an_entry_is_named(pool: PgPool) {
    let sent = with(&with(&entry(), "weeklyGoal", json!(9)), "timeZone", json!("Nowhere/Town"));
    rejected(&pool, &sent, &["weekly_goal_out_of_range", "invalid_time_zone"]).await;
}

#[sqlx::test(migrations = "./migrations")]
async fn an_entry_carrying_a_quiz_result_xp_or_a_total_is_rejected_as_an_unknown_field(pool: PgPool) {
    for field in ["quizCorrect", "xp", "totalXp"] {
        rejected(&pool, &with(&entry(), field, json!(true)), &["unknown_field"]).await;
    }
    let dish = json!({"recipeId": RECIPE, "graphVersion": FIXTURE_GRAPH, "xp": 60});
    rejected(&pool, &with(&entry(), "dish", dish), &["unknown_field"]).await;
}

#[sqlx::test(migrations = "./migrations")]
async fn an_entry_that_cannot_be_read_is_rejected_and_the_rest_of_the_batch_is_stored(pool: PgPool) {
    let (app, cook) = (server(pool.clone()), new_cook());
    let unreadable = [
        with(&entry(), "dish", json!({"graphVersion": FIXTURE_GRAPH})),
        with(&entry(), "dish", json!({"recipeId": RECIPE})),
        with(&entry(), "outcome", json!("burnt")),
        with(&entry(), "cookedAt", json!("yesterday")),
        with(&entry(), "level", json!("one")),
    ];
    let mut sent = unreadable.to_vec();
    sent.push(entry());
    let replies = results(&app, &cook, &sent).await;
    for (entry, reply) in unreadable.iter().zip(&replies) {
        assert_eq!(reply, &json!({"id": entry["id"], "status": "rejected", "problems": ["unreadable_entry"]}));
    }
    assert_eq!(replies[5]["status"], "accepted");
    assert_eq!(stored_rows(&pool).await, 1);
}

#[sqlx::test(migrations = "./migrations")]
async fn an_entry_without_a_readable_id_is_answered_in_its_place(pool: PgPool) {
    let (app, cook) = (server(pool), new_cook());
    let no_id = with(&entry(), "id", json!("not-a-uuid"));
    let replies = results(&app, &cook, &[no_id, entry()]).await;
    assert_eq!(replies[0], json!({"id": null, "status": "rejected", "problems": ["unreadable_entry"]}));
    assert_eq!(replies[1]["status"], "accepted");
}

#[sqlx::test(migrations = "./migrations")]
async fn a_skill_or_level_the_catalog_does_not_know_is_accepted(pool: PgPool) {
    let (app, cook) = (server(pool), new_cook());
    let other_graph = json!({"recipeId": RECIPE, "graphVersion": "older"});
    let unknown_skill = with(&with(&entry(), "skill", json!("basket-weaving")), "dish", other_graph.clone());
    let unknown_level = with(&with(&entry(), "level", json!(9)), "dish", other_graph);
    for sent in [unknown_skill, unknown_level] {
        assert_eq!(result(&app, &cook, &sent).await["status"], "accepted");
    }
}
