//! The cook's calendar: their IANA time zone and the weekday their week starts on, parsed once.

use jiff::civil::{Date, Weekday};
use jiff::tz::TimeZone;
use jiff::{Timestamp, ToSpan};

/// Days in every week of the Gregorian calendar, whatever its first weekday.
const DAYS_IN_A_WEEK: i8 = 7;

/// The time zone and first weekday a request names. Weeks are found by calendar days, never by
/// adding up hours, so a week across a daylight saving change is still seven of the cook's days.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CookCalendar {
    time_zone: TimeZone,
    first_weekday: Weekday,
}

/// Why a calendar from a request was refused. There is no default zone: a guess would move cooks
/// to the wrong day.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum ParseCalendarError {
    #[error("the time zone is not a known IANA zone")]
    UnknownTimeZone,
    #[error("the first weekday is not 1 (Sunday) to 7 (Saturday)")]
    FirstWeekdayOutOfRange,
}

impl CookCalendar {
    /// Reads a zone such as `Europe/Amsterdam` and a first weekday in Foundation's numbering,
    /// 1 for Sunday to 7 for Saturday.
    pub fn parse(time_zone: &str, first_weekday: i64) -> Result<Self, ParseCalendarError> {
        let Some(time_zone) = known_zone(time_zone) else {
            return Err(ParseCalendarError::UnknownTimeZone);
        };
        let first_weekday = weekday_from_sunday_one(first_weekday)?;
        Ok(CookCalendar { time_zone, first_weekday })
    }

    pub fn time_zone(&self) -> &TimeZone {
        &self.time_zone
    }

    pub fn first_weekday(&self) -> Weekday {
        self.first_weekday
    }

    /// The day `moment` falls on in this calendar's zone.
    pub fn day_of(&self, moment: Timestamp) -> Date {
        day_in_zone(moment, &self.time_zone)
    }

    /// The first day of the week that holds `day`.
    pub fn first_day_of_week(&self, day: Date) -> Date {
        let days_since_start = day.weekday().since(self.first_weekday);
        day.saturating_sub(days_since_start.days())
    }

    /// The first day of the week before the one that starts on `first_day`.
    pub fn first_day_of_week_before(&self, first_day: Date) -> Date {
        self.first_day_of_week(first_day.saturating_sub(1.week()))
    }

    /// Days left in the week of `day`, that day included: the last day of the week gives 1.
    pub fn days_left_in_week(&self, day: Date) -> u32 {
        let days_since_start = day.weekday().since(self.first_weekday);
        u32::from((DAYS_IN_A_WEEK - days_since_start).unsigned_abs())
    }
}

/// The day `moment` falls on, on the clocks of `zone`.
pub fn day_in_zone(moment: Timestamp, zone: &TimeZone) -> Date {
    moment.to_zoned(zone.clone()).date()
}

fn weekday_from_sunday_one(number: i64) -> Result<Weekday, ParseCalendarError> {
    let Ok(small) = i8::try_from(number) else {
        return Err(ParseCalendarError::FirstWeekdayOutOfRange);
    };
    match Weekday::from_sunday_one_offset(small) {
        Ok(weekday) => Ok(weekday),
        Err(_) => Err(ParseCalendarError::FirstWeekdayOutOfRange),
    }
}

/// A named IANA time zone, or none for offsets and names the database does not know.
pub fn known_zone(name: &str) -> Option<TimeZone> {
    let zone = TimeZone::get(name).ok()?;
    zone.iana_name()?;
    Some(zone)
}
