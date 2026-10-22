//! The steps of a recipe as this cook reads them: the foods written in, left-out clauses gone, empty steps dropped.

use super::error::AdaptError;
use super::lines::UsedLine;
use super::source::SourceRecipe;
use super::AdaptedStep;
use crate::graph::foods::english_name;
use crate::graph::steps::{self, Mention, RenderContext, StepError};
use crate::graph::Graph;
use std::collections::BTreeSet;

/// What the steps are written from.
pub struct StepKit<'a> {
    pub graph: &'a Graph,
    pub lines: &'a [UsedLine<'a>],
    pub left_out: &'a BTreeSet<String>,
}

pub fn adapted_steps(kit: &StepKit, source: &SourceRecipe) -> Result<Vec<AdaptedStep>, AdaptError> {
    let mut kept = Vec::new();
    for step in &source.steps {
        let text = written(kit, &step.text)?;
        if text.is_empty() {
            continue;
        }
        let (time, temperature) = (step.time.clone(), step.temperature.clone());
        let safety_points = step.safety_points.clone();
        kept.push(AdaptedStep { number: kept.len() + 1, text, time, temperature, safety_points });
    }
    Ok(kept)
}

fn written(kit: &StepKit, text: &str) -> Result<String, AdaptError> {
    let words = &kit.graph.serving.step_text.proper_name_words;
    let context = RenderContext { left_out: kit.left_out, proper_name_words: words };
    let pieces = steps::parse(text)?;
    Ok(steps::render(&pieces, &context, |mention| name_of(kit, mention))?)
}

/// The English name of the food the cook uses where the mention stands.
fn name_of(kit: &StepKit, mention: &Mention) -> Result<String, StepError> {
    let found = kit.lines.iter().find(|line| stands_for(line, mention));
    let Some(line) = found else {
        return Err(StepError::UnknownPlaceholder(mention.part.clone()));
    };
    let Some(food) = kit.graph.thing(line.food) else {
        return Err(StepError::UnknownFood(line.food.to_string()));
    };
    english_name(line.food, food, mention.plural)
}

fn stands_for(line: &UsedLine, mention: &Mention) -> bool {
    if line.source.part != mention.part {
        return false;
    }
    match &mention.ingredient {
        Some(slug) => line.source.thing.rsplit('/').next() == Some(slug.as_str()),
        None => true,
    }
}
