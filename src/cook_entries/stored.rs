//! A cook log row as stored in the database.

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
