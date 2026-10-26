//! The cook's progress, computed from their cook entries: XP, rank, open levels and mastered skills.
//!
//! Every rule here is pure: it takes the entries, the catalog and the cook's answers as arguments
//! and decides from nothing else, so the same entries in any order give the same answer. Nothing is
//! stored beside the entries. The rules are a port of Tomi's TomiCore `Game` and `Progress` rules.
//!
//! `routes.rs` takes the cook entries the phone sends (`POST /v1/cook-entries`), `received` reads and checks
//! each one, and `db.rs` keeps the first accepted copy of each entry id. `cook_log` reads a cook's stored
//! entries back, so suggestions open levels from that cook's own log.

mod db;
mod entries;
mod experience;
pub mod game;
mod received;
mod routes;
mod skills;
mod snapshot;

#[cfg(test)]
mod fixtures;

pub use db::cook_log;
pub use received::{
    Dish, EntryBatch, EntryOutcome, EntryProblem, EntryResult, EntryStatus, ReceivedEntry, RequestCalendar,
    ENTRY_SCHEMA_VERSION, MAX_ENTRIES_PER_BATCH,
};
pub use routes::router;

pub use entries::{finished_cooks, CookCounts, CookEntry, Outcome};
pub use experience::{earned_xp, xp_for, Rank, RankTitle};
pub use skills::{granted_skills, open_levels, LevelProgress, OpenLevels, ProgressRules, SkillProgress, SkillStatus};
pub use snapshot::ProgressSnapshot;
