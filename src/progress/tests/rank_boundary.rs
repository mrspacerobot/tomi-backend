//! Port of TomiCore's `RankBoundaryTests`: exactly where each rank begins, so an off-by-one at a
//! threshold cannot hide.

use crate::progress::game::{RANK_THRESHOLDS, XP_PER_TIER_PAST_TOP};
use crate::progress::{Rank, RankTitle};

const TOP: u32 = RANK_THRESHOLDS[7];
const STEP: u32 = XP_PER_TIER_PAST_TOP;

fn named_ranks() -> impl Iterator<Item = (RankTitle, u32)> {
    RankTitle::ALL.into_iter().zip(RANK_THRESHOLDS)
}

/// The app's case `forXP(-10)` cannot be written here: XP is unsigned, so a negative total
/// cannot reach the ladder at all.
#[test]
fn a_new_cook_is_a_novice_at_the_foot_of_the_ladder() {
    let novice = Rank { title: RankTitle::Novice, tier: 1, xp: 0, starts_at: 0, next_at: RANK_THRESHOLDS[1] };
    assert_eq!(Rank::for_xp(0), novice);
}

#[test]
fn a_named_rank_begins_at_its_threshold() {
    for (title, threshold) in named_ranks() {
        let rank = Rank::for_xp(threshold);
        assert_eq!(rank.title, title);
        assert_eq!(rank.tier, 1);
        assert_eq!(rank.starts_at, threshold);
    }
}

#[test]
fn one_xp_short_of_a_rank_is_still_the_one_below() {
    for (title, threshold) in named_ranks().skip(1) {
        let below = Rank::for_xp(threshold - 1);
        assert!(below.title < title, "{threshold}");
        assert_eq!(below.next_at, threshold);
    }
}

#[test]
fn toque_is_the_top_named_rank_and_leads_to_toque_ii() {
    let toque = Rank { title: RankTitle::Toque, tier: 1, xp: TOP, starts_at: TOP, next_at: TOP + STEP };
    assert_eq!(Rank::for_xp(TOP), toque);
}

#[test]
fn each_toque_tier_begins_one_step_after_the_last() {
    for tier in 2..=6 {
        let start = TOP + (tier - 1) * STEP;
        let rank = Rank { title: RankTitle::Toque, tier, xp: start, starts_at: start, next_at: start + STEP };
        assert_eq!(Rank::for_xp(start), rank);
        assert_eq!(Rank::for_xp(start - 1).tier, tier - 1);
    }
}

#[test]
fn far_past_toque_the_tiers_keep_counting() {
    let rank = Rank::for_xp(TOP + 99 * STEP + 1);
    assert_eq!(rank.title, RankTitle::Toque);
    assert_eq!(rank.tier, 100);
    assert_eq!(rank.next_at, TOP + 100 * STEP);
}
