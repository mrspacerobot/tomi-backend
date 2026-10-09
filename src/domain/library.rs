//! Tomi's reviewed library: ingredient table, equipment, skills and lessons.
//!
//! Parsed from the same JSON files the app ships (`library/`). The table,
//! equipment list and catalog are read strictly: an unknown field or an
//! undeclared id rejects the whole library. A lesson that names something
//! the lists lack is left out and reported, never served.

use crate::domain::ids::SavedId;
use std::collections::BTreeMap;

mod catalog;
mod lesson;
mod table;
#[cfg(test)]
pub(crate) mod tests;
mod text;

pub use catalog::{Catalog, EquipmentList};
pub use lesson::{Lesson, LessonIngredient, Swap};
pub use table::{IngredientTable, Refusal, Restrictions};
pub use text::LocalizedText;

/// The library's files as text. Reading them from disk is the caller's job,
/// so the rules here stay free of I/O.
pub struct LibraryFiles {
    pub catalog: String,
    pub ingredients: String,
    pub equipment: String,
    /// File name and text of every lesson file.
    pub lessons: Vec<(String, String)>,
}

pub struct Library {
    pub version: i64,
    pub table: IngredientTable,
    pub equipment: EquipmentList,
    pub catalog: Catalog,
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
        let table = IngredientTable::parse(&files.ingredients).map_err(in_file("ingredients.json"))?;
        let equipment = EquipmentList::parse(&files.equipment).map_err(in_file("equipment.json"))?;
        let catalog = Catalog::parse(&files.catalog).map_err(in_file("catalog.json"))?;
        let mut library =
            Library { version: catalog.library_version, table, equipment, catalog, lessons: BTreeMap::new() };
        let left_out = library.add_lessons(&files.lessons);
        Ok((library, left_out))
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
