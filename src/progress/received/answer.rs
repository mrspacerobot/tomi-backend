//! The answer for each cook entry of a batch: stored now, already stored, conflicting, or rejected.

use super::problems::EntryProblem;
use serde::Serialize;
use uuid::Uuid;

/// One entry's answer, in the place of the entry in the batch. The id is none only when the entry had no
/// readable id.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct EntryResult {
    pub id: Option<Uuid>,
    #[serde(flatten)]
    pub status: EntryStatus,
}

/// What became of an entry. Only a rejected entry has problems, and it always has at least one.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "status", rename_all = "camelCase")]
pub enum EntryStatus {
    /// Stored now.
    Accepted,
    /// The same id with the same body was already stored.
    Duplicate,
    /// The same id with a different body was already stored. The first copy stays and nothing changes.
    Conflict,
    /// Not stored, for these permanent reasons.
    Rejected { problems: Vec<EntryProblem> },
}

impl EntryStatus {
    /// What a copy of an entry already stored is: a duplicate when it says the same, a conflict otherwise.
    pub fn of_copy(stored_sha256: &str, sent_sha256: &str) -> EntryStatus {
        if stored_sha256 == sent_sha256 {
            return EntryStatus::Duplicate;
        }
        EntryStatus::Conflict
    }
}
