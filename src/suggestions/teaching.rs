//! Teaching introductions and quizzes read unchanged from the pinned graph.

use super::Candidate;
use crate::graph::{Channel, Graph};
use crate::kernel::language::Language;
use serde::Serialize;

mod quiz;
mod selection;
mod source;
#[cfg(test)]
mod tests;
mod tier;

use quiz::Question;
use source::{Introduction, Technique, TechniqueSource};
use tier::ContentTier;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Teaching {
    pub schema_version: u8,
    pub graph_version: String,
    pub graph_channel: &'static str,
    pub language: &'static str,
    pub technique: Technique,
    pub introduction: Introduction,
    pub quiz: Option<Vec<Question>>,
}

pub fn for_candidate(candidate: &Candidate, graph: &Graph, language: &Language) -> Option<Teaching> {
    if !compatible(candidate, graph) {
        return None;
    }
    let technique = selection::primary(candidate, graph)?;
    from_technique(&technique, graph, language)
}

fn compatible(candidate: &Candidate, graph: &Graph) -> bool {
    let Some(served) = graph.serving.recipes.get(&candidate.recipe_id) else {
        return false;
    };
    let Ok(source_tier) = serde_json::from_value::<ContentTier>(candidate.source_recipe["tier"].clone()) else {
        return false;
    };
    let Ok(served_tier) = serde_json::from_value::<ContentTier>(serde_json::Value::String(served.tier.clone())) else {
        return false;
    };
    candidate.recipe.swaps.is_empty()
        && candidate.recipe.left_out.is_empty()
        && source_tier.permitted(graph.channel)
        && served_tier.permitted(graph.channel)
}

fn from_technique(id: &str, graph: &Graph, language: &Language) -> Option<Teaching> {
    let thing = graph.thing(id)?;
    let source = permitted_source(thing, graph.channel, language)?;
    let quiz = quiz::read(thing, id, graph.channel);
    if !quiz_requirement_met(graph.channel, quiz.is_some()) {
        return None;
    }
    Some(Teaching {
        schema_version: 1,
        graph_version: graph.version.clone(),
        graph_channel: graph.channel.as_str(),
        language: "en",
        technique: source.identity(id, language),
        introduction: source.intro,
        quiz,
    })
}

fn permitted_source(thing: &serde_json::Value, channel: Channel, language: &Language) -> Option<TechniqueSource> {
    let source: TechniqueSource = serde_json::from_value(thing.clone()).ok()?;
    source.permitted(channel, language).then_some(source)
}

/// Production teaches a technique only with its reviewed quiz, so a gap in the content is refused, not shown.
/// A draft graph may still lack one while the content is being written.
fn quiz_requirement_met(channel: Channel, has_quiz: bool) -> bool {
    match channel {
        Channel::Draft => true,
        Channel::Production => has_quiz,
    }
}
