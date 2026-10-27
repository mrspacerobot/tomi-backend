//! A cook entry as the cook log stores it, read back into what the rules need.

use super::{CookEntry, Outcome};
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
    pub fn read(&self) -> Result<CookEntry, UnreadableStoredEntry> {
        Ok(CookEntry {
            id: self.id,
            skill: SavedId::parse(&self.skill).map_err(|_| UnreadableStoredEntry)?,
            level: u8::try_from(self.level).map_err(|_| UnreadableStoredEntry)?,
            outcome: self.read_outcome()?,
            cooked_at: Timestamp::from_microsecond(self.cooked_at_microseconds).map_err(|_| UnreadableStoredEntry)?,
            time_zone: TimeZone::get(&self.time_zone).map_err(|_| UnreadableStoredEntry)?,
            weekly_goal: u8::try_from(self.weekly_goal).map_err(|_| UnreadableStoredEntry)?,
        })
    }

    /// The quiz bonus is not earned yet: the quiz result will be derived from the answers recorded under the
    /// entry's id (mrspacerobot/tomi-backend#44), never sent by the phone.
    fn read_outcome(&self) -> Result<Outcome, UnreadableStoredEntry> {
        if self.outcome == EntryOutcome::Finished.as_str() {
            return Ok(Outcome::Finished { quiz_passed: false });
        }
        if self.outcome == EntryOutcome::StoppedEarly.as_str() {
            return Ok(Outcome::StoppedEarly);
        }
        Err(UnreadableStoredEntry)
    }
}
