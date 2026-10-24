//! The second look: the adapted recipe against what the cook avoids, from the graph's own lists.
//!
//! The plan was made from the same lists, but this runs over the finished recipe, so a plan that names a
//! food the graph does not list for its line, or one that is unsafe, or leaves out a required part, never ships.

use super::error::AdaptError;
use super::lines::UsedLine;
use crate::graph::serving::{ServedLine, ServedRecipe};
use std::collections::BTreeSet;

/// What the recipe is checked against.
pub struct Rules<'a> {
    pub left_out: &'a BTreeSet<String>,
    pub avoided: &'a BTreeSet<String>,
}

pub fn recheck(served: &ServedRecipe, used: &[UsedLine], rules: &Rules) -> Result<(), AdaptError> {
    check_left_out(served, rules.left_out)?;
    for line in used {
        let unsafe_for = facts_of(&served.lines[line.index], line.food)?;
        if !unsafe_for.is_disjoint(rules.avoided) {
            return Err(AdaptError::Unsafe);
        }
    }
    Ok(())
}

/// Only an optional part may go, and the plan must name a part the recipe has.
fn check_left_out(served: &ServedRecipe, left_out: &BTreeSet<String>) -> Result<(), AdaptError> {
    for part in left_out {
        let mut lines = served.lines.iter().filter(|line| line.part == *part).peekable();
        if lines.peek().is_none() {
            return Err(AdaptError::UnlistedFood);
        }
        if lines.any(|line| !line.optional) {
            return Err(AdaptError::RequiredLeftOut);
        }
    }
    Ok(())
}

/// The facts the food is unsafe for on this line: the line's own food or one of its listed swaps.
fn facts_of<'a>(line: &'a ServedLine, food: &str) -> Result<&'a BTreeSet<String>, AdaptError> {
    if line.thing == food {
        return Ok(&line.unsafe_for);
    }
    match line.swaps.iter().find(|swap| swap.thing == food) {
        Some(swap) => Ok(&swap.unsafe_for),
        None => Err(AdaptError::UnlistedFood),
    }
}
