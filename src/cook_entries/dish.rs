//! What was cooked: a graph recipe at its graph version.

use super::problems::EntryProblem;
use crate::graph::Graph;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

/// Longer than any recipe or graph version id, short enough to refuse junk.
const MAX_ID_LENGTH: usize = 200;

/// A graph recipe and the version at which it was cooked.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Dish {
    pub recipe_id: String,
    pub graph_version: String,
}

/// The dish as sent, before both identifiers are checked.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DishInput {
    recipe_id: Option<String>,
    graph_version: Option<String>,
    #[serde(flatten)]
    unknown: BTreeMap<String, Value>,
}

impl Dish {
    pub fn parse(file: DishInput) -> Result<Dish, EntryProblem> {
        if !file.unknown.is_empty() {
            return Err(EntryProblem::UnknownField);
        }
        let ids = [&file.recipe_id, &file.graph_version];
        if ids.into_iter().flatten().any(|id| !is_readable_id(id)) {
            return Err(EntryProblem::UnreadableEntry);
        }
        match (file.recipe_id, file.graph_version) {
            (Some(recipe_id), Some(graph_version)) => Ok(Dish { recipe_id, graph_version }),
            _ => Err(EntryProblem::UnreadableEntry),
        }
    }

    /// False only when the pinned graph can tell: the dish is a recipe of that very graph version, the graph knows
    /// the recipe, and none of its techniques is taught at this skill level. Anything else is not judged, so a cook
    /// of an older graph is never refused for what this graph says.
    pub fn may_teach(&self, graph: &Graph, skill_level: (&str, u8)) -> bool {
        if self.graph_version != graph.version {
            return true;
        }
        let Some(recipe) = graph.serving.recipes.get(&self.recipe_id) else {
            return true;
        };
        let (skill, level) = skill_level;
        graph.program.levels_for(&recipe.techniques).contains(&(skill.to_string(), level))
    }
}

fn is_readable_id(id: &str) -> bool {
    !id.is_empty() && id.len() <= MAX_ID_LENGTH
}
