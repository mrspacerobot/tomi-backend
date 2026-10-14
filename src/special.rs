//! Tonight's special: which lessons this cook may make with what they have.
//!
//! Code decides what may be offered; a model may only reorder that list.
//! The steps follow Tomi's plan: filter by open skill level, cuisine,
//! kitchen, diet and avoid list, apply only the swaps a lesson lists, check
//! the final dish again, then rank.

use crate::kernel::ids::SavedId;
use crate::library::{Lesson, Library, Restrictions};
use crate::profile::Profile;
use crate::special::progress::OpenLevels;
use serde::Serialize;
use std::collections::BTreeSet;

pub mod game;
mod offer;
mod plan;
pub mod progress;
mod ranking;
mod routes;
#[cfg(test)]
mod tests;

pub use routes::router;

pub use offer::{Offer, OfferContext};
pub use ranking::{overlap_order, ranked, Ranked, MODEL_SHORT_LIST};

/// One lesson the cook may make, as it would be cooked tonight.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Candidate {
    pub lesson_id: SavedId,
    pub revision: u32,
    pub title: String,
    pub minutes: u32,
    pub skill: SavedId,
    pub level: u8,
    /// The ingredients after swaps, in the lesson's order.
    pub ingredients: Vec<SavedId>,
    pub swaps: Vec<AppliedSwap>,
    /// Final ingredients the cook did not say they have.
    pub missing: Vec<SavedId>,
}

/// A listed swap applied tonight, named as in the app's `AppliedSwap`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AppliedSwap {
    pub removed: SavedId,
    pub added: SavedId,
}

/// What the cook brings tonight.
pub struct Tonight<'a> {
    pub profile: &'a Profile,
    /// Confirmed from the scan or typed, plus the cupboard staples.
    pub available: BTreeSet<SavedId>,
    /// The skill levels the cook has reached; nothing locked is offered.
    pub open: &'a OpenLevels,
}

/// Every lesson this cook may be offered tonight, in the library's order.
/// An unknown diet or avoid item leaves the list empty.
pub fn candidates(library: &Library, tonight: &Tonight) -> Vec<Candidate> {
    let mut found = Vec::new();
    for lesson in library.lessons.values() {
        let is_open = tonight.open.contains(&lesson.skill, lesson.level);
        if !is_open || !fits_kitchen_and_taste(library, lesson, tonight.profile) {
            continue;
        }
        if let Some(candidate) = plan::cook(library, lesson, tonight) {
            found.push(candidate);
        }
    }
    found
}

fn fits_kitchen_and_taste(library: &Library, lesson: &Lesson, profile: &Profile) -> bool {
    if profile.cuisines_never.contains(&lesson.cuisine) {
        return false;
    }
    let kitchen = match &profile.equipment {
        Some(answered) => answered,
        None => &library.equipment.standard_kitchen,
    };
    lesson.equipment.iter().all(|item| kitchen.contains(item))
}

/// The cook's restrictions as the ingredient table reads them.
pub fn restrictions(profile: &Profile) -> Restrictions<'_> {
    Restrictions { diet: profile.diet.as_ref(), avoid: &profile.avoid }
}
