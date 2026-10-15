//! The cook's diets and avoid items, each defined as the graph facts it rules out.
//!
//! Ids are permanent (saved preferences keep them); only a person edits `restrictions.json`.
//! An id the list does not have refuses, never passes.

use super::{Choice, LocalizedText};
use crate::kernel::ids::SavedId;
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};

#[cfg(test)]
mod tests;

const SCHEMA_VERSION: i64 = 1;
const FACT_PREFIX: &str = "fact/";

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct File {
    schema_version: i64,
    avoid_items: Vec<Entry>,
    diets: Vec<Entry>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Entry {
    id: SavedId,
    name: LocalizedText,
    summary: Option<LocalizedText>,
    avoids: BTreeSet<String>,
}

/// What the cook may not eat, from their profile.
pub struct Restrictions<'a> {
    pub diet: Option<&'a SavedId>,
    pub avoid: &'a BTreeSet<SavedId>,
}

/// Why a cook's restrictions cannot be read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Refusal {
    UnknownDiet(SavedId),
    UnknownAvoidItem(SavedId),
}

#[derive(Debug)]
pub struct RestrictionList {
    /// What the app shows, in the file's order.
    pub diets: Vec<Choice>,
    pub avoid_items: Vec<Choice>,
    diet_facts: BTreeMap<SavedId, BTreeSet<String>>,
    item_facts: BTreeMap<SavedId, BTreeSet<String>>,
}

impl RestrictionList {
    pub fn parse(text: &str) -> Result<Self, String> {
        let file: File = serde_json::from_str(text).map_err(|error| error.to_string())?;
        if file.schema_version != SCHEMA_VERSION {
            return Err(format!("schema version {} is not {SCHEMA_VERSION}", file.schema_version));
        }
        check_entries(&file.avoid_items)?;
        check_entries(&file.diets)?;
        Ok(RestrictionList::from(file))
    }

    /// Every graph fact this cook's diet and avoid items rule out.
    pub fn facts_to_avoid(&self, cook: &Restrictions) -> Result<BTreeSet<String>, Refusal> {
        let mut facts = BTreeSet::new();
        if let Some(diet) = cook.diet {
            let found = self.diet_facts.get(diet);
            facts.extend(found.ok_or_else(|| Refusal::UnknownDiet(diet.clone()))?.iter().cloned());
        }
        for item in cook.avoid {
            let found = self.item_facts.get(item);
            facts.extend(found.ok_or_else(|| Refusal::UnknownAvoidItem(item.clone()))?.iter().cloned());
        }
        Ok(facts)
    }

    /// Every fact named here must be one the graph has, or the start is refused.
    pub fn check_against(&self, graph_facts: &BTreeSet<String>) -> Result<(), String> {
        let named = self.diet_facts.values().chain(self.item_facts.values()).flatten();
        match named.into_iter().find(|fact| !graph_facts.contains(*fact)) {
            Some(missing) => Err(format!("restrictions.json names `{missing}`, which the graph does not have")),
            None => Ok(()),
        }
    }
}

fn check_entries(entries: &[Entry]) -> Result<(), String> {
    for entry in entries {
        if entry.avoids.is_empty() {
            return Err(format!("`{}` rules nothing out", entry.id));
        }
        if let Some(wrong) = entry.avoids.iter().find(|fact| !fact.starts_with(FACT_PREFIX)) {
            return Err(format!("`{}` names `{wrong}`, which is not a graph fact id", entry.id));
        }
    }
    Ok(())
}

impl From<File> for RestrictionList {
    fn from(file: File) -> Self {
        let facts = |entries: &[Entry]| entries.iter().map(|entry| (entry.id.clone(), entry.avoids.clone())).collect();
        let choice = |entry: &Entry| match &entry.summary {
            Some(summary) => Choice::described(&entry.id, &entry.name, summary),
            None => Choice::named(&entry.id, &entry.name),
        };
        RestrictionList {
            diets: file.diets.iter().map(choice).collect(),
            avoid_items: file.avoid_items.iter().map(choice).collect(),
            diet_facts: facts(&file.diets),
            item_facts: facts(&file.avoid_items),
        }
    }
}
