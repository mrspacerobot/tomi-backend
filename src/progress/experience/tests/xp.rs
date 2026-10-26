//! Ports of TomiCore's `CookLogXPTests` (XP is counted from the cook entries, with the numbers of
//! `game` and nothing stored beside them) and `RankPacingTests` (about 60 XP a cook, three or four
//! cooks a week).

use crate::progress::fixtures::{cook, cooks, stopped_before_the_end, with_the_quiz_wrong};
use crate::progress::game::{QUIZ_BONUS_XP, RANK_THRESHOLDS, XP_PER_FINISHED_COOK, XP_PER_TIER_PAST_TOP};
use crate::progress::{earned_xp, xp_for, CookEntry, Outcome, Rank, RankTitle};

const XP_WITH_QUIZ: u32 = XP_PER_FINISHED_COOK + QUIZ_BONUS_XP;

#[test]
fn an_empty_log_has_no_xp() {
    let log: Vec<CookEntry> = Vec::new();
    assert_eq!(earned_xp(&log), 0);
}

#[test]
fn a_finished_cook_with_the_quiz_right_earns_the_cook_and_the_quiz() {
    assert_eq!(earned_xp(&[cook("knife", 1)]), XP_WITH_QUIZ);
    assert_eq!(XP_WITH_QUIZ, 60);
}

#[test]
fn a_finished_cook_with_the_quiz_wrong_earns_the_cook_only() {
    let entry = with_the_quiz_wrong(&cook("knife", 1));
    assert_eq!(earned_xp(&[entry]), XP_PER_FINISHED_COOK);
    assert_eq!(XP_PER_FINISHED_COOK, 50);
}

#[test]
fn an_entry_stopped_early_earns_nothing_even_with_the_quiz_right() {
    let entry = stopped_before_the_end(&cook("knife", 1));
    assert_eq!(earned_xp(&[entry]), 0);
}

#[test]
fn the_same_entry_twice_after_a_merge_earns_once() {
    let entry = cook("knife", 1);
    let other = with_the_quiz_wrong(&cook("eggs", 1));
    let merged = [entry.clone(), other.clone(), other, entry];
    assert_eq!(earned_xp(&merged), XP_WITH_QUIZ + XP_PER_FINISHED_COOK);
}

#[test]
fn an_entry_the_catalog_does_not_know_still_earns_xp() {
    assert_eq!(earned_xp(&[cook("sous-vide", 9)]), XP_WITH_QUIZ);
}

#[test]
fn every_finished_cook_adds_up() {
    let log = [cooks(3, "knife", 1), cooks(2, "eggs", 2)].concat();
    assert_eq!(earned_xp(&log), 5 * XP_WITH_QUIZ);
}

#[test]
fn the_cook_that_reaches_a_threshold_crosses_into_the_next_rank() {
    let commis_at = RANK_THRESHOLDS[1];
    let cooks_still_below = (commis_at - 1) / XP_WITH_QUIZ;
    let below = cooks(cooks_still_below, "knife", 1);
    let across = [below.clone(), vec![cook("knife", 1)]].concat();
    assert_eq!(Rank::for_xp(earned_xp(&below)).title, RankTitle::Novice);
    assert_eq!(Rank::for_xp(earned_xp(&across)).title, RankTitle::Commis);
}

#[test]
fn the_way_a_cook_ended_decides_what_it_earns() {
    assert_eq!(xp_for(Outcome::StoppedEarly), 0);
    assert_eq!(xp_for(Outcome::Finished { quiz_passed: false }), XP_PER_FINISHED_COOK);
    assert_eq!(xp_for(Outcome::Finished { quiz_passed: true }), XP_PER_FINISHED_COOK + QUIZ_BONUS_XP);
}

/// A cook with the quiz right, the usual case.
const TYPICAL_COOK: u32 = XP_PER_FINISHED_COOK + QUIZ_BONUS_XP;
const COOKS_PER_WEEK: f64 = 3.5;

/// The cook on which a regular cook reaches each named rank, from the pacing table in tomi#63.
const PROMOTION_COOKS: [(RankTitle, u32); 8] = [
    (RankTitle::Novice, 0),
    (RankTitle::Commis, 3),
    (RankTitle::Cook, 7),
    (RankTitle::Saucier, 15),
    (RankTitle::Sous, 30),
    (RankTitle::Chef, 55),
    (RankTitle::Patron, 100),
    (RankTitle::Toque, 184),
];

fn weeks_to_earn(xp: u32) -> f64 {
    f64::from(xp) / f64::from(TYPICAL_COOK) / COOKS_PER_WEEK
}

#[test]
fn the_pacing_table_covers_every_named_rank() {
    let titles: Vec<RankTitle> = PROMOTION_COOKS.iter().map(|(title, _)| *title).collect();
    assert_eq!(titles, RankTitle::ALL);
}

#[test]
fn a_regular_cook_is_promoted_on_the_planned_cook() {
    for (title, cook) in PROMOTION_COOKS.into_iter().skip(1) {
        assert_eq!(Rank::for_xp(cook * TYPICAL_COOK).title, title);
        assert!(Rank::for_xp((cook - 1) * TYPICAL_COOK).title < title, "{title:?}");
    }
}

#[test]
fn even_without_the_quiz_the_first_promotion_comes_in_week_one() {
    assert!(Rank::for_xp(4 * XP_PER_FINISHED_COOK).title >= RankTitle::Commis);
}

#[test]
fn toque_takes_about_a_year() {
    assert!((45.0..=60.0).contains(&weeks_to_earn(RANK_THRESHOLDS[7])));
}

#[test]
fn each_toque_tier_past_the_top_takes_about_a_season() {
    assert!((11.0..=17.0).contains(&weeks_to_earn(XP_PER_TIER_PAST_TOP)));
}
