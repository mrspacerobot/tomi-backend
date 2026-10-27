//! Ports of TomiCore's `CookLogStreakTests` (the cook log feeds the streak rules, so one log is the
//! input for every rule) and `GameStateTests` (XP, rank and streak from the same log at the same moment).

use super::test_calendar::{TestCalendar, TEST_GOAL};
use crate::progress::fixtures::{cook, cooks, made_at, october, september, stopped_before_the_end, with_weekly_goal};
use crate::progress::game::{QUIZ_BONUS_XP, RANK_THRESHOLDS, XP_PER_FINISHED_COOK};
use crate::progress::{earned_xp, finished_cooks, CookEntry, Rank, RankTitle};

const XP_WITH_QUIZ: u32 = XP_PER_FINISHED_COOK + QUIZ_BONUS_XP;

/// A finished cook at 6 pm on this day in UTC, with the streak tests' goal.
fn on(day: jiff::civil::Date, skill: &str) -> CookEntry {
    with_weekly_goal(&made_at(&cook(skill, 1), &TestCalendar::new().at(day, 18)), TEST_GOAL)
}

#[test]
fn a_finished_entry_counts_for_the_streak_with_its_moment_zone_and_goal() {
    let entry = with_weekly_goal(&on(october(2), "knife"), 4);
    let counted = finished_cooks(std::slice::from_ref(&entry));
    assert_eq!(counted, [&entry]);
    assert_eq!(counted[0].cooked_at, TestCalendar::new().at(october(2), 18).timestamp());
    assert_eq!(counted[0].weekly_goal, 4);
    let test = TestCalendar::new();
    let three_more = test.cooks_with_goal(&[september(28), september(29), september(30)], 1);
    assert_eq!(test.streak(&[three_more, vec![entry]].concat(), october(5)).weeks(), 1);
}

#[test]
fn an_entry_stopped_early_does_not_count_for_the_streak() {
    let entry = stopped_before_the_end(&on(october(5), "knife"));
    assert_eq!(TestCalendar::new().streak(&[entry], october(5)).days_this_week, 0);
}

#[test]
fn the_same_entry_twice_counts_as_one_day_once() {
    let entry = on(october(5), "knife");
    let streak = TestCalendar::new().streak(&[entry.clone(), entry], october(6));
    assert_eq!(streak.days_this_week, 1);
}

#[test]
fn entries_the_catalog_does_not_know_still_count_for_the_streak() {
    let unknown = made_at(&cook("sous-vide", 9), &TestCalendar::new().at(october(5), 18));
    assert_eq!(TestCalendar::new().streak(&[unknown], october(6)).days_this_week, 1);
}

#[test]
fn the_streak_is_counted_from_the_log() {
    let last_week = vec![on(september(28), "knife"), on(october(2), "eggs")];
    let abandoned = stopped_before_the_end(&on(october(6), "knife"));
    let this_week = vec![on(october(5), "knife"), abandoned];
    let streak = TestCalendar::new().streak(&[last_week, this_week].concat(), october(6));
    assert_eq!(streak.weeks(), 1);
    assert_eq!(streak.days_this_week, 1);
}

#[test]
fn a_new_cook_starts_with_no_xp_as_a_novice_with_no_streak() {
    let streak = TestCalendar::new().streak(&[], october(6));
    assert_eq!(earned_xp(&[]), 0);
    assert_eq!(Rank::for_xp(earned_xp(&[])).title, RankTitle::Novice);
    assert_eq!((streak.weeks(), streak.days_this_week), (0, 0));
}

#[test]
fn xp_rank_and_streak_come_from_the_same_log() {
    let log = vec![on(september(28), "knife"), on(october(2), "eggs"), on(october(5), "knife")];
    let streak = TestCalendar::new().streak(&log, october(6));
    assert_eq!(earned_xp(&log), 3 * XP_WITH_QUIZ);
    assert_eq!(Rank::for_xp(earned_xp(&log)), Rank::for_xp(3 * XP_WITH_QUIZ));
    assert_eq!((streak.weeks(), streak.days_this_week), (1, 1));
}

#[test]
fn stopped_early_and_repeated_entries_change_nothing() {
    let test = TestCalendar::new();
    let entry = on(october(5), "knife");
    let abandoned = stopped_before_the_end(&on(october(6), "eggs"));
    let noisy = vec![entry.clone(), abandoned, entry.clone()];
    assert_eq!(test.streak(&noisy, october(6)), test.streak(std::slice::from_ref(&entry), october(6)));
    assert_eq!(earned_xp(&noisy), earned_xp(&[entry]));
}

#[test]
fn enough_cooks_raise_the_rank() {
    let commis_at = RANK_THRESHOLDS[1];
    let cooks_needed = (commis_at - 1) / XP_WITH_QUIZ + 1;
    let rank = Rank::for_xp(earned_xp(&cooks(cooks_needed, "knife", 1)));
    assert_eq!(rank.title, RankTitle::Commis);
    assert_eq!(rank.xp, earned_xp(&cooks(cooks_needed, "knife", 1)));
}

#[test]
fn the_streak_is_the_one_the_streak_rules_give_for_the_log_and_the_current_goal() {
    let test = TestCalendar::new();
    let log = vec![on(september(28), "knife"), on(october(5), "eggs")];
    let streak = test.with_goal(4).streak(&log, october(6));
    assert_eq!((streak.goal, streak.days_this_week, streak.weeks()), (4, 1, 0));
    assert_eq!(streak.days_left_in_week, 6);
}
