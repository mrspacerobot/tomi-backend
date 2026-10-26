//! A batch of cook entries from the phone's outbox, with the cook's calendar, read once at the edge.

use crate::progress::{CookCalendar, ParseCalendarError};
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
    first_weekday: i64,
}

/// A batch that may be answered entry by entry.
pub struct EntryBatch {
    pub calendar: CookCalendar,
    pub entries: Vec<Value>,
}

impl EntryBatch {
    /// The batch, or why it is refused as a whole, as the problems of an `invalid_request`.
    pub fn parse(file: BatchFile) -> Result<EntryBatch, Vec<String>> {
        if file.entries.len() > MAX_ENTRIES_PER_BATCH {
            return Err(vec![format!("entries: at most {MAX_ENTRIES_PER_BATCH} in one batch")]);
        }
        let calendar = parse_calendar(&file.calendar)?;
        Ok(EntryBatch { calendar, entries: file.entries })
    }
}

/// The calendar, or the problem of an `invalid_request` that names the field at fault.
fn parse_calendar(file: &CalendarFile) -> Result<CookCalendar, Vec<String>> {
    match CookCalendar::parse(&file.time_zone, file.first_weekday) {
        Ok(calendar) => Ok(calendar),
        Err(ParseCalendarError::UnknownTimeZone) => {
            Err(vec!["calendar.timeZone: not a known IANA time zone".to_string()])
        }
        Err(ParseCalendarError::FirstWeekdayOutOfRange) => {
            Err(vec!["calendar.firstWeekday: from 1 (Sunday) to 7 (Saturday)".to_string()])
        }
    }
}
