//! Which skills are served and mastered, and which levels are open, as Tomi's `ProgressRules`.
//!
//! A skill is served once the first level of every prerequisite is complete. Its level 1 then
//! opens, and each later level opens once every level before it is complete. "Comfortable" in
//! onboarding counts every skill of the first course as complete, without a cook and without XP.
//! The catalog is an argument, so the rules hold for any menu and never name a skill.

use super::counts::CookCounts;
use super::game::is_level_complete;
use super::log::CookEntry;
use super::menu::CookProgress;
use super::skill::{LevelProgress, SkillProgress};
use crate::kernel::ids::SavedId;
use crate::library::{Catalog, Skill};
use crate::profile::Experience;
use std::collections::BTreeSet;

/// Works out the cook's progress through one catalog, for one experience answer.
pub struct ProgressRules<'a> {
    catalog: &'a Catalog,
    /// Skills the experience answer covers. Every level of one is complete without a cook, so
    /// nothing fake is ever written to the cook log, and a grant earns no XP.
    granted: BTreeSet<SavedId>,
}

impl<'a> ProgressRules<'a> {
    pub fn new(catalog: &'a Catalog, experience: Experience) -> Self {
        ProgressRules { catalog, granted: granted_skills(catalog, experience) }
    }

    /// Progress for every skill on the menu, from a log in any order and with any copies.
    pub fn progress(&self, log: &[CookEntry]) -> CookProgress {
        self.progress_from_counts(&CookCounts::from_log(log))
    }

    /// Progress for every skill on the menu, in menu order. Counts for a skill or level the
    /// catalog does not know are left out, because the log may hold cooks from a newer menu.
    pub fn progress_from_counts(&self, counts: &CookCounts) -> CookProgress {
        let skills = self.catalog.skills.iter().map(|skill| self.skill_progress(skill, counts)).collect();
        CookProgress { skills }
    }

    fn skill_progress(&self, skill: &Skill, counts: &CookCounts) -> SkillProgress {
        let missing = self.missing_prerequisites(skill, counts);
        let start = LevelStart { is_served: missing.is_empty(), is_granted: self.granted.contains(&skill.id) };
        let levels = levels(skill, counts, start);
        SkillProgress { skill: skill.id.clone(), levels, missing_prerequisites: missing }
    }

    /// Prerequisites whose first level is not complete, in catalog order. Asking for every level
    /// would hold the next techniques back for weeks.
    fn missing_prerequisites(&self, skill: &Skill, counts: &CookCounts) -> Vec<SavedId> {
        let missing =
            skill.prerequisites.iter().filter(|prerequisite| !self.is_first_level_complete(prerequisite, counts));
        missing.cloned().collect()
    }

    fn is_first_level_complete(&self, id: &SavedId, counts: &CookCounts) -> bool {
        if self.granted.contains(id) {
            return true;
        }
        let Some(skill) = self.catalog.skill(id) else {
            return false;
        };
        let Some(first_level) = skill.levels.first() else {
            return false;
        };
        is_level_complete(counts.cooks(id, first_level.number))
    }
}

/// What decides a skill's levels besides its cooks.
struct LevelStart {
    is_served: bool,
    is_granted: bool,
}

/// Level 1 is open once the skill is served. Each later level opens when every level before it is
/// complete.
fn levels(skill: &Skill, counts: &CookCounts, start: LevelStart) -> Vec<LevelProgress> {
    let is_granted = start.is_granted;
    let mut is_open = start.is_served;
    let mut levels = Vec::new();
    for level in &skill.levels {
        let cooks = counts.cooks(&skill.id, level.number);
        let progress = LevelProgress { number: level.number, cooks, is_open, is_granted };
        is_open = is_open && progress.is_complete();
        levels.push(progress);
    }
    levels
}

/// Every skill of the first course in the catalog, for a cook who answered "comfortable".
/// "Never" and "basics" differ only in how much Tomi explains, so neither grants anything.
pub fn granted_skills(catalog: &Catalog, experience: Experience) -> BTreeSet<SavedId> {
    if experience != Experience::Comfortable {
        return BTreeSet::new();
    }
    let Some(first_course) = catalog.courses.first() else {
        return BTreeSet::new();
    };
    let in_course = catalog.skills.iter().filter(|skill| skill.course == first_course.id);
    in_course.map(|skill| skill.id.clone()).collect()
}

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

/// Every open level of the menu, for the cook's experience answer and finished cooks.
pub fn open_levels(catalog: &Catalog, experience: Experience, counts: &CookCounts) -> OpenLevels {
    let progress = ProgressRules::new(catalog, experience).progress_from_counts(counts);
    let mut open = BTreeSet::new();
    for skill in &progress.skills {
        let open_here = skill.levels.iter().filter(|level| level.is_open);
        open.extend(open_here.map(|level| (skill.skill.clone(), level.number)));
    }
    OpenLevels(open)
}
