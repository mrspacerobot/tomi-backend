//! The week the snapshot reports: from the first moment of its first day to the first moment of the next week's,
//! on the clocks of the request's zone, so it is 167 or 169 hours long across a daylight saving change.

use super::{utc, Request};
use crate::progress::fixtures::{local, march, october, september};
use jiff::{SignedDuration, Timestamp};

fn week_of(request: &Request) -> (Timestamp, Timestamp) {
    let streak = request.snapshot(&[]).streak;
    (streak.week_starts_at, streak.week_ends_at)
}

/// Clocks go back at 3 am on Sunday 25 October: the week of Monday 19 October begins on summer time (UTC+2) and
/// ends at midnight on winter time (UTC+1).
#[test]
fn week_ends_at_is_the_end_of_the_week_in_the_request_zone_after_summer_time_ends() {
    let request = Request::new("Europe/Amsterdam", 2, local("Europe/Amsterdam", october(21), 12).timestamp());
    let (starts_at, ends_at) = week_of(&request);
    assert_eq!(starts_at, utc(october(18), (22, 0, 0)));
    assert_eq!(ends_at, utc(october(25), (23, 0, 0)));
    assert_eq!(ends_at.duration_since(starts_at), SignedDuration::from_hours(169));
}

/// Clocks go forward at 2 am on Sunday 8 March: the Sunday-first week of 8 March begins on standard time (UTC-5)
/// and ends at midnight on daylight time (UTC-4).
#[test]
fn week_ends_at_is_the_end_of_the_week_in_the_request_zone_after_daylight_saving_starts() {
    let request = Request::new("America/New_York", 1, local("America/New_York", march(10), 12).timestamp());
    let (starts_at, ends_at) = week_of(&request);
    assert_eq!(starts_at, utc(march(8), (5, 0, 0)));
    assert_eq!(ends_at, utc(march(15), (4, 0, 0)));
    assert_eq!(ends_at.duration_since(starts_at), SignedDuration::from_hours(167));
}

/// The last moment of a week still belongs to it, and the week's end is the next week's start.
#[test]
fn the_week_ends_where_the_next_one_starts() {
    let last_moment = utc(october(25), (22, 59, 59));
    let (_, ends_at) = week_of(&Request::new("Europe/Amsterdam", 2, last_moment));
    let (next_starts_at, _) = week_of(&Request::new("Europe/Amsterdam", 2, ends_at));
    assert_eq!(ends_at, utc(october(25), (23, 0, 0)));
    assert_eq!(next_starts_at, ends_at);
}

/// In Santiago clocks skip from midnight to 1 am on Sunday 6 September, so a Sunday-first week starts at 1 am.
#[test]
fn a_week_whose_first_midnight_is_skipped_starts_when_its_first_day_does() {
    let request = Request::new("America/Santiago", 1, local("America/Santiago", september(9), 12).timestamp());
    let (starts_at, _) = week_of(&request);
    assert_eq!(starts_at, utc(september(6), (4, 0, 0)));
}
