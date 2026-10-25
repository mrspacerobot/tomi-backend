//! AI failures stay useful in logs without exposing the cook's data.

use crate::helpers::*;
use axum::http::StatusCode;
use serde_json::json;
use sqlx::PgPool;
use std::io::{self, Write};
use std::sync::{Arc, Mutex, Once};
use tomi_api::ai::{Ai, AiError, AiFuture, FridgeQuestion, NoAi, RankQuestion};
use tomi_api::scans::fridge::ModelItem;
use tomi_api::suggestions::Ranked;
use tracing::instrument::WithSubscriber;

#[derive(Clone, Default)]
struct LogBuffer(Arc<Mutex<Vec<u8>>>);

impl LogBuffer {
    fn subscriber(&self) -> impl tracing::Subscriber + Send + Sync + 'static {
        // A permanent registry keeps callsite interest enabled while parallel scoped captures come and go.
        static LOGGING: Once = Once::new();
        LOGGING.call_once(|| tracing::subscriber::set_global_default(tracing_subscriber::registry()).unwrap());
        let writer = self.clone();
        tracing_subscriber::fmt().json().with_writer(move || writer.clone()).finish()
    }

    fn text(&self) -> String {
        String::from_utf8(self.0.lock().unwrap().clone()).unwrap()
    }
}

impl Write for LogBuffer {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

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

    fn rank_recipes<'a>(&'a self, _question: RankQuestion<'a>) -> AiFuture<'a, Vec<Ranked>> {
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

#[sqlx::test(migrations = "./migrations")]
async fn failed_recipe_ranking_logs_the_cause_and_keeps_the_overlap_fallback(pool: PgPool) {
    let ai = Arc::new(FailingAi(AiError::Rejected { status: 429 }));
    let setup = Setup { ai, library: library_without_prerequisites(), ..Setup::default() };
    let app = server_with(pool, setup);
    let cook = crate::suggestions::comfortable_cook(&app, json!({})).await;
    let buffer = LogBuffer::default();
    let reply = suggest(&app, &cook, json!({"ingredients": []})).with_subscriber(buffer.subscriber()).await;
    assert_eq!(reply.status, StatusCode::OK);
    assert_eq!(reply.body["rankedBy"], "overlap");
    let logs = buffer.text();
    assert!(logs.contains(r#""operation":"recipe_ranking""#), "{logs}");
    assert!(logs.contains(r#""error_code":"ai_provider_rate_limited""#), "{logs}");
    assert!(!logs.contains(&cook));
}
