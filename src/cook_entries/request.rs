//! One cook entry from the phone with the cook's calendar, read at the edge.

use crate::kernel::calendar::{CookCalendar, ParseCalendarError};
use serde::Deserialize;
use serde_json::Value;

/// The request as sent. The entry stays raw so it can receive a specific result.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CookEntryRequest {
    calendar: CalendarInput,
    entry: Value,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct CalendarInput {
    time_zone: String,
    first_weekday: i64,
}

/// A request with a checked calendar.
pub struct CheckedRequest {
    pub calendar: CookCalendar,
    pub entry: Value,
}

impl CheckedRequest {
    /// The request, or why it is refused as an `invalid_request`.
    pub fn parse(request: CookEntryRequest) -> Result<CheckedRequest, Vec<String>> {
        let calendar = parse_calendar(&request.calendar)?;
        Ok(CheckedRequest { calendar, entry: request.entry })
    }
}

fn parse_calendar(calendar: &CalendarInput) -> Result<CookCalendar, Vec<String>> {
    match CookCalendar::parse(&calendar.time_zone, calendar.first_weekday) {
        Ok(calendar) => Ok(calendar),
        Err(ParseCalendarError::UnknownTimeZone) => {
            Err(vec!["calendar.timeZone: not a known IANA time zone".to_string()])
        }
        Err(ParseCalendarError::FirstWeekdayOutOfRange) => {
            Err(vec!["calendar.firstWeekday: from 1 (Sunday) to 7 (Saturday)".to_string()])
        }
    }
}
