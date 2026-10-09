//! The table's own checks, as in Tomi's validator: declared facts, rules
//! that rule something out, and no id twice.

use super::TableFile;
use crate::domain::ids::SavedId;
use std::collections::BTreeSet;

/// Every fact named anywhere must be declared, every avoid item and diet
/// must rule something out, and no id may appear twice, as in Tomi's own
/// validator.
pub fn check_declared(file: &TableFile) -> Result<(), String> {
    let mut seen = BTreeSet::new();
    for item in &file.ingredients {
        check_new(&item.id, &mut seen)?;
        check_known(&item.id, &item.contains, &file.facts)?;
    }
    for item in &file.avoid_items {
        check_rule(&item.id, &item.triggered_by, &file.facts)?;
    }
    for diet in &file.diets {
        check_rule(&diet.id, &diet.excludes, &file.facts)?;
    }
    Ok(())
}

fn check_new<'a>(id: &'a SavedId, seen: &mut BTreeSet<&'a SavedId>) -> Result<(), String> {
    if seen.insert(id) {
        return Ok(());
    }
    Err(format!("`{id}` appears twice"))
}

fn check_rule(id: &SavedId, facts: &BTreeSet<String>, declared: &BTreeSet<String>) -> Result<(), String> {
    if facts.is_empty() {
        return Err(format!("`{id}` rules nothing out"));
    }
    check_known(id, facts, declared)
}

fn check_known(id: &SavedId, facts: &BTreeSet<String>, declared: &BTreeSet<String>) -> Result<(), String> {
    match facts.difference(declared).next() {
        Some(fact) => Err(format!("`{id}` names the undeclared fact `{fact}`")),
        None => Ok(()),
    }
}
