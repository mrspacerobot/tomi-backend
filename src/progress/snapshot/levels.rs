//! Each skill of the catalog and its levels, as the snapshot sends them.

use crate::kernel::ids::SavedId;
use crate::progress::game::COOKS_TO_COMPLETE_LEVEL;
use crate::progress::{LevelProgress, SkillProgress, SkillStatus};
use serde::Serialize;

/// Where a skill stands. Each is shown with words or a mark on the phone, never by colour alone.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum SkillStatusAnswer {
    Locked,
    Served,
    Mastered,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillAnswer {
    pub skill: SavedId,
    pub status: SkillStatusAnswer,
    /// Prerequisites whose first level is still to complete, in catalog order.
    pub missing_prerequisites: Vec<SavedId>,
    /// The first level not complete yet, or the last once the skill is mastered.
    pub current_level: u8,
    pub levels: Vec<LevelAnswer>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelAnswer {
    pub number: u8,
    /// Finished cooks at this level.
    pub cooks: u32,
    pub cooks_to_complete: u32,
    /// Recipes of this level may be offered.
    pub open: bool,
    /// The cook's experience answer completes this level without a cook, and without XP.
    pub granted: bool,
    pub complete: bool,
}

impl SkillAnswer {
    pub fn of(progress: &SkillProgress) -> SkillAnswer {
        SkillAnswer {
            skill: progress.skill.clone(),
            status: SkillStatusAnswer::of(&progress.status()),
            missing_prerequisites: progress.missing_prerequisites.clone(),
            current_level: progress.current_level(),
            levels: progress.levels.iter().map(LevelAnswer::of).collect(),
        }
    }
}

impl SkillStatusAnswer {
    fn of(status: &SkillStatus) -> SkillStatusAnswer {
        match status {
            SkillStatus::Locked { .. } => SkillStatusAnswer::Locked,
            SkillStatus::Served => SkillStatusAnswer::Served,
            SkillStatus::Mastered => SkillStatusAnswer::Mastered,
        }
    }
}

impl LevelAnswer {
    fn of(level: &LevelProgress) -> LevelAnswer {
        LevelAnswer {
            number: level.number,
            cooks: level.cooks,
            cooks_to_complete: COOKS_TO_COMPLETE_LEVEL,
            open: level.is_open,
            granted: level.is_granted,
            complete: level.is_complete(),
        }
    }
}
