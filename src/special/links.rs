//! Which graph recipe a lesson is: the one that uses a technique its level teaches, with its exact ingredients.
//!
//! Nobody types this link. If it is not exactly one recipe, the lesson is not offered, and the reason is given.

use crate::graph::serving::ServedRecipe;
use crate::graph::Graph;
use crate::library::{Lesson, Library};
use std::collections::BTreeSet;

#[cfg(test)]
pub(crate) mod tests;

const INGREDIENT_PREFIX: &str = "ingredient/";

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum LinkProblem {
    #[error("its level teaches no technique the graph knows")]
    NoTechnique,
    #[error("no graph recipe uses a technique of its level with exactly its ingredients")]
    NoRecipe,
    #[error("several graph recipes fit: {0:?}")]
    Several(Vec<String>),
}

/// The graph recipe this lesson is.
pub fn recipe_for(lesson: &Lesson, graph: &Graph) -> Result<String, LinkProblem> {
    let techniques = graph.program.teaches(lesson.skill.as_str(), lesson.level);
    if techniques.is_empty() {
        return Err(LinkProblem::NoTechnique);
    }
    let mut fits = recipes_fitting(lesson, graph, &techniques);
    match fits.len() {
        0 => Err(LinkProblem::NoRecipe),
        1 => Ok(fits.remove(0)),
        _ => Err(LinkProblem::Several(fits)),
    }
}

/// A line for every lesson that matches no single recipe, so a start can say so.
pub fn unlinked(library: &Library, graph: &Graph) -> Vec<String> {
    let mut problems = Vec::new();
    for lesson in library.lessons.values() {
        if let Err(problem) = recipe_for(lesson, graph) {
            problems.push(format!("{}: {problem}", lesson.id));
        }
    }
    problems
}

/// The recipes that use one of these techniques and have exactly the lesson's ingredients.
fn recipes_fitting(lesson: &Lesson, graph: &Graph, techniques: &[String]) -> Vec<String> {
    let wanted: BTreeSet<String> = lesson.ingredient_ids().map(|id| format!("{INGREDIENT_PREFIX}{id}")).collect();
    let fits = |recipe: &ServedRecipe| {
        let uses_technique = recipe.techniques.iter().any(|technique| techniques.contains(technique));
        uses_technique && recipe.ingredients.iter().cloned().collect::<BTreeSet<_>>() == wanted
    };
    let found = graph.serving.recipes.iter().filter(|(_, recipe)| fits(recipe));
    found.map(|(recipe_id, _)| recipe_id.clone()).collect()
}
