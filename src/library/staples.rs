//! The everyday ingredients a cook can say they always have, and which of them a kitchen is taken to have.
//!
//! A person owns this list: it is what the cupboard question offers. Everything else a cook has is typed.
//! Each ingredient must be one the graph knows, or the start is refused.

use crate::graph::Foods;
use crate::kernel::ids::SavedId;
use serde::Deserialize;
use std::collections::BTreeSet;

#[cfg(test)]
mod tests;

const SCHEMA_VERSION: i64 = 1;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct File {
    schema_version: i64,
    staples: Vec<Staple>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Staple {
    pub ingredient: SavedId,
    /// Counted as at home until the cook has answered the cupboard question.
    pub assumed: bool,
}

#[derive(Debug)]
pub struct StapleList {
    listed: Vec<Staple>,
}

impl StapleList {
    pub fn parse(text: &str) -> Result<Self, String> {
        let file: File = serde_json::from_str(text).map_err(|error| error.to_string())?;
        if file.schema_version != SCHEMA_VERSION {
            return Err(format!("schema version {} is not {SCHEMA_VERSION}", file.schema_version));
        }
        if file.staples.is_empty() {
            return Err("the list has no staples".to_string());
        }
        let mut seen = BTreeSet::new();
        match file.staples.iter().find(|staple| !seen.insert(&staple.ingredient)) {
            Some(twice) => Err(format!("`{}` is listed twice", twice.ingredient)),
            None => Ok(StapleList { listed: file.staples }),
        }
    }

    pub fn listed(&self) -> &[Staple] {
        &self.listed
    }

    /// The staples a kitchen is taken to have until the cook answers.
    pub fn assumed(&self) -> BTreeSet<SavedId> {
        let assumed = self.listed.iter().filter(|staple| staple.assumed);
        assumed.map(|staple| staple.ingredient.clone()).collect()
    }

    /// Every staple must be an ingredient the graph has, or the start is refused.
    pub fn check_against(&self, foods: &Foods) -> Result<(), String> {
        match self.listed.iter().find(|staple| !foods.knows(&staple.ingredient)) {
            Some(unknown) => Err(format!("staples.json names `{}`, which the graph does not have", unknown.ingredient)),
            None => Ok(()),
        }
    }
}
