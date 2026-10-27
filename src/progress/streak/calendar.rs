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

/// The instants a week of the cook's calendar begins and ends at.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WeekBounds {
    /// The first moment of the week's first day.
    pub starts_at: Timestamp,
    /// The first moment of the next week's first day: the week ends just before it.
    pub ends_at: Timestamp,
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
        let Ok(time_zone) = TimeZone::get(time_zone) else {
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

    /// The zone's IANA name, as the request named it.
    pub fn time_zone_name(&self) -> &str {
        self.time_zone.iana_name().unwrap_or_default()
    }

    /// The first weekday in Foundation's numbering, 1 for Sunday to 7 for Saturday.
    pub fn first_weekday_number(&self) -> i8 {
        self.first_weekday.to_sunday_one_offset()
    }

    /// The week in progress at `now`, from the first moment of its first day to the first moment of the next
    /// week's, on this zone's clocks: a week across a daylight saving change is an hour longer or shorter.
    pub fn week_of(&self, now: Timestamp) -> WeekBounds {
        let first_day = self.first_day_of_week(self.day_of(now));
        let next_first_day = first_day.saturating_add(1.week());
        WeekBounds { starts_at: self.first_moment_of(first_day), ends_at: self.first_moment_of(next_first_day) }
    }

    /// The first moment of `day` on this zone's clocks: midnight, or the end of a gap that skips it.
    fn first_moment_of(&self, day: Date) -> Timestamp {
        let first_moment = day.to_zoned(self.time_zone.clone()).and_then(|moment| moment.start_of_day());
        match first_moment {
            Ok(moment) => moment.timestamp(),
            // Only a day at the very edge of jiff's range has no first moment, and no request time is one.
            Err(_) => Timestamp::MAX,
        }
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
