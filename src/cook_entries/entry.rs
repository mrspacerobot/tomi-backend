//! One cook entry as the phone sends it, read once into a type that cannot hold a bad value.

use super::dish::{Dish, DishFile};
use super::fields::Fields;
use super::problems::EntryProblem;
use crate::graph::{sha256_hex, Graph};
use crate::kernel::ids::SavedId;
use jiff::Timestamp;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use uuid::Uuid;

/// The only entry layout this server reads.
pub const ENTRY_SCHEMA_VERSION: u16 = 1;

/// A cook entry that may be stored. The time zone is the canonical IANA name of a zone the database knows.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReceivedEntry {
    pub schema_version: u16,
    pub id: Uuid,
    pub cooked_at: Timestamp,
    pub time_zone: String,
    pub dish: Dish,
    pub skill: SavedId,
    pub level: u8,
    pub outcome: EntryOutcome,
    pub weekly_goal: u8,
}

/// How the cook ended, as the phone says it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum EntryOutcome {
    Finished,
    StoppedEarly,
}

impl EntryOutcome {
    /// The stored word, the same as the JSON one.
    pub fn as_str(self) -> &'static str {
        match self {
            EntryOutcome::Finished => "finished",
            EntryOutcome::StoppedEarly => "stoppedEarly",
        }
    }
}

/// What an entry is checked against besides itself: the time it arrived and the pinned graph, if any.
pub struct EntryChecks<'a> {
    pub received_at: Timestamp,
    pub graph: Option<&'a Graph>,
}

/// The entry as sent. Unknown fields are collected, not ignored, so an entry carrying a quiz result, XP or a total
/// is refused.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct EntryFile {
    id: Uuid,
    cooked_at: String,
    time_zone: String,
    dish: DishFile,
    skill: String,
    level: i64,
    outcome: EntryOutcome,
    weekly_goal: i64,
    /// Read and checked before the rest; named here only so it is not taken for an unknown field.
    #[serde(rename = "schemaVersion")]
    _schema_version: u64,
    #[serde(flatten)]
    unknown: BTreeMap<String, Value>,
}

/// The entry's id, when it has a readable one, so even a rejected entry is answered by its id.
pub fn id_of(raw: &Value) -> Option<Uuid> {
    let text = raw.get("id")?.as_str()?;
    Uuid::parse_str(text).ok()
}

impl ReceivedEntry {
    /// Reads an entry, or names every problem that keeps it from being stored. An unknown schema version is
    /// refused before anything else in the entry is read.
    pub fn read(raw: &Value, checks: &EntryChecks) -> Result<ReceivedEntry, Vec<EntryProblem>> {
        let version = raw.get("schemaVersion").and_then(Value::as_u64);
        if version != Some(u64::from(ENTRY_SCHEMA_VERSION)) {
            return Err(vec![EntryProblem::UnsupportedSchemaVersion]);
        }
        match readable(raw) {
            Ok(fields) => fields.checked(checks),
            Err(problem) => Err(vec![problem]),
        }
    }

    /// A fingerprint of what the entry says, the same however the phone wrote it: the instant in UTC, the zone by
    /// its canonical name, the fields in one fixed order.
    pub fn payload_sha256(&self) -> Result<String, serde_json::Error> {
        let canonical = serde_json::to_string(self)?;
        Ok(sha256_hex(&canonical))
    }
}

/// The entry's fields with their shapes checked, or why it cannot be read at all.
fn readable(raw: &Value) -> Result<Fields, EntryProblem> {
    let Ok(file) = serde_json::from_value::<EntryFile>(raw.clone()) else {
        return Err(EntryProblem::UnreadableEntry);
    };
    if !file.unknown.is_empty() {
        return Err(EntryProblem::UnknownField);
    }
    let dish = Dish::parse(file.dish)?;
    let Ok(cooked_at) = file.cooked_at.parse::<Timestamp>() else {
        return Err(EntryProblem::UnreadableEntry);
    };
    let (id, time_zone, skill, outcome) = (file.id, file.time_zone, file.skill, file.outcome);
    let (level, weekly_goal) = (file.level, file.weekly_goal);
    Ok(Fields { id, cooked_at, time_zone, dish, skill, level, outcome, weekly_goal })
}
