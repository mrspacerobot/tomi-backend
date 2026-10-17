//! The cook's profile, with the field names of Tomi's `SavedProfile`.
//!
//! Field names are permanent once released: add fields, never rename or
//! remove one. JSON uses the same camelCase names as the app.

use crate::kernel::ids::SavedId;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

mod db;
mod patch;
mod problems;
mod routes;
mod stored;
#[cfg(test)]
mod tests;

pub use routes::router;

pub use db::{read, StoredProfile};
pub use patch::{apply, PatchError, ProfilePatch};
pub use stored::{from_stored, StoredProfileError};

/// Bumped whenever the profile gains a field or the stored JSON shape changes. See `stored.rs`.
pub const PROFILE_SCHEMA_VERSION: i32 = 2;

/// How much the cook already cooks. The raw values are saved by the app.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Experience {
    Never,
    #[default]
    Basics,
    Comfortable,
}

/// How amounts are written on a recipe card. The raw values are saved by the app.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Units {
    Metric,
    Cups,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields, default)]
pub struct Profile {
    /// The diet the cook keeps, or none when they eat everything.
    pub diet: Option<SavedId>,
    pub avoid: BTreeSet<SavedId>,
    pub cuisines_loved: BTreeSet<SavedId>,
    pub cuisines_to_try: BTreeSet<SavedId>,
    pub cuisines_never: BTreeSet<SavedId>,
    /// Ingredients the cook always has at home.
    pub cupboard_staples: BTreeSet<SavedId>,
    /// Whether the cook has answered the cupboard question. Until then the staples a kitchen is taken to have
    /// count, and `cupboardStaples` is not read: an empty list alone cannot say "never asked".
    pub cupboard_answered: bool,
    /// None while the cook has not answered. An empty list is an answer.
    pub equipment: Option<BTreeSet<SavedId>>,
    pub experience: Experience,
    pub servings: u8,
    /// None while the cook has not picked: the app then follows the locale.
    pub units: Option<Units>,
    /// Cooking days a week that keep the streak going.
    pub weekly_goal: u8,
    /// None when the menu order decides what is next.
    pub focus_skill: Option<SavedId>,
    pub onboarding_done: bool,
}

impl Default for Profile {
    #[expect(
        clippy::too_many_lines,
        reason = "a flat literal of every field's default, the long shape AGENTS.md allows"
    )]
    fn default() -> Self {
        Profile {
            diet: None,
            avoid: BTreeSet::new(),
            cuisines_loved: BTreeSet::new(),
            cuisines_to_try: BTreeSet::new(),
            cuisines_never: BTreeSet::new(),
            cupboard_staples: BTreeSet::new(),
            cupboard_answered: false,
            equipment: None,
            experience: Experience::default(),
            servings: 2,
            units: None,
            weekly_goal: 3,
            focus_skill: None,
            onboarding_done: false,
        }
    }
}

impl Profile {
    /// The restrictions this profile has that `next` no longer has, as
    /// `diet: vegetarian` or `avoid: nuts`.
    ///
    /// Changing one diet for another counts as dropping the first: diets
    /// are not ordered, so a change could be weaker.
    pub fn restrictions_dropped_by(&self, next: &Profile) -> Vec<String> {
        let mut dropped = Vec::new();
        if let Some(diet) = &self.diet {
            if next.diet.as_ref() != Some(diet) {
                dropped.push(format!("diet: {diet}"));
            }
        }
        for item in self.avoid.difference(&next.avoid) {
            dropped.push(format!("avoid: {item}"));
        }
        dropped
    }
}
