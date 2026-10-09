//! Which skills and levels are open to the cook, as Tomi's `ProgressRules`.
//!
//! A skill is served once level 1 of every prerequisite is complete. Its
//! level 1 then opens, and each later level opens once every level before
//! it is complete. "Comfortable" in onboarding counts every skill of the
//! first course as complete, without a cook and without XP.

use crate::domain::game::COOKS_TO_COMPLETE_LEVEL;
use crate::domain::ids::SavedId;
use crate::domain::library::{Catalog, Skill};
use crate::domain::profile::Experience;
use std::collections::{BTreeMap, BTreeSet};

#[cfg(test)]
mod tests;

/// Finished cooks per skill and level. Empty until the server keeps the
/// cook log, so for now only grants and level 1 of open skills count.
#[derive(Debug, Default)]
pub struct CookCounts(pub BTreeMap<(SavedId, u8), u32>);

/// The skill levels a cook may be offered a lesson at.
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

pub fn open_levels(catalog: &Catalog, experience: Experience, counts: &CookCounts) -> OpenLevels {
    let progress = Progress { granted: granted_skills(catalog, experience), counts };
    let mut open = BTreeSet::new();
    for skill in &catalog.skills {
        if progress.is_served(skill) {
            open.extend(progress.open_levels_of(skill));
        }
    }
    OpenLevels(open)
}

/// Every skill of the first course, for a cook who answered "comfortable".
fn granted_skills(catalog: &Catalog, experience: Experience) -> BTreeSet<SavedId> {
    let Some(first_course) = catalog.courses.first() else {
        return BTreeSet::new();
    };
    if experience != Experience::Comfortable {
        return BTreeSet::new();
    }
    let in_course = catalog.skills.iter().filter(|skill| skill.course == first_course.id);
    in_course.map(|skill| skill.id.clone()).collect()
}

struct Progress<'a> {
    granted: BTreeSet<SavedId>,
    counts: &'a CookCounts,
}

impl Progress<'_> {
    fn is_complete(&self, skill: &SavedId, level: u8) -> bool {
        if self.granted.contains(skill) {
            return true;
        }
        let cooks = self.counts.0.get(&(skill.clone(), level)).copied().unwrap_or(0);
        cooks >= COOKS_TO_COMPLETE_LEVEL
    }

    fn is_served(&self, skill: &Skill) -> bool {
        skill.prerequisites.iter().all(|prerequisite| self.is_complete(prerequisite, 1))
    }

    /// Level 1, then each level whose levels before it are all complete.
    fn open_levels_of(&self, skill: &Skill) -> Vec<(SavedId, u8)> {
        let mut open = Vec::new();
        for level in &skill.levels {
            open.push((skill.id.clone(), level.number));
            if !self.is_complete(&skill.id, level.number) {
                break;
            }
        }
        open
    }
}
