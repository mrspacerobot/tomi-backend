//! One cook entry from the phone with the cook's calendar, read at the edge.

use crate::kernel::calendar::CookCalendar;
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
    first_weekday: i64,
}

/// A request with a checked calendar.
pub struct CheckedRequest {
    pub calendar: CookCalendar,
    pub entry: Value,
}

impl CheckedRequest {
    /// The request, or why it is refused as an `invalid_request`.
    pub fn parse(file: EntryRequest) -> Result<CheckedRequest, Vec<String>> {
        let calendar = parse_calendar(&file.calendar)?;
        Ok(CheckedRequest { calendar, entry: file.entry })
    }
}

fn parse_calendar(file: &CalendarFile) -> Result<CookCalendar, Vec<String>> {
    CookCalendar::parse(&file.time_zone, file.first_weekday).map_err(|error| vec![error.problem("calendar.")])
}
