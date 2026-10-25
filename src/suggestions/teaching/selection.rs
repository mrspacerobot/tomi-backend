//! The primary technique actually occurs in the recipe and belongs to its offered skill level.

use crate::graph::Graph;
use crate::suggestions::Candidate;

pub fn primary(candidate: &Candidate, graph: &Graph) -> Option<String> {
    let recipe = graph.serving.recipes.get(&candidate.recipe_id)?;
    let mut techniques = recipe.techniques.clone();
    techniques.sort();
    techniques.dedup();
    techniques.into_iter().find(|technique| matches_level(technique, candidate, graph))
}

fn matches_level(technique: &String, candidate: &Candidate, graph: &Graph) -> bool {
    let levels = graph.program.levels_for(std::slice::from_ref(technique));
    levels.contains(&(candidate.skill.to_string(), candidate.level))
}
