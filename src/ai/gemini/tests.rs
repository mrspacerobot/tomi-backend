//! The provider boundary uses local HTTP fixtures, never a live model.

use super::*;
use axum::http::StatusCode;
use axum::routing::post;
use axum::{Json, Router};
use serde_json::json;
use tokio::net::TcpListener;
use tokio::task::JoinHandle;

struct Provider {
    ai: Gemini,
    server: JoinHandle<()>,
}

impl Drop for Provider {
    fn drop(&mut self) {
        self.server.abort();
    }
}

async fn provider(router: Router) -> Provider {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let server = tokio::spawn(async move { axum::serve(listener, router).await.unwrap() });
    let mut ai = Gemini::new("private-test-key".into(), "test-model".into()).unwrap();
    ai.api_root = format!("http://{address}");
    Provider { ai, server }
}

fn photo() -> crate::scans::photo::Photo {
    crate::scans::photo::Photo::parse(Some("image/jpeg"), vec![0xFF, 0xD8, 0xFF, 0xD9]).unwrap()
}

async fn schema_provider(Json(body): Json<Value>) -> (StatusCode, Json<Value>) {
    let ingredient =
        &body["generationConfig"]["response_schema"]["properties"]["items"]["items"]["properties"]["ingredient"];
    if ingredient.get("enum").is_some() {
        return (StatusCode::BAD_REQUEST, Json(json!({"error": {"status": "INVALID_ARGUMENT"}})));
    }
    let text = body["contents"][0]["parts"][0]["text"].as_str().unwrap();
    assert!(text.contains("item-599"));
    (StatusCode::OK, Json(json!({"candidates": [{"content": {"parts": [{"text": "{\"items\": []}"}]}}]})))
}

#[tokio::test]
async fn a_large_graph_vocabulary_does_not_make_the_provider_reject_a_scan() {
    let provider = provider(Router::new().fallback(post(schema_provider))).await;
    let ingredients: Vec<_> =
        (0..600).map(|index| crate::kernel::ids::SavedId::parse(&format!("item-{index}")).unwrap()).collect();
    let photo = photo();
    let result = provider.ai.read_fridge(FridgeQuestion { photo: &photo, ingredients: &ingredients }).await;
    assert!(result.unwrap().is_empty());
}

#[tokio::test]
async fn a_provider_rejection_preserves_status_without_keeping_its_sensitive_body() {
    let reply = json!({"error": {"status": "INVALID_ARGUMENT", "message": "private-test-key private-photo"}});
    let router = Router::new().fallback(post(move || async { (StatusCode::BAD_REQUEST, Json(reply)) }));
    let provider = provider(router).await;
    let error = provider.ai.generate(json!({})).await.unwrap_err();
    assert_eq!(error, AiError::Rejected { status: 400 });
    assert_eq!(error.code(), "ai_provider_invalid_request");
    assert!(!format!("{error:?}").contains("private"));
}

#[tokio::test]
async fn a_non_json_provider_response_has_a_different_code_from_invalid_model_content() {
    let router = Router::new().fallback(post(|| async { "not json private-photo" }));
    let provider = provider(router).await;
    assert_eq!(provider.ai.generate(json!({})).await.unwrap_err(), AiError::Unreadable);
    assert_eq!(parse_ranking("not json").unwrap_err(), AiError::InvalidContent);
}

#[tokio::test(start_paused = true)]
async fn a_stalled_provider_is_a_timeout_not_a_connection_error() {
    let router = Router::new().fallback(post(std::future::pending::<Json<Value>>));
    let provider = provider(router).await;
    let error = provider.ai.generate(json!({})).await.unwrap_err();
    assert_eq!(error, AiError::TimedOut);
    assert_eq!(error.code(), "ai_timeout");
    assert_eq!(error.provider_status(), None);
}

#[tokio::test]
async fn an_invalid_connection_is_not_a_provider_rejection() {
    let mut ai = Gemini::new("private-test-key".into(), "test-model".into()).unwrap();
    ai.api_root = "http://[".into();
    let error = ai.generate(json!({})).await.unwrap_err();
    assert_eq!(error, AiError::Unreachable);
    assert_eq!(error.provider_status(), None);
}

#[test]
fn incomplete_and_blocked_responses_have_distinct_codes() {
    let limited = json!({"candidates": [{"finishReason": "MAX_TOKENS", "content": {"parts": [{"text": "partial"}]}}]});
    assert_eq!(response_text(&limited).unwrap_err(), AiError::OutputLimit);
    assert_eq!(response_text(&json!({"promptFeedback": {"blockReason": "SAFETY"}})).unwrap_err(), AiError::Blocked);
    assert_eq!(response_text(&json!({"candidates": [{"finishReason": "SAFETY"}]})).unwrap_err(), AiError::Blocked);
    assert_eq!(response_text(&json!({})).unwrap_err(), AiError::EmptyResponse);
}

#[test]
fn operational_errors_have_stable_specific_codes() {
    let cases = [
        (AiError::TimedOut, "ai_timeout"),
        (AiError::Unreachable, "ai_transport_error"),
        (AiError::Rejected { status: 401 }, "ai_provider_auth_error"),
        (AiError::Rejected { status: 403 }, "ai_provider_auth_error"),
        (AiError::Rejected { status: 404 }, "ai_provider_model_not_found"),
        (AiError::Rejected { status: 429 }, "ai_provider_rate_limited"),
        (AiError::Rejected { status: 503 }, "ai_provider_unavailable"),
    ];
    for (error, code) in cases {
        assert_eq!(error.code(), code);
    }
}
