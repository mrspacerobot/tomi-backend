//! What was cooked: a graph recipe at a graph version, or a lesson of the old development logs.

use super::problems::EntryProblem;
use crate::graph::Graph;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

/// Longer than any recipe, lesson or graph version id, short enough to refuse junk.
const MAX_ID_LENGTH: usize = 200;

/// Exactly one kind of dish, so an entry can never name both or neither.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(untagged, rename_all_fields = "camelCase")]
pub enum Dish {
    Recipe { recipe_id: String, graph_version: String },
    Lesson { lesson_id: String },
}

/// The dish as sent, before it is known to be exactly one kind.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DishFile {
    recipe_id: Option<String>,
    graph_version: Option<String>,
    lesson_id: Option<String>,
    #[serde(flatten)]
    unknown: BTreeMap<String, Value>,
}

impl Dish {
    pub fn parse(file: DishFile) -> Result<Dish, EntryProblem> {
        if !file.unknown.is_empty() {
            return Err(EntryProblem::UnknownField);
        }
        let ids = [&file.recipe_id, &file.graph_version, &file.lesson_id];
        if ids.into_iter().flatten().any(|id| !is_readable_id(id)) {
            return Err(EntryProblem::UnreadableEntry);
        }
        match (file.recipe_id, file.graph_version, file.lesson_id) {
            (Some(recipe_id), Some(graph_version), None) => Ok(Dish::Recipe { recipe_id, graph_version }),
            (None, None, Some(lesson_id)) => Ok(Dish::Lesson { lesson_id }),
            _ => Err(EntryProblem::UnreadableEntry),
        }
    }

    /// False only when the pinned graph can tell: the dish is a recipe of that very graph version, the graph knows
    /// the recipe, and none of its techniques is taught at this skill level. Anything else is not judged, so a cook
    /// of an older graph or a lesson is never refused for what this graph says.
    pub fn may_teach(&self, graph: &Graph, skill_level: (&str, u8)) -> bool {
        let Dish::Recipe { recipe_id, graph_version } = self else {
            return true;
        };
        if graph_version != &graph.version {
            return true;
        }
        let Some(recipe) = graph.serving.recipes.get(recipe_id) else {
            return true;
        };
        let (skill, level) = skill_level;
        graph.program.levels_for(&recipe.techniques).contains(&(skill.to_string(), level))
    }
}

fn is_readable_id(id: &str) -> bool {
    !id.is_empty() && id.len() <= MAX_ID_LENGTH
}
