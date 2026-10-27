//! The outcome of submitting a cook entry.

use super::problems::EntryProblem;
use serde::Serialize;

/// What became of an entry. Only a rejected entry has problems, and it always has at least one.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "status", rename_all = "camelCase")]
pub enum SubmissionOutcome {
    /// Stored now.
    Accepted,
    /// The same id with the same body was already stored.
    Duplicate,
    /// The same id with a different body was already stored. The first copy stays and nothing changes.
    Conflict,
    /// Not stored, for these permanent reasons.
    Rejected { problems: Vec<EntryProblem> },
}

impl SubmissionOutcome {
    /// What a copy of an entry already stored is: a duplicate when it says the same, a conflict otherwise.
    pub fn of_copy(stored_sha256: &str, sent_sha256: &str) -> SubmissionOutcome {
        if stored_sha256 == sent_sha256 {
            return SubmissionOutcome::Duplicate;
        }
        SubmissionOutcome::Conflict
    }
}
