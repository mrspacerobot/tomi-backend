//! Port of TomiCore's `RankLadderTests`: every XP belongs to exactly one rank, and the steps grow.

use crate::progress::game::{RANK_THRESHOLDS, XP_PER_TIER_PAST_TOP};
use crate::progress::{Rank, RankTitle};

const TOP: u32 = RANK_THRESHOLDS[7];

/// Every XP a cook can hold, from zero to five Toque tiers past the top.
fn every_xp() -> std::ops::RangeInclusive<u32> {
    0..=(TOP + 5 * XP_PER_TIER_PAST_TOP)
}

fn place(rank: &Rank) -> (RankTitle, u32) {
    (rank.title, rank.tier)
}

#[test]
fn every_named_rank_has_a_threshold_starting_at_zero() {
    assert_eq!(RANK_THRESHOLDS.len(), RankTitle::ALL.len());
    assert_eq!(RANK_THRESHOLDS[0], 0);
}

#[test]
fn each_named_step_costs_roughly_twice_the_one_before() {
    let steps: Vec<f64> = RANK_THRESHOLDS.windows(2).map(|pair| f64::from(pair[1] - pair[0])).collect();
    for pair in steps.windows(2) {
        let ratio = pair[1] / pair[0];
        assert!((1.5..=2.5).contains(&ratio), "{} then {}", pair[0], pair[1]);
    }
}

#[test]
fn every_xp_falls_inside_its_rank() {
    for xp in every_xp() {
        let rank = Rank::for_xp(xp);
        assert_eq!(rank.xp, xp);
        assert!(rank.starts_at <= xp && xp < rank.next_at, "{xp}");
        assert!((0.0..1.0).contains(&rank.progress()), "{xp}");
    }
}

#[test]
fn each_rank_ends_exactly_where_a_higher_one_begins() {
    for xp in every_xp() {
        let rank = Rank::for_xp(xp);
        let next = Rank::for_xp(rank.next_at);
        assert_eq!(next.starts_at, rank.next_at, "{xp}");
        assert!(place(&next) > place(&rank), "{xp}");
    }
}

#[test]
fn more_xp_never_means_a_lower_rank() {
    let ranks: Vec<Rank> = every_xp().map(Rank::for_xp).collect();
    assert!(ranks.windows(2).all(|pair| place(&pair[1]) >= place(&pair[0])));
}

#[test]
fn progress_is_the_share_of_the_way_to_the_next_rank() {
    let commis = Rank::for_xp(RANK_THRESHOLDS[1]);
    let halfway = commis.starts_at + (commis.next_at - commis.starts_at) / 2;
    assert_eq!(commis.progress(), 0.0);
    assert_eq!(Rank::for_xp(halfway).progress(), 0.5);
    assert_eq!(Rank::for_xp(TOP + XP_PER_TIER_PAST_TOP / 4).progress(), 0.25);
}
