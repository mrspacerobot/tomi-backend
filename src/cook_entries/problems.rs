//! Why a cook entry was rejected. The codes are permanent once released; the phone logs them and never retries.

use serde::Serialize;

/// One reason a cook entry is not stored, in the order `docs/progress.md` lists them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EntryProblem {
    /// `schemaVersion` is not one this server reads. Nothing else of the entry is read.
    UnsupportedSchemaVersion,
    /// The entry or its dish has a field this schema does not have, such as `quizCorrect`, `xp` or a total.
    UnknownField,
    /// A field is missing or of the wrong shape, so the entry cannot be read at all.
    UnreadableEntry,
    InvalidSkillId,
    LevelOutOfRange,
    WeeklyGoalOutOfRange,
    CookedInFuture,
    InvalidTimeZone,
    RecipeDoesNotTeachLevel,
}
