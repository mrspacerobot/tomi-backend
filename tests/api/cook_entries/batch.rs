//! The batch as a whole: who may send it, when it is refused, the table it fills, the logs and the example.

use super::*;
use crate::logs::LogBuffer;
use axum::http::StatusCode;
use serde_json::json;
use tracing::instrument::WithSubscriber;

#[sqlx::test(migrations = "./migrations")]
async fn without_a_token_the_batch_is_unauthenticated(pool: PgPool) {
    let reply = send(&server(pool.clone()), None, &batch(&[entry()])).await;
    assert_eq!(reply.status, StatusCode::UNAUTHORIZED);
    assert_eq!(stored_rows(&pool).await, 0);
}

#[sqlx::test(migrations = "./migrations")]
async fn a_deleted_account_is_refused_and_nothing_is_stored(pool: PgPool) {
    let app = server(pool.clone());
    let subject = Uuid::new_v4();
    let token = token_with(&claims(subject));
    assert_eq!(get(&app, "/v1/profile", Some(&token)).await.status, StatusCode::OK);
    let delete = "update tomi.users set deleted_at = now() where auth_subject = $1";
    sqlx::query(delete).bind(subject).execute(&pool).await.unwrap();

    let reply = send(&app, Some(&token), &batch(&[entry()])).await;
    assert_eq!(reply.status, StatusCode::FORBIDDEN);
    assert_eq!(reply.body["error"]["code"], "account_deleted");
    assert_eq!(stored_rows(&pool).await, 0);
}

#[sqlx::test(migrations = "./migrations")]
async fn a_hundred_entries_are_read_and_one_more_refuses_the_batch(pool: PgPool) {
    let (app, cook) = (server(pool.clone()), new_cook());
    let too_many: Vec<Value> = (0..101).map(|_| entry()).collect();
    let refused = send(&app, Some(&cook), &batch(&too_many)).await;
    assert_eq!(refused.status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(refused.body["error"]["code"], "invalid_request");
    assert_eq!(stored_rows(&pool).await, 0);

    assert_eq!(results(&app, &cook, &too_many[..100]).await.len(), 100);
    assert_eq!(stored_rows(&pool).await, 100);
}

#[sqlx::test(migrations = "./migrations")]
async fn a_batch_without_a_readable_calendar_or_body_is_refused(pool: PgPool) {
    let (app, cook) = (server(pool.clone()), new_cook());
    let refused = [
        json!({"calendar": {"firstWeekday": 2}, "entries": [entry()]}),
        json!({"calendar": {"timeZone": "Mars/Olympus_Mons", "firstWeekday": 2}, "entries": [entry()]}),
        json!({"calendar": {"timeZone": "Europe/Amsterdam", "firstWeekday": 0}, "entries": [entry()]}),
        json!({"calendar": {"timeZone": "Europe/Amsterdam", "firstWeekday": 8}, "entries": [entry()]}),
        json!({"calendar": {"timeZone": "Europe/Amsterdam"}, "entries": [entry()]}),
        json!({"entries": [entry()]}),
        json!({"calendar": {"timeZone": "Europe/Amsterdam", "firstWeekday": 2}}),
        json!([entry()]),
    ];
    for body in refused {
        let reply = send(&app, Some(&cook), &body).await;
        assert_eq!(reply.status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
        assert_eq!(reply.body["error"]["code"], "invalid_request");
    }
    assert_eq!(stored_rows(&pool).await, 0);
}

#[sqlx::test(migrations = "./migrations")]
async fn the_cook_log_has_row_level_security_and_no_policies(pool: PgPool) {
    let query = "select relrowsecurity from pg_class where oid = 'tomi.cook_entries'::regclass";
    let secured: bool = sqlx::query_scalar(query).fetch_one(&pool).await.unwrap();
    assert!(secured);
    let query = "select count(*) from pg_policies where schemaname = 'tomi' and tablename = 'cook_entries'";
    let policies: i64 = sqlx::query_scalar(query).fetch_one(&pool).await.unwrap();
    assert_eq!(policies, 0);
}

#[sqlx::test(migrations = "./migrations")]
async fn a_conflict_logs_the_id_and_code_and_no_body_content_reaches_the_logs(pool: PgPool) {
    let (app, cook) = (server(pool), new_cook());
    let sent = with(&entry(), "timeZone", json!("Pacific/Chatham"));
    let changed = with(&sent, "outcome", json!("stoppedEarly"));
    let rejected = with(&with(&entry(), "skill", json!("Secret!")), "cookedAt", json!("2999-01-01T12:00:00Z"));
    let buffer = LogBuffer::default();
    let bodies = [batch(std::slice::from_ref(&sent)), batch(&[changed, rejected])];
    for body in &bodies {
        send(&app, Some(&cook), body).with_subscriber(buffer.subscriber()).await;
    }
    let logs = buffer.text();
    assert!(logs.contains(sent["id"].as_str().unwrap()), "{logs}");
    assert!(logs.contains(r#""code":"conflict""#), "{logs}");
    for content in
        ["Pacific/Chatham", "Europe/Amsterdam", RECIPE, "stoppedEarly", "Secret!", "2999-01-01", "2026-10-20"]
    {
        assert!(!logs.contains(content), "{content} reached the logs: {logs}");
    }
}

#[sqlx::test(migrations = "./migrations")]
async fn the_documented_example_is_what_the_server_answers(pool: PgPool) {
    let example: Value = serde_json::from_str(include_str!("../../../docs/examples/cook-entries.json")).unwrap();
    let reply = send(&server(pool), Some(&new_cook()), &example["request"]).await;
    assert_eq!(reply.status, StatusCode::OK, "{}", reply.body);
    assert_eq!(reply.body, example["response"]);
}
