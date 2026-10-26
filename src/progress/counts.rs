//! How many finished cooks the log holds for each level of each skill.

use super::log::{finished_cooks, CookEntry};
use crate::kernel::ids::SavedId;
use std::collections::BTreeMap;

/// Finished cooks per skill and level. A cook counts at the level it taught, whether or not that
/// level was open at the time, so the counts depend only on which entries exist, never on their
/// order. A skill or level the catalog does not know is counted here and simply never asked for.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct CookCounts(BTreeMap<(SavedId, u8), u32>);

impl CookCounts {
    pub fn from_log(log: &[CookEntry]) -> Self {
        let mut counts = BTreeMap::new();
        for cook in finished_cooks(log) {
            let cooks_so_far = counts.entry((cook.skill.clone(), cook.level)).or_insert(0);
            *cooks_so_far += 1;
        }
        CookCounts(counts)
    }

    /// Finished cooks of this skill at this level.
    pub fn cooks(&self, skill: &SavedId, level: u8) -> u32 {
        match self.0.get(&(skill.clone(), level)) {
            Some(cooks) => *cooks,
            None => 0,
        }
    }
}
