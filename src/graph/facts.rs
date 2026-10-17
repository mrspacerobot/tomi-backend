//! The graph's facts as the server needs them to build avoid items: a name, and whether it is an allergen.
//!
//! A concern this build does not know is read as an allergen: showing one item too many is safe, one too few is not.

use crate::library::LocalizedText;
use serde_json::Value;
use std::collections::{BTreeMap, HashMap};

#[cfg(test)]
mod tests;

const FACT_PREFIX: &str = "fact/";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Concern {
    Allergen,
    Diet,
}

#[derive(Debug, Clone)]
pub struct FactInfo {
    pub name: LocalizedText,
    pub concern: Concern,
}

#[derive(Debug, Default)]
pub struct Facts {
    by_id: BTreeMap<String, FactInfo>,
    left_out: usize,
}

impl Facts {
    pub fn from_things(things: &HashMap<String, Value>) -> Facts {
        let mut facts = Facts::default();
        for (thing_id, thing) in things.iter().filter(|(thing_id, _)| thing_id.starts_with(FACT_PREFIX)) {
            match info_of(thing) {
                Some(info) => facts.by_id.insert(thing_id.clone(), info),
                None => {
                    facts.left_out += 1;
                    None
                }
            };
        }
        facts
    }

    pub fn get(&self, fact_id: &str) -> Option<&FactInfo> {
        self.by_id.get(fact_id)
    }

    /// The allergen facts, in id order.
    pub fn allergens(&self) -> impl Iterator<Item = (&String, &FactInfo)> {
        self.by_id.iter().filter(|(_, info)| info.concern == Concern::Allergen)
    }

    pub fn ids(&self) -> impl Iterator<Item = &String> {
        self.by_id.keys()
    }

    pub fn left_out(&self) -> usize {
        self.left_out
    }
}

fn info_of(thing: &Value) -> Option<FactInfo> {
    let name: LocalizedText = serde_json::from_value(thing.get("names")?.clone()).ok()?;
    let concern = match thing.get("concern").and_then(Value::as_str) {
        Some("diet") => Concern::Diet,
        _ => Concern::Allergen,
    };
    Some(FactInfo { name, concern })
}
