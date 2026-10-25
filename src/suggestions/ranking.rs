//! Deterministic ordering of eligible dishes by confirmed ingredients and pantry overlap.

use super::Candidate;

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
