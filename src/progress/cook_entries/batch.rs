//! A batch of cook entries from the phone's outbox, with the cook's calendar, read once at the edge.

use super::zone::known_zone;
use serde::Deserialize;
use serde_json::Value;

/// More than this in one request is refused as a whole; the phone sends its outbox in parts.
pub const MAX_ENTRIES_PER_BATCH: usize = 100;

/// The request as sent. Each entry stays raw here, so one bad entry is answered on its own and never refuses the
/// others.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BatchFile {
    calendar: CalendarFile,
    entries: Vec<Value>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct CalendarFile {
    time_zone: String,
    first_weekday: u8,
}

/// The cook's calendar on the request: a zone the database knows, by its canonical name, and the first weekday,
/// 1 (Sunday) to 7 (Saturday), as Foundation numbers them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequestCalendar {
    pub time_zone: String,
    pub first_weekday: u8,
}

/// A batch that may be answered entry by entry.
pub struct EntryBatch {
    pub calendar: RequestCalendar,
    pub entries: Vec<Value>,
}

impl EntryBatch {
    /// The batch, or why it is refused as a whole, as the problems of an `invalid_request`.
    pub fn parse(file: BatchFile) -> Result<EntryBatch, Vec<String>> {
        if file.entries.len() > MAX_ENTRIES_PER_BATCH {
            return Err(vec![format!("entries: at most {MAX_ENTRIES_PER_BATCH} in one batch")]);
        }
        let calendar = RequestCalendar::parse(file.calendar)?;
        Ok(EntryBatch { calendar, entries: file.entries })
    }
}

impl RequestCalendar {
    fn parse(file: CalendarFile) -> Result<RequestCalendar, Vec<String>> {
        let Some(time_zone) = known_zone(&file.time_zone) else {
            return Err(vec!["calendar.timeZone: not a known IANA time zone".to_string()]);
        };
        if !(1..=7).contains(&file.first_weekday) {
            return Err(vec!["calendar.firstWeekday: from 1 (Sunday) to 7 (Saturday)".to_string()]);
        }
        Ok(RequestCalendar { time_zone, first_weekday: file.first_weekday })
    }
}
