//! Tomi's reviewed library: diets and avoid items, equipment, skills and lessons.
//!
//! Parsed from the same JSON files the app ships (`library/`). The restrictions,
//! equipment list and catalog are read strictly: an unknown field or an
//! undeclared id rejects the whole library. A lesson that names something
//! the lists lack is left out and reported, never served.

use crate::graph::Graph;
use crate::kernel::ids::SavedId;
use std::collections::BTreeMap;

mod catalog;
mod choices;
mod equipment;
pub mod files;
mod lesson;
mod restrictions;
mod staples;
#[cfg(test)]
pub(crate) mod tests;
mod text;

pub use catalog::{Catalog, Course, Level, Skill};
pub use choices::Choices;
pub use equipment::{Equipment, EquipmentList};
pub use lesson::{Lesson, LessonIngredient};
pub use restrictions::{Kind, Refusal, RestrictionList, Restrictions};
pub use staples::{Staple, StapleList};
pub use text::{Choice, LocalizedText};

/// The library's files as text. Reading them from disk is the caller's job,
/// so the rules here stay free of I/O.
pub struct LibraryFiles {
    pub catalog: String,
    pub restrictions: String,
    pub staples: String,
    pub equipment: String,
    /// The server's own `choices.json`: cuisines, experience and units.
    pub choices: String,
    /// File name and text of every lesson file.
    pub lessons: Vec<(String, String)>,
}

pub struct Library {
    pub version: i64,
    pub restrictions: RestrictionList,
    pub staples: StapleList,
    pub equipment: EquipmentList,
    pub catalog: Catalog,
    pub choices: Choices,
    pub lessons: BTreeMap<SavedId, Lesson>,
}

#[derive(Debug, thiserror::Error)]
#[error("{file}: {problem}")]
pub struct ParseLibraryError {
    pub file: String,
    pub problem: String,
}

impl Library {
    /// The library, and the lessons left out with the reason for each.
    pub fn parse(files: &LibraryFiles) -> Result<(Library, Vec<String>), ParseLibraryError> {
        let restrictions = RestrictionList::parse(&files.restrictions).map_err(in_file("restrictions.json"))?;
        let staples = StapleList::parse(&files.staples).map_err(in_file("staples.json"))?;
        let equipment = EquipmentList::parse(&files.equipment).map_err(in_file("equipment.json"))?;
        let catalog = Catalog::parse(&files.catalog).map_err(in_file("catalog.json"))?;
        let choices = Choices::parse(&files.choices).map_err(in_file("choices.json"))?;
        let version = catalog.library_version;
        let mut library =
            Library { version, restrictions, staples, equipment, catalog, choices, lessons: BTreeMap::new() };
        let left_out = library.add_lessons(&files.lessons);
        Ok((library, left_out))
    }

    /// Settles what only the graph can tell: the avoid items it adds, and that every fact the restrictions name
    /// is one it has. Without this the library's own avoid items stand as written.
    pub fn agree_with(&mut self, graph: &Graph) -> Result<(), String> {
        self.restrictions.extend_from_graph(&graph.facts)?;
        self.restrictions.check_against(&graph.serving.facts)?;
        self.staples.check_against(&graph.foods)
    }

    fn add_lessons(&mut self, files: &[(String, String)]) -> Vec<String> {
        let mut left_out = Vec::new();
        for (name, text) in files {
            let added = Lesson::parse(text, self).and_then(|lesson| self.add_lesson(lesson));
            if let Err(problem) = added {
                left_out.push(format!("{name}: {problem}"));
            }
        }
        left_out
    }

    fn add_lesson(&mut self, lesson: Lesson) -> Result<(), String> {
        if self.lessons.contains_key(&lesson.id) {
            return Err(format!("lesson `{}` appears twice", lesson.id));
        }
        self.lessons.insert(lesson.id.clone(), lesson);
        Ok(())
    }
}

fn in_file(file: &'static str) -> impl Fn(String) -> ParseLibraryError {
    move |problem| ParseLibraryError { file: file.to_string(), problem }
}
