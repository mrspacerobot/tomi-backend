//! The adapted recipe and its changes as the app reads them, with names in the cook's language.

use super::{named_option, suggestion_swaps, NamedOption, SuggestionSwap};
use crate::graph::Graph;
use crate::kernel::language::Language;
use crate::recipe::{AdaptedStep, OmittedPart, PreparedIngredient, PreparedRecipe, RecipeAmount};
use crate::suggestions::Candidate;
use serde::Serialize;

/// The recipe made for this cook: what to buy and measure, and what to do.
#[derive(Debug, Serialize)]
pub struct AdaptedRecipe {
    pub yields: RecipeAmount,
    pub ingredients: Vec<AdaptedIngredient>,
    pub steps: Vec<AdaptedStep>,
}

#[derive(Debug, Serialize)]
pub struct AdaptedIngredient {
    pub part: String,
    #[serde(flatten)]
    pub food: NamedOption,
    pub amount: RecipeAmount,
    pub forms: Vec<NamedOption>,
}

/// What differs from the graph's recipe.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecipeChanges {
    pub swaps: Vec<SuggestionSwap>,
    pub left_out: Vec<LeftOutPart>,
}

#[derive(Debug, Serialize)]
pub struct LeftOutPart {
    pub part: String,
    /// A name the cook can read, from the food the part would have held.
    pub name: String,
    pub ingredients: Vec<NamedOption>,
}

/// The recipe written for the cook and what differs from the graph's own.
pub fn adaptation(candidate: &Candidate, graph: &Graph, language: &Language) -> Option<(AdaptedRecipe, RecipeChanges)> {
    Some((adapted_recipe(&candidate.recipe, graph, language)?, changes(candidate, graph, language)?))
}

fn adapted_recipe(adapted: &PreparedRecipe, graph: &Graph, language: &Language) -> Option<AdaptedRecipe> {
    let lines = adapted.lines.iter().map(|line| adapted_ingredient(line, graph, language));
    Some(AdaptedRecipe {
        yields: adapted.yields.clone(),
        ingredients: lines.collect::<Option<Vec<_>>>()?,
        steps: adapted.steps.clone(),
    })
}

fn adapted_ingredient(line: &PreparedIngredient, graph: &Graph, language: &Language) -> Option<AdaptedIngredient> {
    let forms = line.forms.iter().map(|form| named_option(form, graph, language));
    Some(AdaptedIngredient {
        part: line.part.clone(),
        food: named_option(&line.food, graph, language)?,
        amount: line.amount.clone(),
        forms: forms.collect::<Option<Vec<_>>>()?,
    })
}

fn changes(candidate: &Candidate, graph: &Graph, language: &Language) -> Option<RecipeChanges> {
    let left_out = candidate.recipe.left_out.iter().map(|part| left_out_part(part, graph, language));
    Some(RecipeChanges {
        swaps: suggestion_swaps(&candidate.swaps, graph, language)?,
        left_out: left_out.collect::<Option<Vec<_>>>()?,
    })
}

fn left_out_part(part: &OmittedPart, graph: &Graph, language: &Language) -> Option<LeftOutPart> {
    let foods = part.foods.iter().map(|food| named_option(food, graph, language));
    let ingredients = foods.collect::<Option<Vec<_>>>()?;
    let name = ingredients.first()?.name.clone();
    Some(LeftOutPart { part: part.part.clone(), name, ingredients })
}
