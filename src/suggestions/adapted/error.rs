//! Why a recipe could not be adapted for a cook. Every one of them means the recipe is not offered.

use crate::graph::steps::StepError;

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum AdaptError {
    #[error("the graph has no such recipe")]
    UnknownRecipe,
    #[error("the recipe cannot be read")]
    Unreadable,
    #[error("the recipe's lines and the serving section's lines disagree")]
    LinesDisagree,
    #[error("a part that is not optional would be left out")]
    RequiredLeftOut,
    #[error("a food is not one the graph lists for its line")]
    UnlistedFood,
    #[error("a food is unsafe for what the cook avoids")]
    Unsafe,
    #[error("a swap's ratio cannot scale the line's amount")]
    Ratio,
    #[error(transparent)]
    Step(#[from] StepError),
}

impl AdaptError {
    /// A short stable word for the log. It never holds a food.
    pub fn reason(&self) -> &'static str {
        match self {
            AdaptError::UnknownRecipe => "unknown_recipe",
            AdaptError::Unreadable => "unreadable",
            AdaptError::LinesDisagree => "lines_disagree",
            AdaptError::RequiredLeftOut => "required_left_out",
            AdaptError::UnlistedFood => "unlisted_food",
            AdaptError::Unsafe => "unsafe",
            AdaptError::Ratio => "ratio",
            AdaptError::Step(StepError::Malformed(_)) => "step_malformed",
            AdaptError::Step(StepError::UnknownPlaceholder(_)) => "step_placeholder",
            AdaptError::Step(StepError::MissingPlural(_)) => "step_plural",
            AdaptError::Step(StepError::UnknownFood(_)) => "step_food",
        }
    }
}
