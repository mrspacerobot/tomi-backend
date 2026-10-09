//! The cook's profile and the rules for changing it.
//!
//! Field names are a first guess. Before the backend becomes canonical they
//! must be matched to Tomi's `SavedProfile`, whose names stay stable.

use crate::ids::TypedId;
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::BTreeSet;

/// Bumped when the stored JSON shape changes, so old rows can be migrated.
pub const PROFILE_SCHEMA_VERSION: i32 = 1;

pub const MAX_LIST_LEN: usize = 200;
pub const SERVINGS_RANGE: std::ops::RangeInclusive<u8> = 1..=12;
pub const WEEKLY_GOAL_RANGE: std::ops::RangeInclusive<u8> = 0..=14;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Experience {
    Beginner,
    Intermediate,
    Advanced,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Units {
    Metric,
    Us,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Profile {
    pub diets: BTreeSet<TypedId>,
    pub avoid: BTreeSet<TypedId>,
    pub cuisines: BTreeSet<TypedId>,
    pub pantry: BTreeSet<TypedId>,
    pub equipment: BTreeSet<TypedId>,
    pub experience: Option<Experience>,
    pub units: Units,
    pub servings: u8,
    pub weekly_goal: u8,
    pub focus_skill: Option<TypedId>,
}

impl Default for Profile {
    fn default() -> Self {
        Profile {
            diets: BTreeSet::new(),
            avoid: BTreeSet::new(),
            cuisines: BTreeSet::new(),
            pantry: BTreeSet::new(),
            equipment: BTreeSet::new(),
            experience: None,
            units: Units::Metric,
            servings: 2,
            weekly_goal: 3,
            focus_skill: None,
        }
    }
}

/// A partial update. A field that is present replaces the stored value.
/// `version` is the version the client last read; a stale one is refused.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProfilePatch {
    pub version: i64,
    /// Removing a diet or an avoided fact must be asked for explicitly,
    /// so that a stale or buggy client cannot quietly loosen restrictions.
    #[serde(default)]
    pub allow_weaker_restrictions: bool,
    pub diets: Option<BTreeSet<TypedId>>,
    pub avoid: Option<BTreeSet<TypedId>>,
    pub cuisines: Option<BTreeSet<TypedId>>,
    pub pantry: Option<BTreeSet<TypedId>>,
    pub equipment: Option<BTreeSet<TypedId>>,
    #[serde(default, deserialize_with = "present")]
    pub experience: Option<Option<Experience>>,
    pub units: Option<Units>,
    pub servings: Option<u8>,
    pub weekly_goal: Option<u8>,
    #[serde(default, deserialize_with = "present")]
    pub focus_skill: Option<Option<TypedId>>,
}

#[derive(Debug, PartialEq)]
pub enum PatchError {
    Invalid(Vec<String>),
    WeakensRestrictions { removed: Vec<TypedId> },
}

/// Tells "field absent" (outer None) apart from "field set to null"
/// (Some(None)), so a client can clear an optional field.
fn present<'de, D, T>(deserializer: D) -> Result<Option<Option<T>>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::<T>::deserialize(deserializer).map(Some)
}

impl Profile {
    /// Every problem at once, so the app can show them together.
    pub fn problems(&self) -> Vec<String> {
        let mut problems = Vec::new();
        check_list(&mut problems, "diets", &self.diets, "diet");
        check_list(&mut problems, "avoid", &self.avoid, "fact");
        check_list(&mut problems, "cuisines", &self.cuisines, "cuisine");
        check_list(&mut problems, "pantry", &self.pantry, "ingredient");
        check_list(&mut problems, "equipment", &self.equipment, "equipment");
        if let Some(skill) = &self.focus_skill {
            check_kind(&mut problems, "focus_skill", skill, "skill");
        }
        if !SERVINGS_RANGE.contains(&self.servings) {
            problems.push(format!(
                "servings must be from {} to {}",
                SERVINGS_RANGE.start(),
                SERVINGS_RANGE.end()
            ));
        }
        if !WEEKLY_GOAL_RANGE.contains(&self.weekly_goal) {
            problems.push(format!(
                "weekly_goal must be from {} to {}",
                WEEKLY_GOAL_RANGE.start(),
                WEEKLY_GOAL_RANGE.end()
            ));
        }
        problems
    }

    /// Restrictions this profile has that `next` drops.
    pub fn restrictions_dropped_by(&self, next: &Profile) -> Vec<TypedId> {
        let diets = self.diets.difference(&next.diets);
        let avoid = self.avoid.difference(&next.avoid);
        diets.chain(avoid).cloned().collect()
    }
}

fn check_list(problems: &mut Vec<String>, field: &str, ids: &BTreeSet<TypedId>, kind: &str) {
    if ids.len() > MAX_LIST_LEN {
        problems.push(format!("{field} holds more than {MAX_LIST_LEN} items"));
    }
    for id in ids {
        check_kind(problems, field, id, kind);
    }
}

fn check_kind(problems: &mut Vec<String>, field: &str, id: &TypedId, kind: &str) {
    if id.kind() != kind {
        problems.push(format!("{field}: `{id}` is not of kind {kind}"));
    }
}

/// Applies a patch to the stored profile and returns the new profile.
/// The version check is the caller's job, because it needs the database.
pub fn apply(current: &Profile, patch: ProfilePatch) -> Result<Profile, PatchError> {
    let mut next = current.clone();
    if let Some(diets) = patch.diets {
        next.diets = diets;
    }
    if let Some(avoid) = patch.avoid {
        next.avoid = avoid;
    }
    if let Some(cuisines) = patch.cuisines {
        next.cuisines = cuisines;
    }
    if let Some(pantry) = patch.pantry {
        next.pantry = pantry;
    }
    if let Some(equipment) = patch.equipment {
        next.equipment = equipment;
    }
    if let Some(experience) = patch.experience {
        next.experience = experience;
    }
    if let Some(units) = patch.units {
        next.units = units;
    }
    if let Some(servings) = patch.servings {
        next.servings = servings;
    }
    if let Some(weekly_goal) = patch.weekly_goal {
        next.weekly_goal = weekly_goal;
    }
    if let Some(focus_skill) = patch.focus_skill {
        next.focus_skill = focus_skill;
    }

    let problems = next.problems();
    if !problems.is_empty() {
        return Err(PatchError::Invalid(problems));
    }
    let removed = current.restrictions_dropped_by(&next);
    if !removed.is_empty() && !patch.allow_weaker_restrictions {
        return Err(PatchError::WeakensRestrictions { removed });
    }
    Ok(next)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn id(raw: &str) -> TypedId {
        TypedId::parse(raw).unwrap()
    }

    fn patch(value: serde_json::Value) -> ProfilePatch {
        serde_json::from_value(value).unwrap()
    }

    fn with_peanuts() -> Profile {
        Profile {
            avoid: [id("fact/peanuts")].into(),
            diets: [id("diet/vegetarian")].into(),
            ..Profile::default()
        }
    }

    #[test]
    fn adding_a_restriction_is_allowed() {
        let next = apply(
            &with_peanuts(),
            patch(json!({"version": 1, "avoid": ["fact/peanuts", "fact/sesame"]})),
        )
        .unwrap();
        assert!(next.avoid.contains(&id("fact/sesame")));
    }

    #[test]
    fn dropping_a_restriction_silently_is_refused() {
        let result = apply(&with_peanuts(), patch(json!({"version": 1, "avoid": []})));
        assert_eq!(
            result,
            Err(PatchError::WeakensRestrictions {
                removed: vec![id("fact/peanuts")]
            })
        );
    }

    #[test]
    fn dropping_a_diet_silently_is_refused() {
        let result = apply(&with_peanuts(), patch(json!({"version": 1, "diets": []})));
        assert_eq!(
            result,
            Err(PatchError::WeakensRestrictions {
                removed: vec![id("diet/vegetarian")]
            })
        );
    }

    #[test]
    fn dropping_a_restriction_on_request_is_allowed() {
        let next = apply(
            &with_peanuts(),
            patch(json!({"version": 1, "avoid": [], "allow_weaker_restrictions": true})),
        )
        .unwrap();
        assert!(next.avoid.is_empty());
    }

    #[test]
    fn a_patch_without_restrictions_leaves_them_alone() {
        let next = apply(&with_peanuts(), patch(json!({"version": 1, "servings": 4}))).unwrap();
        assert_eq!(next.avoid, with_peanuts().avoid);
        assert_eq!(next.servings, 4);
    }

    #[test]
    fn ids_of_the_wrong_kind_are_refused() {
        let result = apply(
            &Profile::default(),
            patch(json!({"version": 0, "avoid": ["diet/vegan"], "pantry": ["fact/milk"]})),
        );
        assert_eq!(
            result,
            Err(PatchError::Invalid(vec![
                "avoid: `diet/vegan` is not of kind fact".into(),
                "pantry: `fact/milk` is not of kind ingredient".into(),
            ]))
        );
    }

    #[test]
    fn numbers_out_of_range_are_refused() {
        let result = apply(
            &Profile::default(),
            patch(json!({"version": 0, "servings": 0, "weekly_goal": 30})),
        );
        assert!(matches!(result, Err(PatchError::Invalid(problems)) if problems.len() == 2));
    }

    #[test]
    fn null_clears_an_optional_field() {
        let start = Profile {
            focus_skill: Some(id("skill/knife")),
            ..Profile::default()
        };
        let next = apply(&start, patch(json!({"version": 1, "focus_skill": null}))).unwrap();
        assert_eq!(next.focus_skill, None);
        let kept = apply(&start, patch(json!({"version": 1}))).unwrap();
        assert_eq!(kept.focus_skill, Some(id("skill/knife")));
    }

    #[test]
    fn unknown_fields_are_refused() {
        let result: Result<ProfilePatch, _> =
            serde_json::from_value(json!({"version": 1, "xp": 9000}));
        assert!(result.is_err());
    }
}
