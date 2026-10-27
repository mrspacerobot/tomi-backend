//! The cook's progress through the whole catalog: every skill, the focus and the mastered count.

use super::{SkillProgress, SkillStatus};
use crate::kernel::ids::SavedId;

/// The cook's progress through the whole catalog. Computed from the cook entries by `ProgressRules`,
/// never stored.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatalogProgress {
    /// One entry per skill in the catalog, in catalog order.
    pub skills: Vec<SkillProgress>,
}

impl CatalogProgress {
    pub fn skill(&self, id: &SavedId) -> Option<&SkillProgress> {
        self.skills.iter().find(|skill| &skill.skill == id)
    }

    /// The skill to cook next: the cook's choice while it is served and not mastered,
    /// otherwise the first such skill in catalog order. None when nothing is left to cook.
    pub fn focus(&self, chosen: Option<&SavedId>) -> Option<SavedId> {
        if let Some(chosen) = chosen {
            if self.can_be_focus(chosen) {
                return Some(chosen.clone());
            }
        }
        let first_in_catalog_order = self.skills.iter().find(|skill| skill.status() == SkillStatus::Served)?;
        Some(first_in_catalog_order.skill.clone())
    }

    fn can_be_focus(&self, id: &SavedId) -> bool {
        match self.skill(id) {
            Some(progress) => progress.status() == SkillStatus::Served,
            None => false,
        }
    }

    /// How many skills of the catalog the cook has mastered, by cooking or by their experience answer.
    pub fn mastered_count(&self) -> usize {
        self.skills.iter().filter(|skill| skill.is_mastered()).count()
    }
}
