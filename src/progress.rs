//! The cook's progress, computed from their cook entries: XP, rank, weekly streak, open levels and
//! mastered skills.
//!
//! Every rule here is pure: it takes the entries, the catalog and the cook's answers as arguments
//! and decides from nothing else, so the same entries in any order give the same answer. Nothing is
//! stored beside the entries. The rules are a port of Tomi's TomiCore `Game` and `Progress` rules.

mod entries;
mod experience;
pub mod game;
mod routes;
mod skills;
mod snapshot;
mod streak;

#[cfg(test)]
mod fixtures;

pub use routes::{router, snapshot, SnapshotRequest};

pub use entries::{finished_cooks, CookCounts, CookEntry, Outcome, QuizAnswers, StoredEntry, UnreadableStoredEntry};
pub use experience::{earned_xp, xp_for, Rank, RankTitle};
pub use skills::{
    granted_skills, open_levels, CatalogProgress, LevelProgress, OpenLevels, ProgressRules, SkillProgress, SkillStatus,
};
pub use snapshot::{
    CalendarAnswer, CatalogAnswer, Counted, LevelAnswer, ProgressAnswers, ProgressSnapshot, RankAnswer, SkillAnswer,
    SkillStatusAnswer, SnapshotInputs, StreakAnswer, SNAPSHOT_SCHEMA_VERSION,
};
pub use streak::{CookCalendar, ParseCalendarError, Streak, StreakRules, WeekBounds};
