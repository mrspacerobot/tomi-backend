//! Cuisines, experience answers and units the app offers in onboarding.
//!
//! The app's library has no file for these, so the server keeps
//! `library/choices.json` (see `library/SOURCE.md`).

use super::catalog::check_schema;
use super::{Choice, LocalizedText};
use crate::domain::ids::SavedId;
use crate::domain::profile::{Experience, Units};
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ChoicesFile {
    schema_version: i64,
    cuisines: Vec<Entry>,
    experience: Vec<Entry>,
    units: Vec<Entry>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Entry {
    id: SavedId,
    name: LocalizedText,
    summary: Option<LocalizedText>,
}

pub struct Choices {
    pub cuisines: Vec<Choice>,
    pub experience: Vec<Choice>,
    pub units: Vec<Choice>,
}

impl Choices {
    pub fn parse(text: &str) -> Result<Self, String> {
        let file: ChoicesFile = serde_json::from_str(text).map_err(|error| error.to_string())?;
        check_schema(file.schema_version)?;
        check_saved_values::<Experience>(&file.experience)?;
        check_saved_values::<Units>(&file.units)?;
        let choices = |entries: Vec<Entry>| entries.into_iter().map(Choice::from).collect();
        Ok(Choices {
            cuisines: choices(file.cuisines),
            experience: choices(file.experience),
            units: choices(file.units),
        })
    }

    pub fn has_cuisine(&self, id: &SavedId) -> bool {
        self.cuisines.iter().any(|choice| &choice.id == id)
    }
}

impl From<Entry> for Choice {
    fn from(entry: Entry) -> Self {
        Choice { id: entry.id, name: entry.name, summary: entry.summary }
    }
}

/// Every entry must be a value the profile can save, so the app never
/// offers an answer the server would refuse.
fn check_saved_values<T: for<'de> Deserialize<'de>>(entries: &[Entry]) -> Result<(), String> {
    for entry in entries {
        let value = serde_json::Value::String(entry.id.to_string());
        if serde_json::from_value::<T>(value).is_err() {
            return Err(format!("`{}` is not a value the profile can save", entry.id));
        }
    }
    Ok(())
}
