//! The skill levels a cook may be offered a recipe at, read from their skill progress.

use super::rules::ProgressRules;
use crate::kernel::ids::SavedId;
use crate::library::Catalog;
use crate::profile::Experience;
use crate::progress::CookCounts;
use std::collections::BTreeSet;

/// The skill levels a cook may be offered a recipe at.
#[derive(Debug, Default, PartialEq)]
pub struct OpenLevels(BTreeSet<(SavedId, u8)>);

impl OpenLevels {
    /// Exactly these levels, for callers that know them already.
    pub fn from_levels(levels: impl IntoIterator<Item = (SavedId, u8)>) -> Self {
        OpenLevels(levels.into_iter().collect())
    }

    pub fn contains(&self, skill: &SavedId, level: u8) -> bool {
        self.0.contains(&(skill.clone(), level))
    }
}

/// Every open level of the catalog, for the cook's experience answer and finished cooks.
pub fn open_levels(catalog: &Catalog, experience: Experience, counts: &CookCounts) -> OpenLevels {
    let progress = ProgressRules::new(catalog, experience).progress_from_counts(counts);
    let mut open = BTreeSet::new();
    for skill in &progress.skills {
        let open_here = skill.levels.iter().filter(|level| level.is_open);
        open.extend(open_here.map(|level| (skill.skill.clone(), level.number)));
    }
    OpenLevels(open)
}
