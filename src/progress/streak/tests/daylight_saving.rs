//! Weeks that hold a daylight saving change, in both directions, in Europe/Amsterdam (weeks from
//! Monday) and America/New_York (weeks from Sunday). Such a week is 167 or 169 hours long, so a week
//! counted as seven times 24 hours would move a cook near midnight into the wrong week.

use super::test_calendar::{TestCalendar, TEST_GOAL};
use crate::progress::fixtures::{cook, local, local_time, made_at, march, november, october, with_weekly_goal};
use crate::progress::CookEntry;
use jiff::civil::date;
use jiff::Zoned;

fn made(moments: &[Zoned]) -> Vec<CookEntry> {
    moments.iter().map(|moment| with_weekly_goal(&made_at(&cook("knife", 1), moment), TEST_GOAL)).collect()
}

/// Clocks go back at 3 am on Sunday 25 October: the week of 19 October has 169 hours.
#[test]
fn amsterdam_the_week_summer_time_ends_keeps_its_sunday_night() {
    let amsterdam = TestCalendar::with("Europe/Amsterdam", 2);
    let monday = local("Europe/Amsterdam", october(19), 12);
    let sunday_night = local_time("Europe/Amsterdam", october(25), (23, 30));
    let log = made(&[monday, sunday_night]);
    let in_progress = amsterdam.streak_at(&log, &local_time("Europe/Amsterdam", october(25), (23, 45)));
    assert_eq!((in_progress.days_this_week, in_progress.days_left_in_week), (2, 1));
    let a_week_later = amsterdam.streak(&log, october(28));
    assert_eq!((a_week_later.weeks_before_this_week, a_week_later.days_this_week), (1, 0));
}

/// Clocks go forward at 2 am on Sunday 29 March: the week of 23 March has 167 hours, and half past
/// midnight on Monday 30 March already belongs to the next week.
#[test]
fn amsterdam_the_week_summer_time_starts_ends_at_its_own_midnight() {
    let amsterdam = TestCalendar::with("Europe/Amsterdam", 2);
    let monday = local("Europe/Amsterdam", march(23), 12);
    let sunday_night = local_time("Europe/Amsterdam", march(29), (23, 30));
    let next_monday_early = local_time("Europe/Amsterdam", march(30), (0, 30));
    let log = made(&[monday, sunday_night, next_monday_early]);
    let in_progress = amsterdam.streak(&log, march(31));
    assert_eq!((in_progress.weeks_before_this_week, in_progress.days_this_week), (1, 1));
    assert_eq!(in_progress.days_left_in_week, 6);
    let later = amsterdam.streak(&log, date(2026, 4, 8));
    assert_eq!((later.weeks_before_this_week, later.days_this_week), (0, 0));
}

/// Clocks go forward at 2 am on Sunday 8 March: the week of 8 March has 167 hours, and half past
/// midnight on Sunday 15 March already belongs to the next week.
#[test]
fn new_york_the_week_daylight_saving_starts_ends_at_its_own_midnight() {
    let new_york = TestCalendar::with("America/New_York", 1);
    let sunday = local("America/New_York", march(8), 12);
    let saturday_night = local_time("America/New_York", march(14), (23, 30));
    let next_sunday_early = local_time("America/New_York", march(15), (0, 30));
    let log = made(&[sunday, saturday_night, next_sunday_early]);
    let streak = new_york.streak(&log, march(16));
    assert_eq!((streak.weeks_before_this_week, streak.days_this_week), (1, 1));
    assert_eq!(streak.days_left_in_week, 6);
}

/// Clocks go back at 2 am on Sunday 1 November: the week of 1 November has 169 hours, and half past
/// eleven on Saturday 7 November is still in it.
#[test]
fn new_york_the_week_daylight_saving_ends_keeps_its_saturday_night() {
    let new_york = TestCalendar::with("America/New_York", 1);
    let sunday = local("America/New_York", november(1), 12);
    let saturday_night = local_time("America/New_York", november(7), (23, 30));
    let log = made(&[sunday, saturday_night]);
    let last_minutes = local_time("America/New_York", november(7), (23, 45));
    let in_progress = new_york.streak_at(&log, &last_minutes);
    assert_eq!((in_progress.days_this_week, in_progress.days_left_in_week), (2, 1));
    assert!(in_progress.is_this_week_met());
    let next_week = new_york.streak(&log, november(9));
    assert_eq!((next_week.weeks_before_this_week, next_week.days_this_week), (1, 0));
}

/// A cook made in the hour the clocks repeat counts on that Sunday, whichever of the two it was.
#[test]
fn a_cook_in_the_repeated_hour_counts_on_its_day() {
    let amsterdam = TestCalendar::with("Europe/Amsterdam", 2);
    let first_half_past_two = local_time("Europe/Amsterdam", october(25), (2, 30));
    let second_half_past_two = first_half_past_two.checked_add(jiff::Span::new().hours(1)).unwrap();
    assert_eq!(second_half_past_two.time(), first_half_past_two.time());
    let log = made(&[first_half_past_two, second_half_past_two]);
    let streak = amsterdam.streak(&log, october(28));
    assert_eq!(streak.weeks_before_this_week, 0);
    assert_eq!(amsterdam.streak(&log, october(25)).days_this_week, 1);
}
