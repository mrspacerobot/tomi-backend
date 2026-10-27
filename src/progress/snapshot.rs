//! The progress snapshot: every number the phone's progress screens show, in one versioned answer.
//!
//! Computed from the cook entries on every request, with the cook's answers, the catalog and the request's
//! calendar and moment. Nothing of it is stored.

mod answers;
mod levels;
mod rank;
mod streak;

#[cfg(test)]
mod tests;

pub use answers::{ProgressAnswers, SnapshotInputs};
pub use levels::{LevelAnswer, SkillAnswer, SkillStatusAnswer};
pub use rank::RankAnswer;
pub use streak::{CalendarAnswer, StreakAnswer};

use crate::kernel::ids::SavedId;
use crate::library::Catalog;
use crate::progress::game::RULES_VERSION;
use crate::progress::{earned_xp, finished_cooks, CookEntry, ProgressRules};
use jiff::Timestamp;
use serde::Serialize;

/// The shape of the snapshot. An added field does not change it; a renamed, removed or retyped one does.
pub const SNAPSHOT_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProgressSnapshot {
    pub schema_version: u32,
    /// `game::RULES_VERSION`: which rules worked the numbers out.
    pub rules_version: u32,
    /// The catalog's `libraryVersion`.
    pub catalog_version: i64,
    /// The moment of the request, to the second.
    pub computed_at: Timestamp,
    pub calendar: CalendarAnswer,
    pub xp: u32,
    pub rank: RankAnswer,
    pub streak: StreakAnswer,
    /// `skills`, `masteredCount` and `focusSkill`, in place.
    #[serde(flatten)]
    pub catalog: CatalogAnswer,
    pub counted: Counted,
}

/// How far the cook is through the catalog.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CatalogAnswer {
    /// Every skill of the catalog, in catalog order.
    pub skills: Vec<SkillAnswer>,
    pub mastered_count: usize,
    /// The skill to cook next, or none when everything is mastered.
    pub focus_skill: Option<SavedId>,
}

/// What the log held, as the rules counted it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Counted {
    /// Finished cooks, each entry id once. Stopped-early cooks are not among them.
    pub finished_cooks: usize,
    /// Finished cooks of a skill or level the catalog does not know: they earn XP and count for the streak, and
    /// open nothing.
    pub ignored_unknown_skill_or_level: usize,
}

impl ProgressSnapshot {
    /// The snapshot of this log, in any order and with any copies, at the moment and in the calendar of the request.
    pub fn compute(log: &[CookEntry], inputs: &SnapshotInputs) -> ProgressSnapshot {
        let xp = earned_xp(log);
        ProgressSnapshot {
            schema_version: SNAPSHOT_SCHEMA_VERSION,
            rules_version: RULES_VERSION,
            catalog_version: inputs.catalog.library_version,
            computed_at: to_the_second(inputs.now),
            calendar: CalendarAnswer::of(inputs.calendar),
            xp,
            rank: RankAnswer::for_xp(xp, inputs),
            streak: StreakAnswer::compute(log, inputs),
            catalog: CatalogAnswer::compute(log, inputs),
            counted: Counted::of(log, inputs.catalog),
        }
    }
}

impl CatalogAnswer {
    fn compute(log: &[CookEntry], inputs: &SnapshotInputs) -> CatalogAnswer {
        let progress = ProgressRules::new(inputs.catalog, inputs.answers.experience).progress(log);
        CatalogAnswer {
            skills: progress.skills.iter().map(SkillAnswer::of).collect(),
            mastered_count: progress.mastered_count(),
            focus_skill: progress.focus(inputs.answers.focus_skill.as_ref()),
        }
    }
}

impl Counted {
    fn of(log: &[CookEntry], catalog: &Catalog) -> Counted {
        let finished = finished_cooks(log);
        let unknown = finished.iter().filter(|cook| !catalog.has_level(&cook.skill, cook.level));
        Counted { finished_cooks: finished.len(), ignored_unknown_skill_or_level: unknown.count() }
    }
}

/// Whole seconds, so every client's RFC 3339 reader takes the instant without fractions.
fn to_the_second(moment: Timestamp) -> Timestamp {
    match Timestamp::from_second(moment.as_second()) {
        Ok(whole) => whole,
        Err(_) => moment,
    }
}
