//! The primary technique actually occurs in the recipe and belongs to its offered skill level.

use crate::graph::Graph;
use crate::kernel::ids::SavedId;
use crate::suggestions::Candidate;
use std::collections::BTreeSet;

pub fn primary(candidate: &Candidate, graph: &Graph) -> Option<String> {
    let level = (candidate.skill.to_string(), candidate.level);
    primary_at(&candidate.recipe_id, &level, graph)
}

/// Every technique this recipe is taught with, one for each skill level it can be offered at.
pub fn taught(recipe_id: &str, graph: &Graph) -> BTreeSet<String> {
    let Some(recipe) = graph.serving.recipes.get(recipe_id) else {
        return BTreeSet::new();
    };
    let levels = graph.program.levels_for(&recipe.techniques);
    levels.iter().filter_map(|level| primary_at(recipe_id, level, graph)).collect()
}

fn primary_at(recipe_id: &str, level: &(String, u8), graph: &Graph) -> Option<String> {
    let recipe = graph.serving.recipes.get(recipe_id)?;
    let mut techniques = recipe.techniques.clone();
    techniques.sort();
    techniques.dedup();
    techniques.into_iter().find(|technique| matches_level(technique, level, graph))
}

fn matches_level(technique: &String, level: &(String, u8), graph: &Graph) -> bool {
    let Some(slug) = technique.strip_prefix("technique/") else {
        return false;
    };
    if SavedId::parse(slug).is_err() {
        return false;
    }
    let levels = graph.program.levels_for(std::slice::from_ref(technique));
    levels.contains(level)
}
