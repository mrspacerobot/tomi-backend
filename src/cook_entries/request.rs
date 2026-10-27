//! One cook entry from the phone with the cook's calendar, read at the edge.

use super::zone::known_zone;
use serde::Deserialize;
use serde_json::Value;

/// The request as sent. The entry stays raw so it can receive a specific result.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EntryRequest {
    calendar: CalendarFile,
    entry: Value,
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

/// A request with a checked calendar.
pub struct CheckedRequest {
    pub calendar: RequestCalendar,
    pub entry: Value,
}

impl CheckedRequest {
    /// The request, or why it is refused as an `invalid_request`.
    pub fn parse(file: EntryRequest) -> Result<CheckedRequest, Vec<String>> {
        let calendar = RequestCalendar::parse(file.calendar)?;
        Ok(CheckedRequest { calendar, entry: file.entry })
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
