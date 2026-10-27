//! The rules a readable cook entry must keep before it is stored, each with its own problem code.

use super::dish::Dish;
use super::entry::{CookOutcome, EntryChecks, ValidatedCookEntry, ENTRY_SCHEMA_VERSION};
use super::problems::EntryProblem;
use crate::kernel::calendar::known_zone;
use crate::kernel::ids::SavedId;
use crate::progress::game::{COOKED_IN_FUTURE_TOLERANCE, WEEKLY_GOAL_MAXIMUM, WEEKLY_GOAL_MINIMUM};
use jiff::Timestamp;
use uuid::Uuid;

/// An entry whose fields all have the right shape, before their values are checked.
pub struct ParsedEntry {
    pub id: Uuid,
    pub cooked_at: Timestamp,
    pub time_zone: String,
    pub dish: Dish,
    pub skill: String,
    pub level: i64,
    pub outcome: CookOutcome,
    pub weekly_goal: i64,
}

impl ParsedEntry {
    /// The entry to store, or every rule it breaks, in the order of the problem codes.
    pub fn checked(self, checks: &EntryChecks) -> Result<ValidatedCookEntry, Vec<EntryProblem>> {
        let problems = self.problems(checks);
        if !problems.is_empty() {
            return Err(problems);
        }
        match self.into_entry() {
            Some(entry) => Ok(entry),
            None => Err(vec![EntryProblem::UnreadableEntry]),
        }
    }

    fn problems(&self, checks: &EntryChecks) -> Vec<EntryProblem> {
        let broken = [
            (self.skill().is_none(), EntryProblem::InvalidSkillId),
            (self.level().is_none(), EntryProblem::LevelOutOfRange),
            (self.weekly_goal().is_none(), EntryProblem::WeeklyGoalOutOfRange),
            (is_in_future(self.cooked_at, checks.received_at), EntryProblem::CookedInFuture),
            (known_zone(&self.time_zone).is_none(), EntryProblem::InvalidTimeZone),
            (!self.may_teach(checks), EntryProblem::RecipeDoesNotTeachLevel),
        ];
        broken.into_iter().filter(|(is_broken, _)| *is_broken).map(|(_, problem)| problem).collect()
    }

    /// Whether the dish may teach this skill level. Not judged while the skill or level cannot be read, or without a
    /// pinned graph.
    fn may_teach(&self, checks: &EntryChecks) -> bool {
        let (Some(skill), Some(level), Some(graph)) = (self.skill(), self.level(), checks.graph) else {
            return true;
        };
        self.dish.may_teach(graph, (skill.as_str(), level))
    }

    fn into_entry(self) -> Option<ValidatedCookEntry> {
        Some(ValidatedCookEntry {
            schema_version: ENTRY_SCHEMA_VERSION,
            id: self.id,
            cooked_at: self.cooked_at,
            time_zone: known_zone(&self.time_zone)?.iana_name()?.to_string(),
            skill: self.skill()?,
            level: self.level()?,
            weekly_goal: self.weekly_goal()?,
            outcome: self.outcome,
            dish: self.dish,
        })
    }

    fn skill(&self) -> Option<SavedId> {
        SavedId::parse(&self.skill).ok()
    }

    /// Levels are numbered from 1.
    fn level(&self) -> Option<u8> {
        let level = u8::try_from(self.level).ok()?;
        (level >= 1).then_some(level)
    }

    fn weekly_goal(&self) -> Option<u8> {
        let goal = u8::try_from(self.weekly_goal).ok()?;
        (WEEKLY_GOAL_MINIMUM..=WEEKLY_GOAL_MAXIMUM).contains(&goal).then_some(goal)
    }
}

/// A phone's clock may run a little fast; a cook later than that after it arrived cannot have happened yet.
fn is_in_future(cooked_at: Timestamp, received_at: Timestamp) -> bool {
    let latest = received_at.saturating_add(COOKED_IN_FUTURE_TOLERANCE);
    match latest {
        Ok(latest) => cooked_at > latest,
        Err(_) => false,
    }
}
