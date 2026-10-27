//! A cook entry as the cook log stores it, read back into what the rules need.

use super::{CookEntry, Outcome, QuizAnswers};
use crate::cook_entries::{CookOutcome, StoredEntry};
use crate::kernel::calendar::known_zone;
use crate::kernel::ids::SavedId;
use jiff::Timestamp;

/// A stored row the rules cannot read. Only a row this server did not write can be one, so progress refuses it
/// rather than leave a cook out.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("a stored cook entry could not be read")]
pub struct UnreadableStoredEntry;

impl StoredEntry {
    pub fn read(&self, answers: &QuizAnswers) -> Result<CookEntry, UnreadableStoredEntry> {
        Ok(CookEntry {
            id: self.id,
            skill: SavedId::parse(&self.skill).map_err(|_| UnreadableStoredEntry)?,
            level: u8::try_from(self.level).map_err(|_| UnreadableStoredEntry)?,
            outcome: self.read_outcome(answers)?,
            cooked_at: Timestamp::from_microsecond(self.cooked_at_microseconds).map_err(|_| UnreadableStoredEntry)?,
            time_zone: known_zone(&self.time_zone).ok_or(UnreadableStoredEntry)?,
            weekly_goal: u8::try_from(self.weekly_goal).map_err(|_| UnreadableStoredEntry)?,
        })
    }

    /// The quiz result comes from the first answers recorded under this entry.
    fn read_outcome(&self, answers: &QuizAnswers) -> Result<Outcome, UnreadableStoredEntry> {
        if self.outcome == CookOutcome::Finished.as_str() {
            return Ok(Outcome::Finished { quiz_passed: answers.passed(self.id, &self.recipe_id) });
        }
        if self.outcome == CookOutcome::StoppedEarly.as_str() {
            return Ok(Outcome::StoppedEarly);
        }
        Err(UnreadableStoredEntry)
    }
}
