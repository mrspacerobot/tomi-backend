//! HTTP response for a cook-entry submission.

use super::SubmissionOutcome;
use serde::Serialize;
use uuid::Uuid;

/// The result of one entry submission. The id is none only when the entry had no readable id.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SubmissionResult {
    pub id: Option<Uuid>,
    #[serde(flatten)]
    pub status: SubmissionOutcome,
}

/// The result returned to the phone for one submitted cook.
#[derive(Serialize)]
pub struct CookEntryResponse {
    pub result: SubmissionResult,
}
