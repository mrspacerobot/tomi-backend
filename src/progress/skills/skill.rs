//! How far the cook is with one skill and each of its levels.

use crate::kernel::ids::SavedId;
use crate::progress::game::is_level_complete;

/// Levels are numbered from 1.
const FIRST_LEVEL: u8 = 1;

/// Where a skill stands in the catalog, as the cook sees it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SkillStatus {
    /// Not on offer yet. `missing` lists the prerequisites whose first level is still to be completed.
    Locked { missing: Vec<SavedId> },
    /// On offer, with at least one level still to complete.
    Served,
    /// Every level is complete.
    Mastered,
}

/// How far the cook is with one level of a skill.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LevelProgress {
    /// The level's number in the catalog, counting from 1.
    pub number: u8,
    /// Finished cooks at this level.
    pub cooks: u32,
    /// Recipes of this level may be offered: the skill is served and every level before it is complete.
    pub is_open: bool,
    /// The cook's experience answer covers this level, so it is complete without any cook.
    pub is_granted: bool,
}

impl LevelProgress {
    pub fn is_complete(&self) -> bool {
        if self.is_granted {
            return true;
        }
        is_level_complete(self.cooks)
    }
}

/// How far the cook is with one skill. Computed from the cook log by `ProgressRules`, never stored.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkillProgress {
    pub skill: SavedId,
    /// One entry per level of the skill, in catalog order.
    pub levels: Vec<LevelProgress>,
    /// Prerequisites whose first level is not complete yet, in the order the catalog lists them.
    pub missing_prerequisites: Vec<SavedId>,
}

impl SkillProgress {
    /// A skill is served, meaning unlocked, when no prerequisite is missing.
    pub fn is_served(&self) -> bool {
        self.missing_prerequisites.is_empty()
    }

    /// A skill is mastered when all of its levels are complete.
    pub fn is_mastered(&self) -> bool {
        self.levels.iter().all(LevelProgress::is_complete)
    }

    /// One state for the catalog. Mastered comes first, so finished work still shows
    /// if a newer catalog adds a prerequisite to a skill the cook has already mastered.
    pub fn status(&self) -> SkillStatus {
        if self.is_mastered() {
            return SkillStatus::Mastered;
        }
        if self.is_served() {
            return SkillStatus::Served;
        }
        SkillStatus::Locked { missing: self.missing_prerequisites.clone() }
    }

    /// The level the cook is on: the first one not complete yet, or the last one once the skill is
    /// mastered. Level 1 for a skill without levels.
    pub fn current_level(&self) -> u8 {
        if let Some(first_incomplete) = self.levels.iter().find(|level| !level.is_complete()) {
            return first_incomplete.number;
        }
        match self.levels.last() {
            Some(last) => last.number,
            None => FIRST_LEVEL,
        }
    }
}
