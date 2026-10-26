//! Port of TomiCore's `RankPacingTests`: the pacing promised to a regular cook, about 60 XP a cook,
//! three or four cooks a week.

use crate::progress::game::{QUIZ_BONUS_XP, RANK_THRESHOLDS, XP_PER_FINISHED_COOK, XP_PER_TIER_PAST_TOP};
use crate::progress::{Rank, RankTitle};

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
