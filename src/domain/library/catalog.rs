//! The skill catalog and the equipment list.

use super::LocalizedText;
use crate::domain::ids::SavedId;
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};

const SCHEMA_VERSION: i64 = 1;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct CatalogFile {
    schema_version: i64,
    library_version: i64,
    #[serde(rename = "courses")]
    _courses: Vec<Course>,
    skills: Vec<Skill>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Course {
    #[serde(rename = "id")]
    _id: SavedId,
    #[serde(rename = "title")]
    _title: LocalizedText,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Skill {
    id: SavedId,
    #[serde(rename = "course")]
    _course: SavedId,
    #[serde(rename = "name")]
    _name: LocalizedText,
    #[serde(rename = "summary")]
    _summary: LocalizedText,
    #[serde(rename = "prerequisites")]
    _prerequisites: Vec<SavedId>,
    levels: Vec<Level>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Level {
    #[serde(rename = "number")]
    _number: u8,
    #[serde(rename = "name")]
    _name: LocalizedText,
}

/// The skills and how many levels each has.
pub struct Catalog {
    pub library_version: i64,
    levels: BTreeMap<SavedId, u8>,
}

impl Catalog {
    pub fn parse(text: &str) -> Result<Self, String> {
        let file: CatalogFile = serde_json::from_str(text).map_err(|error| error.to_string())?;
        check_schema(file.schema_version)?;
        let mut levels = BTreeMap::new();
        for skill in file.skills {
            let count = u8::try_from(skill.levels.len()).unwrap_or(u8::MAX);
            levels.insert(skill.id, count);
        }
        Ok(Catalog { library_version: file.library_version, levels })
    }

    /// Whether the catalog has this skill at this level, counting from 1.
    pub fn has_level(&self, skill: &SavedId, level: u8) -> bool {
        match self.levels.get(skill) {
            Some(count) => (1..=*count).contains(&level),
            None => false,
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct EquipmentFile {
    schema_version: i64,
    equipment: Vec<Equipment>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Equipment {
    id: SavedId,
    #[serde(rename = "name")]
    _name: LocalizedText,
    #[serde(rename = "summary")]
    _summary: LocalizedText,
    assumed: bool,
}

/// Everything a lesson may ask a kitchen to have.
pub struct EquipmentList {
    known: BTreeSet<SavedId>,
    /// What a kitchen is taken to have until the cook answers.
    pub standard_kitchen: BTreeSet<SavedId>,
}

impl EquipmentList {
    pub fn parse(text: &str) -> Result<Self, String> {
        let file: EquipmentFile = serde_json::from_str(text).map_err(|error| error.to_string())?;
        check_schema(file.schema_version)?;
        let known = file.equipment.iter().map(|item| item.id.clone()).collect();
        let assumed = file.equipment.into_iter().filter(|item| item.assumed);
        let standard_kitchen: BTreeSet<SavedId> = assumed.map(|item| item.id).collect();
        if standard_kitchen.is_empty() {
            return Err("nothing is assumed, so no kitchen has anything".to_string());
        }
        Ok(EquipmentList { known, standard_kitchen })
    }

    pub fn knows(&self, equipment: &SavedId) -> bool {
        self.known.contains(equipment)
    }
}

fn check_schema(version: i64) -> Result<(), String> {
    if version == SCHEMA_VERSION {
        return Ok(());
    }
    Err(format!("schema version {version} is not {SCHEMA_VERSION}"))
}
