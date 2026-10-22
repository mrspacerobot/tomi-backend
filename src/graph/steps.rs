//! The text of a recipe step: read its `{part}` placeholders and `[part: ...]` clauses, write what a cook reads.
//!
//! The format is the graph repository's `docs/recipe-steps.md`; its `tools/step_text.py` is the reference and the
//! release's `step_text_cases` prove this code writes the same words.

mod parse;
mod render;
#[cfg(test)]
mod tests;

pub use parse::{parse, Clause, Mention, Piece};
pub use render::{render, RenderContext};

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum StepError {
    #[error("the step text is malformed: {0}")]
    Malformed(&'static str),
    #[error("the placeholder `{0}` names no kept line of the recipe")]
    UnknownPlaceholder(String),
    #[error("the food `{0}` has no plural name")]
    MissingPlural(String),
    #[error("the graph does not name the food `{0}` in English")]
    UnknownFood(String),
}
