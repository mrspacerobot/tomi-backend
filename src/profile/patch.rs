//! Changing a profile: a partial update checked as a whole.

use super::{Experience, Profile, Units};
use crate::kernel::ids::SavedId;
use serde::{Deserialize, Deserializer};
use std::collections::BTreeSet;

/// A partial update. A field that is present replaces the stored value;
/// `null` clears a field that may be empty.
///
/// `version` is the version the app last read. The caller refuses a stale
/// one, because checking it needs the database.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProfilePatch {
    pub version: i64,
    /// Removing a diet or an avoid item must be asked for explicitly, so
    /// a stale or buggy app cannot quietly loosen what the cook may eat.
    #[serde(default)]
    pub allow_weaker_restrictions: bool,
    #[serde(default, deserialize_with = "present")]
    diet: Option<Option<SavedId>>,
    avoid: Option<BTreeSet<SavedId>>,
    cuisines_loved: Option<BTreeSet<SavedId>>,
    cuisines_to_try: Option<BTreeSet<SavedId>>,
    cuisines_never: Option<BTreeSet<SavedId>>,
    cupboard_staples: Option<BTreeSet<SavedId>>,
    #[serde(default, deserialize_with = "present")]
    equipment: Option<Option<BTreeSet<SavedId>>>,
    experience: Option<Experience>,
    servings: Option<u8>,
    #[serde(default, deserialize_with = "present")]
    units: Option<Option<Units>>,
    weekly_goal: Option<u8>,
    #[serde(default, deserialize_with = "present")]
    focus_skill: Option<Option<SavedId>>,
    onboarding_done: Option<bool>,
}

#[derive(Debug, PartialEq)]
pub enum PatchError {
    Invalid(Vec<String>),
    WeakensRestrictions { removed: Vec<String> },
}

/// Applies a patch and returns the new profile, or why it was refused.
pub fn apply(current: &Profile, patch: ProfilePatch) -> Result<Profile, PatchError> {
    let allow_weaker = patch.allow_weaker_restrictions;
    let next = patched(current, patch);
    let problems = next.problems();
    if !problems.is_empty() {
        return Err(PatchError::Invalid(problems));
    }
    let removed = current.restrictions_dropped_by(&next);
    if !removed.is_empty() && !allow_weaker {
        return Err(PatchError::WeakensRestrictions { removed });
    }
    Ok(next)
}

fn patched(current: &Profile, patch: ProfilePatch) -> Profile {
    let mut next = current.clone();
    replace_food(&mut next, &patch);
    replace_kitchen(&mut next, patch);
    next
}

fn replace_food(next: &mut Profile, patch: &ProfilePatch) {
    replace(&mut next.diet, &patch.diet);
    replace(&mut next.avoid, &patch.avoid);
    replace(&mut next.cuisines_loved, &patch.cuisines_loved);
    replace(&mut next.cuisines_to_try, &patch.cuisines_to_try);
    replace(&mut next.cuisines_never, &patch.cuisines_never);
    replace(&mut next.cupboard_staples, &patch.cupboard_staples);
}

fn replace_kitchen(next: &mut Profile, patch: ProfilePatch) {
    replace(&mut next.equipment, &patch.equipment);
    replace(&mut next.experience, &patch.experience);
    replace(&mut next.servings, &patch.servings);
    replace(&mut next.units, &patch.units);
    replace(&mut next.weekly_goal, &patch.weekly_goal);
    replace(&mut next.focus_skill, &patch.focus_skill);
    replace(&mut next.onboarding_done, &patch.onboarding_done);
}

/// Overwrites `field` when the patch carries a value for it.
fn replace<T: Clone>(field: &mut T, value: &Option<T>) {
    if let Some(value) = value {
        *field = value.clone();
    }
}

/// Tells "field absent" (outer `None`) apart from "field set to null"
/// (`Some(None)`), so the app can clear a field.
fn present<'de, D, T>(deserializer: D) -> Result<Option<Option<T>>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    let value = Option::<T>::deserialize(deserializer)?;
    Ok(Some(value))
}
