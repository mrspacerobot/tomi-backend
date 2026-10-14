//! Ids with the words the app shows for them, in the cook's language.
//!
//! The app holds no lists of its own: every name it shows comes from here.

use crate::kernel::ids::SavedId;
use crate::kernel::language::Language;
use crate::library::{Choice, Library};
use serde::Serialize;

/// An id and its name, such as `{"id": "egg", "name": "Egg"}`.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Named {
    pub id: SavedId,
    pub name: String,
}

/// An id, its name and a one-line summary when it has one.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Shown {
    pub id: SavedId,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
}

impl Shown {
    pub fn of(choice: &Choice, language: &Language) -> Self {
        let summary = choice.summary.as_ref().map(|summary| summary.text(language.as_str()).to_string());
        Shown { id: choice.id.clone(), name: choice.name.text(language.as_str()).to_string(), summary }
    }

    pub fn all(choices: &[Choice], language: &Language) -> Vec<Shown> {
        choices.iter().map(|choice| Shown::of(choice, language)).collect()
    }
}

/// Names ingredients for one cook's language.
pub struct Namer<'a> {
    pub library: &'a Library,
    pub language: &'a Language,
}

impl Namer<'_> {
    /// The ingredient with its name. An id the table does not list keeps
    /// its id as its name rather than disappearing.
    pub fn ingredient(&self, id: &SavedId) -> Named {
        let choice = self.library.table.listed.ingredient(id);
        let name = match choice {
            Some(choice) => choice.name.text(self.language.as_str()).to_string(),
            None => id.to_string(),
        };
        Named { id: id.clone(), name }
    }

    pub fn ingredients(&self, ids: &[SavedId]) -> Vec<Named> {
        ids.iter().map(|id| self.ingredient(id)).collect()
    }
}
