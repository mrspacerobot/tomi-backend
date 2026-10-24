//! The lines of a recipe as this cook makes it: kept, swapped (with the amount scaled) or left out.

use super::error::AdaptError;
use super::source::{Measure, RecipeAmount, SourceLine, SourceRecipe};
use super::PreparedIngredient;
use crate::graph::serving::{PlannedSwap, RecipePlan, ServedRecipe, SwapRatio};
use std::collections::BTreeSet;

/// A line that stays, with the food the cook uses for it.
pub struct UsedLine<'a> {
    /// The place of the line in the recipe and in the serving section, which list the same lines.
    pub index: usize,
    pub source: &'a SourceLine,
    pub food: &'a str,
    pub swap: Option<&'a PlannedSwap>,
}

/// The parts the plan leaves out.
pub fn left_out_parts(plan: &RecipePlan) -> BTreeSet<String> {
    plan.left_out.iter().cloned().collect()
}

/// Every line the plan keeps. The recipe and the serving section must list the same lines in the same order.
pub fn used_lines<'a>(
    source: &'a SourceRecipe,
    served: &ServedRecipe,
    plan: &'a RecipePlan,
) -> Result<Vec<UsedLine<'a>>, AdaptError> {
    if source.ingredients.len() != served.lines.len() {
        return Err(AdaptError::LinesDisagree);
    }
    let left_out = left_out_parts(plan);
    let mut used = Vec::new();
    for (index, (line, served_line)) in source.ingredients.iter().zip(&served.lines).enumerate() {
        if line.part != served_line.part || line.thing != served_line.thing {
            return Err(AdaptError::LinesDisagree);
        }
        if !left_out.contains(&line.part) {
            used.push(use_line(index, line, plan));
        }
    }
    Ok(used)
}

fn use_line<'a>(index: usize, line: &'a SourceLine, plan: &'a RecipePlan) -> UsedLine<'a> {
    let swap = plan.swaps.iter().find(|swap| swap.part == line.part && swap.original == line.thing);
    let food = match swap {
        Some(swap) => swap.instead.as_str(),
        None => line.thing.as_str(),
    };
    UsedLine { index, source: line, food, swap }
}

pub fn adapted_line(used: &UsedLine) -> Result<PreparedIngredient, AdaptError> {
    let amount = match used.swap.and_then(|swap| swap.ratio.as_ref()) {
        Some(ratio) => scaled(&used.source.amount, ratio)?,
        None => used.source.amount.clone(),
    };
    Ok(PreparedIngredient {
        part: used.source.part.clone(),
        food: used.food.to_string(),
        amount,
        forms: used.source.forms.clone(),
    })
}

/// The stand-in amount for the original amount. A ratio of one changes nothing, however it is measured; any other
/// ratio needs the line to be measured the same way, or there is no honest amount.
fn scaled(amount: &RecipeAmount, ratio: &SwapRatio) -> Result<RecipeAmount, AdaptError> {
    let usable =
        ratio.original > 0.0 && ratio.stand_in > 0.0 && ratio.original.is_finite() && ratio.stand_in.is_finite();
    if !usable {
        return Err(AdaptError::Ratio);
    }
    if ratio.stand_in == ratio.original {
        return Ok(amount.clone());
    }
    if amount.unit.measure() != Some(measure_of(&ratio.by)?) {
        return Err(AdaptError::Ratio);
    }
    let value = amount.value * ratio.stand_in / ratio.original;
    Ok(RecipeAmount { value: (value * 1000.0).round() / 1000.0, unit: amount.unit })
}

fn measure_of(by: &str) -> Result<Measure, AdaptError> {
    match by {
        "weight" => Ok(Measure::Weight),
        "volume" => Ok(Measure::Volume),
        "count" => Ok(Measure::Count),
        _ => Err(AdaptError::Ratio),
    }
}
