//! Tomi's reviewed library: restrictions, staples, options and progression data.
//!
//! Parsed from the reviewed JSON files the app ships (`library/`). Restrictions,
//! equipment and catalog reject unknown fields and undeclared ids.

use crate::graph::Graph;

mod catalog;
mod choices;
mod equipment;
pub mod files;
mod restrictions;
mod staples;
#[cfg(test)]
pub(crate) mod tests;
mod text;

pub use catalog::{Catalog, Course, Level, Skill};
pub use choices::Choices;
pub use equipment::{Equipment, EquipmentList};
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
}

pub struct Library {
    pub version: i64,
    pub restrictions: RestrictionList,
    pub staples: StapleList,
    pub equipment: EquipmentList,
    pub catalog: Catalog,
    pub choices: Choices,
}

#[derive(Debug, thiserror::Error)]
#[error("{file}: {problem}")]
pub struct ParseLibraryError {
    pub file: String,
    pub problem: String,
}

impl Library {
    /// Parses the reviewed library files.
    pub fn parse(files: &LibraryFiles) -> Result<Library, ParseLibraryError> {
        let restrictions = RestrictionList::parse(&files.restrictions).map_err(in_file("restrictions.json"))?;
        let staples = StapleList::parse(&files.staples).map_err(in_file("staples.json"))?;
        let equipment = EquipmentList::parse(&files.equipment).map_err(in_file("equipment.json"))?;
        let catalog = Catalog::parse(&files.catalog).map_err(in_file("catalog.json"))?;
        let choices = Choices::parse(&files.choices).map_err(in_file("choices.json"))?;
        let version = catalog.library_version;
        let library = Library { version, restrictions, staples, equipment, catalog, choices };
        Ok(library)
    }

    /// Settles what only the graph can tell: the avoid items it adds, and that every fact the restrictions name
    /// is one it has. Without this the library's own avoid items stand as written.
    pub fn agree_with(&mut self, graph: &Graph) -> Result<(), String> {
        self.restrictions.extend_from_graph(&graph.facts)?;
        self.restrictions.check_against(&graph.serving.facts)?;
        self.staples.check_against(&graph.foods)
    }
}

fn in_file(file: &'static str) -> impl Fn(String) -> ParseLibraryError {
    move |problem| ParseLibraryError { file: file.to_string(), problem }
}
