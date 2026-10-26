//! The entry id is the idempotency key: a retry is safe, and the first accepted copy wins.

use super::*;
use serde_json::json;

#[sqlx::test(migrations = "./migrations")]
async fn the_same_batch_twice_is_accepted_then_a_duplicate_and_stored_once(pool: PgPool) {
    let (app, cook) = (server(pool.clone()), new_cook());
    let entries = [entry(), entry()];
    let first = results(&app, &cook, &entries).await;
    let again = results(&app, &cook, &entries).await;
    for (sent, (first, again)) in entries.iter().zip(first.iter().zip(&again)) {
        assert_eq!(first, &json!({"id": sent["id"], "status": "accepted"}));
        assert_eq!(again, &json!({"id": sent["id"], "status": "duplicate"}));
    }
    assert_eq!(stored_rows(&pool).await, 2);
}

#[sqlx::test(migrations = "./migrations")]
async fn the_same_cook_written_differently_is_still_a_duplicate(pool: PgPool) {
    let (app, cook) = (server(pool.clone()), new_cook());
    let sent = entry();
    let same_instant = with(&sent, "cookedAt", json!("2026-10-20T20:42:00+02:00"));
    assert_eq!(result(&app, &cook, &sent).await["status"], "accepted");
    assert_eq!(result(&app, &cook, &same_instant).await["status"], "duplicate");
    assert_eq!(stored_rows(&pool).await, 1);
}

#[sqlx::test(migrations = "./migrations")]
async fn a_finished_cook_then_a_stopped_copy_is_a_conflict_and_the_finished_one_stays(pool: PgPool) {
    let finished = entry();
    let stopped = with(&finished, "outcome", json!("stoppedEarly"));
    assert_eq!(first_copy_wins(&pool, &finished, &stopped).await, "finished");
}

#[sqlx::test(migrations = "./migrations")]
async fn a_stopped_cook_then_a_finished_copy_is_a_conflict_and_the_stopped_one_stays(pool: PgPool) {
    let finished = entry();
    let stopped = with(&finished, "outcome", json!("stoppedEarly"));
    assert_eq!(first_copy_wins(&pool, &stopped, &finished).await, "stoppedEarly");
}

/// Sends two copies of one id with different bodies, checks the second is a conflict that changed nothing, and
/// returns the stored outcome.
async fn first_copy_wins(pool: &PgPool, first: &Value, second: &Value) -> String {
    let (app, cook) = (server(pool.clone()), new_cook());
    let accepted = result(&app, &cook, first).await;
    let stored_before = stored_row(pool).await;
    let conflict = result(&app, &cook, second).await;
    let expected = [json!({"id": first["id"], "status": "accepted"}), json!({"id": first["id"], "status": "conflict"})];
    assert_eq!([accepted, conflict], expected);
    assert_eq!(stored_row(pool).await, stored_before);
    stored_before["outcome"].as_str().unwrap().to_string()
}

/// The one stored row as JSON, every column included.
async fn stored_row(pool: &PgPool) -> Value {
    let query = "select to_jsonb(entry) from tomi.cook_entries entry";
    sqlx::query_scalar(query).fetch_one(pool).await.unwrap()
}

#[sqlx::test(migrations = "./migrations")]
async fn copies_inside_one_batch_count_once_and_a_different_copy_is_a_conflict(pool: PgPool) {
    let (app, cook) = (server(pool.clone()), new_cook());
    let sent = entry();
    let changed = with(&sent, "weeklyGoal", json!(4));
    let statuses: Vec<Value> = results(&app, &cook, &[sent.clone(), sent, changed]).await;
    let statuses: Vec<&Value> = statuses.iter().map(|result| &result["status"]).collect();
    assert_eq!(statuses, [&json!("accepted"), &json!("duplicate"), &json!("conflict")]);
    assert_eq!(stored_rows(&pool).await, 1);
}

#[sqlx::test(migrations = "./migrations")]
async fn two_cooks_using_the_same_entry_id_do_not_collide(pool: PgPool) {
    let app = server(pool.clone());
    let (first_cook, second_cook) = (new_cook(), new_cook());
    let sent = entry();
    let other = with(&sent, "outcome", json!("stoppedEarly"));
    assert_eq!(result(&app, &first_cook, &sent).await["status"], "accepted");
    assert_eq!(result(&app, &second_cook, &other).await["status"], "accepted");
    assert_eq!(stored_rows(&pool).await, 2);
}

#[sqlx::test(migrations = "./migrations")]
async fn the_stored_row_holds_the_entry_with_its_own_time_zone(pool: PgPool) {
    let (app, cook) = (server(pool.clone()), new_cook());
    let sent = with(&entry(), "timeZone", json!("Asia/Tokyo"));
    result(&app, &cook, &sent).await;
    let row = stored_row(&pool).await;
    let expected = json!({
        "id": sent["id"], "time_zone": "Asia/Tokyo", "recipe_id": RECIPE, "graph_version": FIXTURE_GRAPH,
        "lesson_id": null, "skill": "eggs", "level": 1, "outcome": "finished", "weekly_goal": 3, "schema_version": 1,
    });
    for (column, value) in expected.as_object().unwrap() {
        assert_eq!(&row[column], value, "{column}");
    }
    let query = "select cooked_at = '2026-10-20T18:42:00Z'::timestamptz from tomi.cook_entries";
    assert!(sqlx::query_scalar::<_, bool>(query).fetch_one(&pool).await.unwrap());
}

#[sqlx::test(migrations = "./migrations")]
async fn an_old_development_log_names_its_lesson(pool: PgPool) {
    let (app, cook) = (server(pool.clone()), new_cook());
    let sent = with(&entry(), "dish", json!({"lessonId": "eggs-scrambled-1"}));
    assert_eq!(result(&app, &cook, &sent).await["status"], "accepted");
    let lesson: Option<String> =
        sqlx::query_scalar("select lesson_id from tomi.cook_entries").fetch_one(&pool).await.unwrap();
    assert_eq!(lesson.as_deref(), Some("eggs-scrambled-1"));
}
