//! Google's Gemini API, called over plain HTTPS.
//!
//! The key goes in a header, never in the URL, so it cannot end up in an
//! access log. A photo is sent in the request body and never logged.

use super::{prompts, Ai, AiError, AiFuture, FridgeQuestion, RankQuestion};
use crate::scans::fridge::ModelItem;
use crate::special::Ranked;
use serde::Deserialize;
use serde_json::Value;
use std::time::Duration;

#[cfg(test)]
mod tests;

const API_ROOT: &str = "https://generativelanguage.googleapis.com/v1beta/models";
/// Reading a photo can take a while; a cook will not wait much longer.
const TIMEOUT: Duration = Duration::from_secs(30);

pub struct Gemini {
    http: reqwest::Client,
    api_key: String,
    model: String,
    api_root: String,
}

#[derive(Deserialize)]
struct FridgeAnswer {
    items: Vec<ModelItem>,
}

#[derive(Deserialize)]
struct RankAnswer {
    ranking: Vec<RankChoice>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RankChoice {
    recipe_id: String,
    reason: String,
}

impl Gemini {
    pub fn new(api_key: String, model: String) -> Result<Self, reqwest::Error> {
        let http = reqwest::Client::builder().timeout(TIMEOUT).build()?;
        Ok(Gemini { http, api_key, model, api_root: API_ROOT.into() })
    }

    /// Sends a request and returns the model's JSON text.
    async fn generate(&self, body: Value) -> Result<String, AiError> {
        let url = format!("{}/{}:generateContent", self.api_root, self.model);
        let request = self.http.post(url).header("x-goog-api-key", &self.api_key).json(&body);
        let response = request.send().await.map_err(AiError::transport)?;
        let response = response.error_for_status().map_err(AiError::transport)?;
        let answer: Value = response.json().await.map_err(AiError::response)?;
        response_text(&answer)
    }
}

impl Ai for Gemini {
    fn model_name(&self) -> &str {
        &self.model
    }

    fn read_fridge<'a>(&'a self, question: FridgeQuestion<'a>) -> AiFuture<'a, Vec<ModelItem>> {
        let body = prompts::fridge_request(&question);
        Box::pin(async move {
            let text = self.generate(body).await?;
            let answer: FridgeAnswer = serde_json::from_str(&text).map_err(|_| AiError::InvalidContent)?;
            Ok(answer.items)
        })
    }

    fn rank_recipes<'a>(&'a self, question: RankQuestion<'a>) -> AiFuture<'a, Vec<Ranked>> {
        let body = prompts::rank_request(&question);
        Box::pin(async move {
            let text = self.generate(body).await?;
            parse_ranking(&text)
        })
    }
}

/// The text of the first candidate's first part, where the JSON answer is.
pub(super) fn answer_text(answer: &Value) -> Option<String> {
    let text = answer.pointer("/candidates/0/content/parts/0/text")?;
    text.as_str().map(ToString::to_string)
}

/// An id the model made up is ignored again when it is matched to candidates.
pub(super) fn parse_ranking(text: &str) -> Result<Vec<Ranked>, AiError> {
    let answer: RankAnswer = serde_json::from_str(text).map_err(|_| AiError::InvalidContent)?;
    let mut ranked = Vec::new();
    for choice in answer.ranking {
        ranked.push(Ranked { recipe_id: choice.recipe_id, reason: Some(choice.reason) });
    }
    Ok(ranked)
}

pub(super) fn response_text(answer: &Value) -> Result<String, AiError> {
    if answer.pointer("/promptFeedback/blockReason").is_some() {
        return Err(AiError::Blocked);
    }
    match answer.pointer("/candidates/0/finishReason").and_then(Value::as_str) {
        Some("MAX_TOKENS") => Err(AiError::OutputLimit),
        Some("SAFETY" | "RECITATION" | "BLOCKLIST" | "PROHIBITED_CONTENT" | "SPII") => Err(AiError::Blocked),
        _ => answer_text(answer).ok_or(AiError::EmptyResponse),
    }
}
