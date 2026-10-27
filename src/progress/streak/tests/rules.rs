//! Ports of TomiCore's `StreakRulesTests`. Weeks start on Monday, for example 5 October. The goal is
//! 2 cooking days unless a test says otherwise.

use super::test_calendar::TestCalendar;
use crate::progress::fixtures::{cook, made_at, october, september, with_weekly_goal};

#[test]
fn no_cooks_means_no_streak() {
    let streak = TestCalendar::new().streak(&[], october(5));
    assert_eq!(streak.weeks(), 0);
    assert_eq!(streak.days_this_week, 0);
}

#[test]
fn consecutive_met_weeks_count() {
    let test = TestCalendar::new();
    let cooks = test.cooks(&[september(21), september(23), september(28), october(2)]);
    assert_eq!(test.streak(&cooks, october(5)).weeks(), 2);
}

#[test]
fn a_missed_week_starts_the_count_again() {
    let test = TestCalendar::new();
    let cooks = test.cooks(&[september(14), september(15), september(21), september(28), october(2)]);
    assert_eq!(test.streak(&cooks, october(5)).weeks(), 1);
}

#[test]
fn the_week_in_progress_adds_only_once_its_goal_is_met() {
    let test = TestCalendar::new();
    let last_week = test.cooks(&[september(28), october(2)]);
    let one_day = [last_week.clone(), test.cooks(&[october(5)])].concat();
    let two_days = [last_week, test.cooks(&[october(5), october(6)])].concat();
    assert_eq!(test.streak(&one_day, october(6)).weeks(), 1);
    assert_eq!(test.streak(&two_days, october(6)).weeks(), 2);
}

#[test]
fn several_cooks_on_one_day_count_as_one_day() {
    let test = TestCalendar::new();
    let morning = made_at(&cook("knife", 1), &test.at(october(5), 9));
    let evening = made_at(&cook("eggs", 1), &test.at(october(5), 19));
    let streak = test.streak(&[with_weekly_goal(&morning, 2), with_weekly_goal(&evening, 2)], october(6));
    assert_eq!(streak.days_this_week, 1);
    assert!(!streak.is_this_week_met());
}

#[test]
fn a_past_week_is_judged_by_the_goal_recorded_that_week() {
    let test = TestCalendar::new();
    let cooks = test.cooks_with_goal(&[september(28), october(2)], 2);
    assert_eq!(test.with_goal(5).streak(&cooks, october(5)).weeks(), 1);
}

#[test]
fn a_weeks_goal_is_the_one_from_its_latest_cook_in_any_order() {
    let test = TestCalendar::new();
    let later_with_goal_two = test.cooks_with_goal(&[october(2)], 2);
    let earlier_with_goal_five = test.cooks_with_goal(&[september(28)], 5);
    let one_order = [later_with_goal_two.clone(), earlier_with_goal_five.clone()].concat();
    let other_order = [earlier_with_goal_five, later_with_goal_two].concat();
    assert_eq!(test.streak(&one_order, october(5)).weeks(), 1);
    assert_eq!(test.streak(&other_order, october(5)).weeks(), 1);
}

/// The server's own promise: two cooks at the same instant with different goals, in either order,
/// give the same week, so no answer depends on arrival order.
#[test]
fn cooks_at_the_same_instant_judge_their_week_the_same_in_any_order() {
    let test = TestCalendar::new();
    let goal_two = test.cooks_with_goal(&[october(2)], 2);
    let goal_five = test.cooks_with_goal(&[october(2)], 5);
    let monday = test.cooks_with_goal(&[september(28)], 2);
    let one_order = [monday.clone(), goal_two.clone(), goal_five.clone()].concat();
    let other_order = [monday, goal_five, goal_two].concat();
    assert_eq!(test.streak(&one_order, october(5)), test.streak(&other_order, october(5)));
}

/// The server's own promise: a goal outside the range the cook may pick is held inside it.
#[test]
fn a_goal_outside_the_range_is_held_inside_it() {
    let test = TestCalendar::new();
    assert_eq!(test.with_goal(0).streak(&[], october(5)).goal, 1);
    assert_eq!(test.with_goal(9).streak(&[], october(5)).goal, 7);
    let one_day_with_goal_nine = test.cooks_with_goal(&[september(28)], 9);
    let six_more = [september(29), september(30), october(1), october(2), october(3), october(4)];
    let seven_days = [one_day_with_goal_nine, test.cooks_with_goal(&six_more, 9)].concat();
    assert_eq!(test.streak(&seven_days, october(5)).weeks_before_this_week, 1);
}
