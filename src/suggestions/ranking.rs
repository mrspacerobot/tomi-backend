//! Ordering the candidates: by what the cook has, then by a model's choice.
//!
//! The model may only reorder the short list it was given. An id it made
//! up is ignored, and every candidate it left out keeps its overlap place.

use super::Candidate;
use serde::Serialize;
use std::collections::BTreeSet;

/// How many candidates a model is asked to rank. More costs tokens and
/// adds little: the top few are what the cook sees.
pub const MODEL_SHORT_LIST: usize = 10;

/// A lesson as a model ranked it, with its one-line reason.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Ranked {
    pub recipe_id: String,
    pub reason: Option<String>,
}

/// Confirmed foods first, then fewest missing foods, pantry overlap and recipe id.
pub fn overlap_order(mut candidates: Vec<Candidate>) -> Vec<Candidate> {
    candidates.sort_by(|left, right| {
        let by_confirmed = right.confirmed_ingredients.len().cmp(&left.confirmed_ingredients.len());
        let by_missing = left.missing.len().cmp(&right.missing.len());
        let by_at_hand = at_hand(right).cmp(&at_hand(left));
        by_confirmed.then(by_missing).then(by_at_hand).then(left.recipe_id.cmp(&right.recipe_id))
    });
    candidates
}

fn at_hand(candidate: &Candidate) -> usize {
    candidate.ingredients.len() - candidate.missing.len()
}

/// The model's order over the overlap order, as (candidate, reason) pairs.
pub fn ranked(by_overlap: Vec<Candidate>, model: &[Ranked]) -> Vec<(Candidate, Option<String>)> {
    let mut placed: BTreeSet<String> = BTreeSet::new();
    let mut result = Vec::new();
    for choice in model {
        let found = by_overlap.iter().find(|candidate| candidate.recipe_id == choice.recipe_id);
        let Some(candidate) = found else { continue };
        if placed.insert(candidate.recipe_id.clone()) {
            result.push((candidate.clone(), choice.reason.clone()));
        }
    }
    for candidate in by_overlap {
        if !placed.contains(&candidate.recipe_id) {
            result.push((candidate, None));
        }
    }
    result
}
