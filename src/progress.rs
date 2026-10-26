//! The cook's progress, computed from their cook log: XP, rank, open levels and mastered skills.
//!
//! Every rule here is pure: it takes the log, the catalog and the cook's answers as arguments and
//! decides from nothing else, so the same log in any order gives the same answer. Nothing is stored
//! beside the log. The rules are a port of Tomi's TomiCore `Game` and `Progress` rules.

mod counts;
pub mod game;
mod levels;
mod log;
mod menu;
mod rank;
mod skill;
mod xp;

#[cfg(test)]
mod tests;

pub use counts::CookCounts;
pub use levels::{granted_skills, open_levels, OpenLevels, ProgressRules};
pub use log::{finished_cooks, CookEntry, Outcome};
pub use menu::CookProgress;
pub use rank::{Rank, RankTitle};
pub use skill::{LevelProgress, SkillProgress, SkillStatus};
pub use xp::{earned_xp, xp_for};
