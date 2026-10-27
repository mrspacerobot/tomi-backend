//! Ports of TomiCore's `StreakCalendarTests` (the streak follows the cook's own calendar: time zone,
//! daylight saving and first weekday), and how a calendar is read from a request.

use super::test_calendar::TestCalendar;
use crate::progress::fixtures::{november, october, september};
use crate::progress::{CookCalendar, ParseCalendarError};
use jiff::civil::Weekday;

#[test]
fn weeks_chain_across_the_end_of_daylight_saving() {
    let london = TestCalendar::with("Europe/London", 2);
    let cooks = london.cooks(&[october(19), october(25), october(26), november(1)]);
    assert_eq!(london.streak(&cooks, november(2)).weeks(), 2);
}

#[test]
fn a_sunday_start_puts_sunday_in_the_coming_week() {
    let cooks = TestCalendar::new().cooks(&[october(4), october(5)]);
    assert_eq!(TestCalendar::with("UTC", 1).streak(&cooks, october(6)).days_this_week, 2);
    assert_eq!(TestCalendar::with("UTC", 2).streak(&cooks, october(6)).days_this_week, 1);
}

/// A Saturday and the Sunday after it: one met week when weeks start on Monday, one day in each of
/// two weeks when they start on Sunday.
#[test]
fn a_sunday_first_and_a_monday_first_week_split_the_same_cooks_differently() {
    let cooks = TestCalendar::new().cooks(&[october(3), october(4)]);
    let monday_first = TestCalendar::with("UTC", 2).streak(&cooks, october(7));
    let sunday_first = TestCalendar::with("UTC", 1).streak(&cooks, october(7));
    assert_eq!((monday_first.weeks_before_this_week, monday_first.days_this_week), (1, 0));
    assert_eq!((sunday_first.weeks_before_this_week, sunday_first.days_this_week), (0, 1));
}

#[test]
fn the_days_left_include_today() {
    assert_eq!(TestCalendar::new().streak(&[], october(5)).days_left_in_week, 7);
    assert_eq!(TestCalendar::new().streak(&[], october(11)).days_left_in_week, 1);
    assert_eq!(TestCalendar::with("UTC", 1).streak(&[], october(11)).days_left_in_week, 7);
    assert_eq!(TestCalendar::with("UTC", 7).streak(&[], october(9)).days_left_in_week, 1);
}

#[test]
fn a_week_is_at_risk_when_every_day_left_is_needed() {
    let test = TestCalendar::new();
    let streak = test.with_goal(3).streak(&test.cooks(&[october(5)]), october(10));
    assert!(streak.is_at_risk());
    assert!(!streak.is_lost());
}

#[test]
fn a_week_is_lost_when_too_few_days_are_left() {
    let test = TestCalendar::new();
    let streak = test.with_goal(3).streak(&test.cooks(&[october(5)]), october(11));
    assert!(streak.is_lost());
    assert!(!streak.is_at_risk());
}

#[test]
fn a_met_week_is_neither_at_risk_nor_lost() {
    let test = TestCalendar::new();
    let streak = test.streak(&test.cooks(&[october(10), october(11)]), october(11));
    assert!(streak.is_this_week_met());
    assert!(!streak.is_at_risk() && !streak.is_lost());
}

/// The week in progress never breaks the streak: with nothing cooked yet, last week's streak stands,
/// even on the last day when this week is already lost.
#[test]
fn the_week_in_progress_never_breaks_the_streak() {
    let test = TestCalendar::new();
    let cooks = test.cooks(&[september(28), october(2)]);
    let last_day = test.streak(&cooks, october(11));
    assert!(last_day.is_lost());
    assert_eq!(last_day.weeks(), 1);
    assert_eq!(test.streak(&cooks, october(12)).weeks(), 0);
}

#[test]
fn a_calendar_is_read_from_an_iana_zone_and_a_first_weekday_from_1_to_7() {
    let calendar = CookCalendar::parse("Europe/Amsterdam", 2).unwrap();
    assert_eq!(calendar.time_zone().iana_name(), Some("Europe/Amsterdam"));
    assert_eq!(calendar.first_weekday(), Weekday::Monday);
    assert_eq!(CookCalendar::parse("Asia/Tokyo", 1).unwrap().first_weekday(), Weekday::Sunday);
    assert_eq!(CookCalendar::parse("America/New_York", 7).unwrap().first_weekday(), Weekday::Saturday);
}

#[test]
fn an_unknown_zone_is_refused_never_read_as_utc() {
    for zone in ["", "Mars/Olympus_Mons", "Europe/Amsterdam ", "+02:00", "CEST"] {
        assert_eq!(CookCalendar::parse(zone, 2), Err(ParseCalendarError::UnknownTimeZone), "{zone:?}");
    }
}

#[test]
fn a_first_weekday_outside_1_to_7_is_refused() {
    for first_weekday in [0, 8, -1, 256, i64::MAX] {
        let refused = CookCalendar::parse("Europe/Amsterdam", first_weekday);
        assert_eq!(refused, Err(ParseCalendarError::FirstWeekdayOutOfRange), "{first_weekday}");
    }
}
