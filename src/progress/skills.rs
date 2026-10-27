//! How far the cook is with each skill of the catalog: served, mastered, and which levels are open.

mod catalog;
mod grants;
mod open;
mod rules;
mod skill;

#[cfg(test)]
mod tests;

pub use catalog::CatalogProgress;
pub use grants::granted_skills;
pub use open::{open_levels, OpenLevels};
pub use rules::ProgressRules;
pub use skill::{LevelProgress, SkillProgress, SkillStatus};
