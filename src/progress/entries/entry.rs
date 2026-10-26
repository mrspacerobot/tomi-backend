//! The cook log as the rules read it: any list of entries, in any order, possibly with copies.

use crate::kernel::ids::SavedId;
use jiff::tz::TimeZone;
use jiff::Timestamp;
use std::collections::BTreeSet;
use uuid::Uuid;

/// One cook the cook made, as the rules need it. The skill and level are the entry's own, so a cook
/// counts where it was cooked whatever dish it was and whether or not the catalog knows them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CookEntry {
    /// The id the phone gave the cook. The same id twice is the same cook.
    pub id: Uuid,
    pub skill: SavedId,
    pub level: u8,
    pub outcome: Outcome,
    /// The instant the cook was made.
    pub cooked_at: Timestamp,
    /// The IANA zone the cook was made in. A week before the current one counts the cook on its day here.
    pub time_zone: TimeZone,
    /// The cook's weekly goal when the cook was made, so a later change of goal never judges a past week again.
    pub weekly_goal: u8,
}

/// How a cook ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    /// Cooked to the end. `quiz_passed` says whether it earns the quiz bonus.
    Finished { quiz_passed: bool },
    /// Abandoned before the last step. It counts for nothing.
    StoppedEarly,
}

/// The entries that count as cooks: finished, and each id once however often it appears.
/// Every rule starts here, so a log with copies gives the same answers as one without.
pub fn finished_cooks(log: &[CookEntry]) -> Vec<&CookEntry> {
    let mut seen_ids = BTreeSet::new();
    let mut finished = Vec::new();
    for entry in log.iter().filter(|entry| entry.outcome.is_finished()) {
        let is_first_copy = seen_ids.insert(entry.id);
        if is_first_copy {
            finished.push(entry);
        }
    }
    finished
}

impl Outcome {
    pub fn is_finished(self) -> bool {
        match self {
            Outcome::Finished { .. } => true,
            Outcome::StoppedEarly => false,
        }
    }
}
