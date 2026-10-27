//! The request: authorization, validation, database protections, logs and the example.

use super::*;

#[sqlx::test(migrations = "./migrations")]
async fn a_single_cook_is_sent_and_answered_without_a_batch(pool: PgPool) {
    let (app, cook) = (server(pool.clone()), new_cook());
    let sent = entry();
    let request = json!({"calendar": {"timeZone": "Europe/Amsterdam", "firstWeekday": 2}, "entry": sent});
    let reply = send(&app, Some(&cook), &request).await;
    assert_eq!(reply.status, axum::http::StatusCode::OK);
    assert_eq!(reply.body["result"], json!({"id": sent["id"], "status": "accepted"}));
    assert_eq!(stored_rows(&pool).await, 1);
}
use crate::logs::LogBuffer;
use axum::http::StatusCode;
use serde_json::json;
use tracing::instrument::WithSubscriber;

#[sqlx::test(migrations = "./migrations")]
async fn without_a_token_the_request_is_unauthenticated(pool: PgPool) {
    let reply = send(&server(pool.clone()), None, &request(&entry())).await;
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

    let reply = send(&app, Some(&token), &request(&entry())).await;
    assert_eq!(reply.status, StatusCode::FORBIDDEN);
    assert_eq!(reply.body["error"]["code"], "account_deleted");
    assert_eq!(stored_rows(&pool).await, 0);
}

#[sqlx::test(migrations = "./migrations")]
async fn a_request_without_a_readable_calendar_or_entry_is_refused(pool: PgPool) {
    let (app, cook) = (server(pool.clone()), new_cook());
    let refused = [
        json!({"calendar": {"firstWeekday": 2}, "entry": entry()}),
        json!({"calendar": {"timeZone": "Mars/Olympus_Mons", "firstWeekday": 2}, "entry": entry()}),
        json!({"calendar": {"timeZone": "Europe/Amsterdam", "firstWeekday": 0}, "entry": entry()}),
        json!({"calendar": {"timeZone": "Europe/Amsterdam", "firstWeekday": 8}, "entry": entry()}),
        json!({"calendar": {"timeZone": "Europe/Amsterdam"}, "entry": entry()}),
        json!({"entry": entry()}),
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
    let bodies = [request(&sent), request(&changed), request(&rejected)];
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
    let reply = send(&server_at_request_time(pool), Some(&new_cook()), &example["request"]).await;
    assert_eq!(reply.status, StatusCode::OK, "{}", reply.body);
    assert_eq!(reply.body, example["response"]);
}
