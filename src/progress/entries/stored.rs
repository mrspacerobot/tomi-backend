//! A cook entry as the cook log stores it, read back into what the rules need, with its derived quiz result.

use super::{CookEntry, Outcome, QuizAnswers};
use crate::kernel::ids::SavedId;
use crate::progress::EntryOutcome;
use jiff::tz::TimeZone;
use jiff::Timestamp;
use uuid::Uuid;

/// One row of the cook log, column by column.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredEntry {
    pub id: Uuid,
    pub skill: String,
    pub level: i16,
    /// The graph recipe cooked, or none for a lesson of the old development logs.
    pub recipe_id: Option<String>,
    pub outcome: String,
    pub weekly_goal: i16,
    /// The instant the cook was made, in microseconds since 1970 in UTC, as the database keeps it.
    pub cooked_at_microseconds: i64,
    pub time_zone: String,
}

/// A stored row the rules cannot read. Only a row this server did not write can be one, so progress refuses it
/// rather than leave a cook out.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("a stored cook entry could not be read")]
pub struct UnreadableStoredEntry;

impl StoredEntry {
    /// The cook, with the quiz result derived from the cook's recorded first answers.
    pub fn read(&self, answers: &QuizAnswers) -> Result<CookEntry, UnreadableStoredEntry> {
        Ok(CookEntry {
            id: self.id,
            skill: SavedId::parse(&self.skill).map_err(|_| UnreadableStoredEntry)?,
            level: u8::try_from(self.level).map_err(|_| UnreadableStoredEntry)?,
            outcome: self.read_outcome(answers)?,
            cooked_at: Timestamp::from_microsecond(self.cooked_at_microseconds).map_err(|_| UnreadableStoredEntry)?,
            time_zone: TimeZone::get(&self.time_zone).map_err(|_| UnreadableStoredEntry)?,
            weekly_goal: u8::try_from(self.weekly_goal).map_err(|_| UnreadableStoredEntry)?,
        })
    }

    /// The quiz result is derived from the answers recorded under the entry's id, never sent by the phone.
    fn read_outcome(&self, answers: &QuizAnswers) -> Result<Outcome, UnreadableStoredEntry> {
        if self.outcome == EntryOutcome::Finished.as_str() {
            let quiz_passed = answers.passed(self.id, self.recipe_id.as_deref());
            return Ok(Outcome::Finished { quiz_passed });
        }
        if self.outcome == EntryOutcome::StoppedEarly.as_str() {
            return Ok(Outcome::StoppedEarly);
        }
        Err(UnreadableStoredEntry)
    }
}
