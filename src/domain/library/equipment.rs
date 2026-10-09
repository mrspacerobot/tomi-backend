//! The equipment list: everything a lesson may ask a kitchen to have.

use super::catalog::check_schema;
use super::LocalizedText;
use crate::domain::ids::SavedId;
use serde::Deserialize;
use std::collections::BTreeSet;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct EquipmentFile {
    schema_version: i64,
    equipment: Vec<Equipment>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Equipment {
    pub id: SavedId,
    pub name: LocalizedText,
    pub summary: LocalizedText,
    /// Whether a kitchen is taken to have it until the cook answers.
    pub assumed: bool,
}

pub struct EquipmentList {
    pub items: Vec<Equipment>,
    pub standard_kitchen: BTreeSet<SavedId>,
}

impl EquipmentList {
    pub fn parse(text: &str) -> Result<Self, String> {
        let file: EquipmentFile = serde_json::from_str(text).map_err(|error| error.to_string())?;
        check_schema(file.schema_version)?;
        let assumed = file.equipment.iter().filter(|item| item.assumed);
        let standard_kitchen: BTreeSet<SavedId> = assumed.map(|item| item.id.clone()).collect();
        if standard_kitchen.is_empty() {
            return Err("nothing is assumed, so no kitchen has anything".to_string());
        }
        Ok(EquipmentList { items: file.equipment, standard_kitchen })
    }

    pub fn knows(&self, equipment: &SavedId) -> bool {
        self.items.iter().any(|item| &item.id == equipment)
    }
}
