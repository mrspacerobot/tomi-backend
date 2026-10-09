//! The skill catalog: courses, skills, their prerequisites and levels.

use super::LocalizedText;
use crate::domain::ids::SavedId;
use serde::Deserialize;

const SCHEMA_VERSION: i64 = 1;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct CatalogFile {
    schema_version: i64,
    library_version: i64,
    courses: Vec<Course>,
    skills: Vec<Skill>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Course {
    pub id: SavedId,
    pub title: LocalizedText,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Skill {
    pub id: SavedId,
    pub course: SavedId,
    pub name: LocalizedText,
    pub summary: LocalizedText,
    pub prerequisites: Vec<SavedId>,
    pub levels: Vec<Level>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Level {
    pub number: u8,
    pub name: LocalizedText,
}

/// The menu, in the order the app shows it.
pub struct Catalog {
    pub library_version: i64,
    pub courses: Vec<Course>,
    pub skills: Vec<Skill>,
}

impl Catalog {
    pub fn parse(text: &str) -> Result<Self, String> {
        let file: CatalogFile = serde_json::from_str(text).map_err(|error| error.to_string())?;
        check_schema(file.schema_version)?;
        Ok(Catalog { library_version: file.library_version, courses: file.courses, skills: file.skills })
    }

    pub fn skill(&self, id: &SavedId) -> Option<&Skill> {
        self.skills.iter().find(|skill| &skill.id == id)
    }

    /// Whether the catalog has this skill at this level.
    pub fn has_level(&self, skill: &SavedId, number: u8) -> bool {
        match self.skill(skill) {
            Some(skill) => skill.level(number).is_some(),
            None => false,
        }
    }
}

impl Skill {
    pub fn level(&self, number: u8) -> Option<&Level> {
        self.levels.iter().find(|level| level.number == number)
    }
}

pub fn check_schema(version: i64) -> Result<(), String> {
    if version == SCHEMA_VERSION {
        return Ok(());
    }
    Err(format!("schema version {version} is not {SCHEMA_VERSION}"))
}
