//! Food identification for fridge scans, behind one interface so the provider can change.
//!
//! The model only identifies foods in scans; all recipe decisions stay in code.

use crate::kernel::ids::SavedId;
use crate::scans::fridge::ModelItem;
use crate::scans::photo::Photo;
use std::future::Future;
use std::pin::Pin;

mod error;
mod gemini;
mod prompts;
#[cfg(test)]
mod tests;

pub use error::AiError;
pub use gemini::Gemini;

pub type AiFuture<'a, T> = Pin<Box<dyn Future<Output = Result<T, AiError>> + Send + 'a>>;

/// A fridge photo and the ingredient ids the model may name.
pub struct FridgeQuestion<'a> {
    pub photo: &'a Photo,
    pub ingredients: &'a [SavedId],
}

pub trait Ai: Send + Sync {
    /// Stored with each kept photo, so the dataset says what labelled it.
    fn model_name(&self) -> &str;
    fn read_fridge<'a>(&'a self, question: FridgeQuestion<'a>) -> AiFuture<'a, Vec<ModelItem>>;
}

/// No model configured: photo scanning is unavailable.
pub struct NoAi;

impl Ai for NoAi {
    fn model_name(&self) -> &str {
        "none"
    }

    fn read_fridge<'a>(&'a self, _question: FridgeQuestion<'a>) -> AiFuture<'a, Vec<ModelItem>> {
        Box::pin(async { Err(AiError::NotConfigured) })
    }
}
