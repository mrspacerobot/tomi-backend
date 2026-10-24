//! A graph recipe made for one cook, in code: the foods they use written into the steps, amounts scaled by the
//! swap's ratio, left-out parts and the steps that were only about them gone, then checked again.
//!
//! A model never writes or rewrites any of this. A recipe that cannot be adapted cleanly is an error here and is
//! never offered.

mod check;
mod error;
mod lines;
mod source;
mod steps;
#[cfg(test)]
mod tests;

pub use error::AdaptError;
pub use source::{Amount, Temperature, Time, Unit};

use crate::graph::serving::{PlannedSwap, RecipePlan};
use crate::graph::Graph;
use lines::{adapted_line, left_out_parts, used_lines};
use serde::Serialize;
use source::SourceRecipe;
use std::collections::BTreeSet;
use steps::{adapted_steps, StepKit};

/// The recipe for this cook, with graph ids where a food is meant. Names are put in when it is shown.
#[derive(Debug, Clone)]
pub struct Adapted {
    pub yields: Amount,
    pub lines: Vec<AdaptedLine>,
    pub steps: Vec<AdaptedStep>,
    /// The swaps of lines that stay, in the recipe's order.
    pub swaps: Vec<PlannedSwap>,
    pub left_out: Vec<LeftOutPart>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptedLine {
    pub part: String,
    /// The graph id of the food in use: the recipe's own, or its stand-in.
    pub food: String,
    pub amount: Amount,
    pub forms: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdaptedStep {
    /// Counted again from one, so a dropped step leaves no gap.
    pub number: usize,
    pub text: String,
    pub time: Option<Time>,
    pub temperature: Option<Temperature>,
    pub safety_points: Vec<String>,
}

/// A part the cook goes without, and the foods the recipe would have used for it.
#[derive(Debug, Clone, PartialEq)]
pub struct LeftOutPart {
    pub part: String,
    pub foods: Vec<String>,
}

/// What a recipe is adapted against: the graph and the facts the cook avoids.
pub struct Basis<'a> {
    pub graph: &'a Graph,
    pub avoided: &'a BTreeSet<String>,
}

pub fn adapt(recipe_id: &str, plan: &RecipePlan, basis: &Basis) -> Result<Adapted, AdaptError> {
    let graph = basis.graph;
    let Some(served) = graph.serving.recipes.get(recipe_id) else {
        return Err(AdaptError::UnknownRecipe);
    };
    let source = read_source(graph, recipe_id)?;
    let used = used_lines(&source, served, plan)?;
    let left_out = left_out_parts(plan);
    check::recheck(served, &used, &check::Rules { left_out: &left_out, avoided: basis.avoided })?;
    let steps = adapted_steps(&StepKit { graph, lines: &used, left_out: &left_out }, &source)?;
    let lines = used.iter().map(adapted_line).collect::<Result<Vec<_>, _>>()?;
    let swaps = used.iter().filter_map(|line| line.swap.cloned()).collect();
    let left_out = left_out_with_foods(&source, &left_out);
    Ok(Adapted { yields: source.yields.clone(), lines, steps, swaps, left_out })
}

fn read_source(graph: &Graph, recipe_id: &str) -> Result<SourceRecipe, AdaptError> {
    let Some(thing) = graph.thing(recipe_id) else {
        return Err(AdaptError::UnknownRecipe);
    };
    serde_json::from_value(thing.clone()).map_err(|_| AdaptError::Unreadable)
}

fn left_out_with_foods(source: &SourceRecipe, left_out: &BTreeSet<String>) -> Vec<LeftOutPart> {
    left_out
        .iter()
        .map(|part| {
            let foods = source.ingredients.iter().filter(|line| &line.part == part);
            LeftOutPart { part: part.clone(), foods: foods.map(|line| line.thing.clone()).collect() }
        })
        .collect()
}
