//! Which zone decides a cook's day (the owner's decision, issue #37): a week before the current one
//! counts each cook on its day in the zone it was made in; the week in progress counts it on its day
//! in the zone the request names. Weeks start on Monday and the goal is 2 unless a test says.

use super::test_calendar::{TestCalendar, TEST_GOAL};
use crate::progress::fixtures::{cook, local, local_time, made_at, october, september, with_weekly_goal};
use crate::progress::CookEntry;
use jiff::Zoned;

fn made(moment: &Zoned) -> CookEntry {
    with_weekly_goal(&made_at(&cook("knife", 1), moment), TEST_GOAL)
}

/// 23:30 on Sunday 4 October in New York is already Monday 5 October in UTC: it belongs to the week
/// of 28 September.
#[test]
fn a_cook_at_half_past_eleven_counts_on_its_local_day_not_the_utc_day() {
    let new_york = TestCalendar::with("America/New_York", 2);
    let late_sunday = made(&local_time("America/New_York", october(4), (23, 30)));
    let log = [new_york.cooks(&[september(28)]), vec![late_sunday]].concat();
    let streak = new_york.streak(&log, october(7));
    assert_eq!((streak.weeks_before_this_week, streak.days_this_week), (1, 0));
}

/// 00:30 on Monday 5 October in Tokyo is still Sunday 4 October in UTC: in the week in progress it
/// counts on the Tokyo Monday.
#[test]
fn a_cook_just_after_midnight_counts_in_the_week_in_progress_on_its_local_day() {
    let tokyo = TestCalendar::with("Asia/Tokyo", 2);
    let early_monday = made(&local_time("Asia/Tokyo", october(5), (0, 30)));
    let streak = tokyo.streak(&[early_monday], october(7));
    assert_eq!((streak.weeks_before_this_week, streak.days_this_week), (0, 1));
}

/// 1 am on Monday 28 September in Tokyo is still Sunday 27 September in Amsterdam: on its Tokyo day
/// it shares a week with the Friday cooked at home.
#[test]
fn a_past_week_cook_made_in_tokyo_counts_on_its_tokyo_day() {
    let amsterdam = TestCalendar::with("Europe/Amsterdam", 2);
    let tokyo_monday = made(&local("Asia/Tokyo", september(28), 1));
    let log = [vec![tokyo_monday], amsterdam.cooks(&[october(2)])].concat();
    assert_eq!(amsterdam.streak(&log, october(7)).weeks_before_this_week, 1);
}

/// 00:30 on Wednesday 7 October in Tokyo is Tuesday 6 October in Amsterdam, where the cook asks
/// from: in the week in progress it counts on the Amsterdam Tuesday, beside the Amsterdam Wednesday.
#[test]
fn a_cook_in_the_current_week_counts_on_its_day_in_the_request_zone() {
    let amsterdam = TestCalendar::with("Europe/Amsterdam", 2);
    let tokyo_wednesday = made(&local_time("Asia/Tokyo", october(7), (0, 30)));
    let log = [vec![tokyo_wednesday], amsterdam.cooks(&[october(7)])].concat();
    let streak = amsterdam.streak(&log, october(7));
    assert_eq!(streak.days_this_week, 2);
    assert!(streak.is_this_week_met());
}

/// 1 am on Monday 5 October in Tokyo is Sunday 4 October in Amsterdam, before the week in progress
/// there. It counts once, on its Tokyo Monday, which is a day of the week in progress.
#[test]
fn a_cook_before_the_current_week_whose_own_day_falls_in_it_counts_once_on_its_own_day() {
    let amsterdam = TestCalendar::with("Europe/Amsterdam", 2);
    let tokyo_monday = made(&local("Asia/Tokyo", october(5), 1));
    let log = [vec![tokyo_monday], amsterdam.cooks(&[september(28)])].concat();
    let streak = amsterdam.streak(&log, october(7));
    assert_eq!((streak.weeks_before_this_week, streak.days_this_week), (0, 1));
}

/// A cook who travels: weeks met in New York, Tokyo and Amsterdam chain into one streak, each cook
/// on the day it had where it was made, and the request's zone does not move the past.
#[test]
fn a_cook_who_travels_between_zones_keeps_each_past_cook_on_its_own_day() {
    let new_york_sunday_night = made(&local_time("America/New_York", september(27), (23, 0)));
    let new_york_monday = made(&local("America/New_York", september(21), 12));
    let tokyo_monday_morning = made(&local("Asia/Tokyo", september(28), 7));
    let tokyo_thursday = made(&local("Asia/Tokyo", october(1), 20));
    let log = vec![new_york_monday, new_york_sunday_night, tokyo_monday_morning, tokyo_thursday];
    for (zone, first_weekday) in [("Europe/Amsterdam", 2), ("America/Los_Angeles", 2), ("Pacific/Auckland", 2)] {
        let request = TestCalendar::with(zone, first_weekday);
        let streak = request.streak(&log, october(7));
        assert_eq!(streak.weeks_before_this_week, 2, "asked from {zone}");
    }
}
