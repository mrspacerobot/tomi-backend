//! Turning what a cook typed into ingredients the table knows.
//!
//! Only an exact match by name or id, in any language, counts as found.
//! A word inside a longer name is offered for the cook to choose, never
//! picked for them.

use crate::domain::display::{Named, Namer};
use crate::domain::library::Choice;
use serde::Serialize;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Basis {
    /// One ingredient has this name.
    Exact,
    /// Several ingredients could be meant: the cook chooses.
    Ambiguous,
    /// No ingredient matches: the label stays text the server cannot use.
    None,
}

#[derive(Debug, Serialize)]
pub struct Resolved {
    pub label: String,
    pub basis: Basis,
    pub candidates: Vec<Named>,
}

pub fn resolve(label: &str, namer: &Namer) -> Resolved {
    let found = found(label, namer);
    let candidates = found.choices.iter().map(|choice| namer.ingredient(&choice.id)).collect();
    Resolved { label: label.trim().to_string(), basis: found.basis, candidates }
}

struct Found<'a> {
    basis: Basis,
    choices: Vec<&'a Choice>,
}

fn found<'a>(label: &str, namer: &Namer<'a>) -> Found<'a> {
    let typed = forms(label);
    let ingredients = &namer.library.table.listed.ingredients;
    let exact: Vec<&Choice> = ingredients.iter().filter(|choice| matches_whole(choice, &typed)).collect();
    match exact.len() {
        1 => return Found { basis: Basis::Exact, choices: exact },
        0 => {}
        _ => return Found { basis: Basis::Ambiguous, choices: exact },
    }
    let partial: Vec<&Choice> = ingredients.iter().filter(|choice| contains_word(choice, &typed)).collect();
    let basis = if partial.is_empty() { Basis::None } else { Basis::Ambiguous };
    Found { basis, choices: partial }
}

/// Every name of the ingredient, and its id read as words.
fn names(choice: &Choice) -> Vec<String> {
    let mut names: Vec<String> = choice.name.all().map(normalized).collect();
    names.push(normalized(choice.id.as_str()));
    names
}

fn matches_whole(choice: &Choice, typed: &[String]) -> bool {
    names(choice).iter().any(|name| forms(name).iter().any(|form| typed.contains(form)))
}

fn contains_word(choice: &Choice, typed: &[String]) -> bool {
    let words: Vec<String> = names(choice).iter().flat_map(|name| words_of(name)).collect();
    typed.iter().any(|form| words.contains(form))
}

fn words_of(name: &str) -> Vec<String> {
    name.split(' ').flat_map(forms).collect()
}

/// Lowercase, single spaces, hyphens read as spaces.
fn normalized(text: &str) -> String {
    let spaced = text.to_lowercase().replace('-', " ");
    spaced.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// The text and its simple singulars, so "eggs" finds "egg" and
/// "tomatoes" finds "tomato".
fn forms(text: &str) -> Vec<String> {
    let text = normalized(text);
    let mut forms = vec![text.clone()];
    for ending in ["es", "s"] {
        if let Some(stem) = text.strip_suffix(ending) {
            forms.push(stem.to_string());
        }
    }
    forms
}
