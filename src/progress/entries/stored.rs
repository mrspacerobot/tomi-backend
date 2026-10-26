//! A cook entry as the cook log keeps it, read back into what the rules need.

use super::entry::{CookEntry, Outcome};
use crate::kernel::ids::SavedId;
use crate::progress::EntryOutcome;
use uuid::Uuid;

/// The columns of a stored cook entry that progress reads, as the database holds them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredCook {
    pub id: Uuid,
    pub skill: String,
    pub level: i16,
    pub outcome: String,
}

impl StoredCook {
    /// The cook as the rules read it, or nothing when a column holds what no accepted entry could.
    /// The quiz result is not derived yet (mrspacerobot/tomi-backend#44), so no cook earns the bonus.
    pub fn read(self) -> Option<CookEntry> {
        let skill = SavedId::parse(&self.skill).ok()?;
        let level = u8::try_from(self.level).ok()?;
        if level < 1 {
            return None;
        }
        let outcome = outcome(&self.outcome)?;
        Some(CookEntry { id: self.id, skill, level, outcome })
    }
}

/// The stored word, the same one `POST /v1/cook-entries` stores.
fn outcome(stored: &str) -> Option<Outcome> {
    if stored == EntryOutcome::Finished.as_str() {
        return Some(Outcome::Finished { quiz_passed: false });
    }
    if stored == EntryOutcome::StoppedEarly.as_str() {
        return Some(Outcome::StoppedEarly);
    }
    None
}

#[cfg(test)]
mod tests;
