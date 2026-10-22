//! The foods the graph knows: every ingredient with its names, as the cook sees and types them.
//!
//! An ingredient whose id or English name this server cannot read is left out and counted, never guessed at.

use crate::graph::steps::StepError;
use crate::kernel::ids::SavedId;
use crate::library::{Choice, LocalizedText};
use serde_json::Value;
use std::collections::HashMap;

#[cfg(test)]
mod tests;

const INGREDIENT_PREFIX: &str = "ingredient/";

#[derive(Debug, Default)]
pub struct Foods {
    /// In id order, so a lookup can search it.
    listed: Vec<Choice>,
    left_out: usize,
}

impl Foods {
    pub fn empty() -> Foods {
        Foods::default()
    }

    pub fn from_things(things: &HashMap<String, Value>) -> Foods {
        let mut foods = Foods::default();
        for (thing_id, thing) in things {
            if let Some(slug) = thing_id.strip_prefix(INGREDIENT_PREFIX) {
                foods.add(slug, thing);
            }
        }
        foods.listed.sort_by(|left, right| left.id.cmp(&right.id));
        foods
    }

    fn add(&mut self, slug: &str, thing: &Value) {
        match choice_of(slug, thing) {
            Some(choice) => self.listed.push(choice),
            None => self.left_out += 1,
        }
    }

    pub fn listed(&self) -> &[Choice] {
        &self.listed
    }

    pub fn choice(&self, id: &SavedId) -> Option<&Choice> {
        let found = self.listed.binary_search_by(|choice| choice.id.cmp(id));
        found.ok().map(|at| &self.listed[at])
    }

    pub fn knows(&self, id: &SavedId) -> bool {
        self.choice(id).is_some()
    }

    pub fn left_out(&self) -> usize {
        self.left_out
    }

    /// Every id, in id order, for the question put to the model.
    pub fn ids(&self) -> Vec<SavedId> {
        self.listed.iter().map(|choice| choice.id.clone()).collect()
    }
}

fn choice_of(slug: &str, thing: &Value) -> Option<Choice> {
    let id = SavedId::parse(slug).ok()?;
    let names = thing.get("names")?.clone();
    let name: LocalizedText = serde_json::from_value(names).ok()?;
    Some(Choice::named(&id, &name))
}

/// The English name of a food, or its English plural when asked for and the food has one.
/// An uncountable food has only its name, which is what a plural placeholder then prints.
pub fn english_name(id: &str, food: &Value, plural: bool) -> Result<String, StepError> {
    let Some(uncountable) = food.get("uncountable").and_then(Value::as_bool) else {
        return Err(StepError::UnknownFood(id.to_string()));
    };
    if plural && !uncountable {
        return word(food, "plurals").ok_or_else(|| StepError::MissingPlural(id.to_string()));
    }
    word(food, "names").ok_or_else(|| StepError::UnknownFood(id.to_string()))
}

fn word(food: &Value, field: &str) -> Option<String> {
    let found = food.get(field)?.get("en")?;
    found.as_str().map(str::to_string)
}
