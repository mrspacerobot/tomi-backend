//! The AI functions, behind one interface so the provider can change.
//!
//! A model only ever reads and ranks. It never adds an ingredient, a diet
//! or a lesson, and its answer is checked by `domain` before anyone sees it.

use crate::kernel::ids::SavedId;
use crate::profile::Profile;
use crate::scans::fridge::ModelItem;
use crate::scans::photo::Photo;
use crate::special::{Candidate, Ranked};
use std::future::Future;
use std::pin::Pin;

mod gemini;
mod prompts;
#[cfg(test)]
mod tests;

pub use gemini::Gemini;

pub type AiFuture<'a, T> = Pin<Box<dyn Future<Output = Result<T, AiError>> + Send + 'a>>;

#[derive(Debug, thiserror::Error)]
pub enum AiError {
    #[error("no AI model is configured")]
    NotConfigured,
    #[error("the model could not be reached")]
    Unreachable,
    #[error("the model's answer could not be read")]
    Unreadable,
}

/// A fridge photo and the ingredient ids the model may name.
pub struct FridgeQuestion<'a> {
    pub photo: &'a Photo,
    pub ingredients: &'a [SavedId],
}

/// The short list to rank, and what the cook told Tomi about themselves.
pub struct RankQuestion<'a> {
    pub candidates: &'a [Candidate],
    pub profile: &'a Profile,
}

pub trait Ai: Send + Sync {
    /// Stored with each kept photo, so the dataset says what labelled it.
    fn model_name(&self) -> &str;
    fn read_fridge<'a>(&'a self, question: FridgeQuestion<'a>) -> AiFuture<'a, Vec<ModelItem>>;
    fn rank_lessons<'a>(&'a self, question: RankQuestion<'a>) -> AiFuture<'a, Vec<Ranked>>;
}

/// No model configured: the scan is unavailable and specials fall back to
/// ranking by what the cook has, as the app does without AI.
pub struct NoAi;

impl Ai for NoAi {
    fn model_name(&self) -> &str {
        "none"
    }

    fn read_fridge<'a>(&'a self, _question: FridgeQuestion<'a>) -> AiFuture<'a, Vec<ModelItem>> {
        Box::pin(async { Err(AiError::NotConfigured) })
    }

    fn rank_lessons<'a>(&'a self, _question: RankQuestion<'a>) -> AiFuture<'a, Vec<Ranked>> {
        Box::pin(async { Err(AiError::NotConfigured) })
    }
}
