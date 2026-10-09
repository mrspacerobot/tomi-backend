//! What makes a profile unfit to save.

use super::Profile;
use crate::domain::ids::SavedId;
use std::collections::BTreeSet;
use std::ops::RangeInclusive;

/// Far more than any list a cook fills in; more is a bug or an attack.
const MAX_LIST_LENGTH: usize = 200;
/// Tomi only asks for at least one. The top guards against typing errors.
const SERVINGS: RangeInclusive<u8> = 1..=24;
/// Cooking days in a week, as Tomi's `GameRules.weeklyGoalOptions`.
const WEEKLY_GOAL: RangeInclusive<u8> = 1..=7;

impl Profile {
    /// Every problem at once, so the app can show them together.
    pub fn problems(&self) -> Vec<String> {
        let mut problems = Vec::new();
        self.check_list_lengths(&mut problems);
        self.check_cuisines(&mut problems);
        problems.extend(out_of_range("servings", self.servings, &SERVINGS));
        problems.extend(out_of_range("weeklyGoal", self.weekly_goal, &WEEKLY_GOAL));
        problems
    }

    fn check_list_lengths(&self, problems: &mut Vec<String>) {
        check_length(problems, "avoid", &self.avoid);
        check_length(problems, "cuisinesLoved", &self.cuisines_loved);
        check_length(problems, "cuisinesToTry", &self.cuisines_to_try);
        check_length(problems, "cuisinesNever", &self.cuisines_never);
        check_length(problems, "cupboardStaples", &self.cupboard_staples);
        if let Some(equipment) = &self.equipment {
            check_length(problems, "equipment", equipment);
        }
    }

    /// A cuisine has one answer. The app would read two answers as the one
    /// that offers it least, but a write should never create the doubt.
    fn check_cuisines(&self, problems: &mut Vec<String>) {
        let mut seen: BTreeSet<&SavedId> = BTreeSet::new();
        let all = self.cuisines_loved.iter().chain(&self.cuisines_to_try).chain(&self.cuisines_never);
        for cuisine in all {
            let is_new = seen.insert(cuisine);
            if !is_new {
                problems.push(format!("cuisine `{cuisine}` is in more than one list"));
            }
        }
    }
}

fn check_length(problems: &mut Vec<String>, field: &str, ids: &BTreeSet<SavedId>) {
    if ids.len() > MAX_LIST_LENGTH {
        problems.push(format!("{field} holds more than {MAX_LIST_LENGTH} items"));
    }
}

fn out_of_range(field: &str, value: u8, range: &RangeInclusive<u8>) -> Option<String> {
    if range.contains(&value) {
        return None;
    }
    let (low, high) = (range.start(), range.end());
    Some(format!("{field} must be from {low} to {high}"))
}
