//! AI failures stay useful in logs without exposing the cook's data.

use crate::helpers::*;
use crate::logs::LogBuffer;
use axum::http::StatusCode;
use sqlx::PgPool;
use std::sync::Arc;
use tomi_api::ai::{Ai, AiError, AiFuture, FridgeQuestion, NoAi};
use tomi_api::scans::fridge::ModelItem;
use tracing::instrument::WithSubscriber;

#[sqlx::test(migrations = "./migrations")]
async fn a_scan_logs_the_cause_and_model_without_changing_the_public_error(pool: PgPool) {
    let app = server_with(pool, Setup { ai: Arc::new(NoAi), ..Setup::default() });
    let buffer = LogBuffer::default();
    let reply = scan(&app, &new_cook(), jpeg()).with_subscriber(buffer.subscriber()).await;
    assert_eq!(reply.status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(reply.body["error"]["code"], "ai_unavailable");
    let logs = buffer.text();
    assert!(logs.contains("ai_not_configured"), "{logs}");
    assert!(logs.contains(r#""model":"none""#), "{logs}");
    assert!(logs.contains(r#""operation":"fridge_scan""#), "{logs}");
    assert!(logs.contains(reply.headers["x-request-id"].to_str().unwrap()), "{logs}");
    assert!(!logs.contains("/9j/"));
}

struct FailingAi(AiError);

impl Ai for FailingAi {
    fn model_name(&self) -> &str {
        "fixture-model"
    }

    fn read_fridge<'a>(&'a self, _question: FridgeQuestion<'a>) -> AiFuture<'a, Vec<ModelItem>> {
        Box::pin(async { Err(self.0.clone()) })
    }
}

#[sqlx::test(migrations = "./migrations")]
async fn a_rejected_scan_logs_http_status_and_its_specific_failure_code(pool: PgPool) {
    let ai = Arc::new(FailingAi(AiError::Rejected { status: 400 }));
    let app = server_with(pool, Setup { ai, ..Setup::default() });
    let buffer = LogBuffer::default();
    let reply = scan(&app, &new_cook(), jpeg()).with_subscriber(buffer.subscriber()).await;
    assert_eq!(reply.body["error"]["code"], "ai_unavailable");
    let logs = buffer.text();
    assert!(logs.contains(r#""error_code":"ai_provider_invalid_request""#), "{logs}");
    assert!(logs.contains(r#""provider_http_status":400"#), "{logs}");
    assert!(logs.contains(r#""model":"fixture-model""#), "{logs}");
    assert!(!logs.contains("private-test-key"));
}
