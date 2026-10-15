//! The cook's diets and avoid items, each defined as the graph facts it rules out.
//!
//! Ids are permanent (saved preferences keep them); only a person edits `restrictions.json`.
//! An id the list does not have refuses, never passes.

use super::{Choice, LocalizedText};
use crate::kernel::ids::SavedId;
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};

mod derived;
#[cfg(test)]
mod tests;

const SCHEMA_VERSION: i64 = 2;
const FACT_PREFIX: &str = "fact/";

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct File {
    schema_version: i64,
    /// Whether every allergen fact no item rules out gets an item of its own, once the graph is known.
    allergens_from_graph: bool,
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

/// Whether an avoid item is about an allergen or a choice. An item nobody can classify is read as an allergen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    Allergen,
    Choice,
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
    kinds: BTreeMap<SavedId, Kind>,
    allergens_from_graph: bool,
    /// Every fact the graph has, once it is known, so a saved id that is a fact's name is still read as it.
    known_facts: BTreeSet<String>,
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
            facts.extend(self.facts_of_item(item)?);
        }
        Ok(facts)
    }

    /// An item's facts. An id no item has is still read as the graph fact of that name, which is only stricter.
    fn facts_of_item(&self, item: &SavedId) -> Result<BTreeSet<String>, Refusal> {
        if let Some(found) = self.item_facts.get(item) {
            return Ok(found.clone());
        }
        let named = format!("{FACT_PREFIX}{item}");
        if self.known_facts.contains(&named) {
            return Ok(BTreeSet::from([named]));
        }
        Err(Refusal::UnknownAvoidItem(item.clone()))
    }

    /// Every avoid item with the facts it rules out, written and derived.
    pub fn item_facts(&self) -> &BTreeMap<SavedId, BTreeSet<String>> {
        &self.item_facts
    }

    pub fn kind_of(&self, item: &SavedId) -> Kind {
        self.kinds.get(item).copied().unwrap_or(Kind::Allergen)
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
            kinds: BTreeMap::new(),
            allergens_from_graph: file.allergens_from_graph,
            known_facts: BTreeSet::new(),
        }
    }
}
