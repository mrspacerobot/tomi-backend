//! The adapted recipe and its changes as the app reads them, with names in the cook's language.

use super::{named_thing, shown_swaps, NamedThing, ShownSwap};
use crate::graph::Graph;
use crate::kernel::language::Language;
use crate::special::adapted::{Adapted, AdaptedLine, AdaptedStep, Amount, LeftOutPart};
use crate::special::Candidate;
use serde::Serialize;

/// The recipe made for this cook: what to buy and measure, and what to do.
#[derive(Debug, Serialize)]
pub struct ShownRecipe {
    pub yields: Amount,
    pub ingredients: Vec<ShownLine>,
    pub steps: Vec<AdaptedStep>,
}

#[derive(Debug, Serialize)]
pub struct ShownLine {
    pub part: String,
    #[serde(flatten)]
    pub food: NamedThing,
    pub amount: Amount,
    pub forms: Vec<NamedThing>,
}

/// What differs from the graph's recipe.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Changes {
    pub swaps: Vec<ShownSwap>,
    pub left_out: Vec<ShownLeftOut>,
}

#[derive(Debug, Serialize)]
pub struct ShownLeftOut {
    pub part: String,
    /// A name the cook can read, from the food the part would have held.
    pub name: String,
    pub ingredients: Vec<NamedThing>,
}

/// The recipe written for the cook and what differs from the graph's own.
pub fn adaptation(candidate: &Candidate, graph: &Graph, language: &Language) -> Option<(ShownRecipe, Changes)> {
    Some((shown_recipe(&candidate.adapted, graph, language)?, changes(candidate, graph, language)?))
}

fn shown_recipe(adapted: &Adapted, graph: &Graph, language: &Language) -> Option<ShownRecipe> {
    let lines = adapted.lines.iter().map(|line| shown_line(line, graph, language));
    Some(ShownRecipe {
        yields: adapted.yields.clone(),
        ingredients: lines.collect::<Option<Vec<_>>>()?,
        steps: adapted.steps.clone(),
    })
}

fn shown_line(line: &AdaptedLine, graph: &Graph, language: &Language) -> Option<ShownLine> {
    let forms = line.forms.iter().map(|form| named_thing(form, graph, language));
    Some(ShownLine {
        part: line.part.clone(),
        food: named_thing(&line.food, graph, language)?,
        amount: line.amount.clone(),
        forms: forms.collect::<Option<Vec<_>>>()?,
    })
}

fn changes(candidate: &Candidate, graph: &Graph, language: &Language) -> Option<Changes> {
    let left_out = candidate.adapted.left_out.iter().map(|part| shown_left_out(part, graph, language));
    Some(Changes {
        swaps: shown_swaps(&candidate.swaps, graph, language)?,
        left_out: left_out.collect::<Option<Vec<_>>>()?,
    })
}

fn shown_left_out(part: &LeftOutPart, graph: &Graph, language: &Language) -> Option<ShownLeftOut> {
    let foods = part.foods.iter().map(|food| named_thing(food, graph, language));
    let ingredients = foods.collect::<Option<Vec<_>>>()?;
    let name = ingredients.first()?.name.clone();
    Some(ShownLeftOut { part: part.part.clone(), name, ingredients })
}
