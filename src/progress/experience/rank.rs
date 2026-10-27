//! Where a cook stands on the rank ladder for their total XP, and how far it is to the next rank.

use crate::progress::game::{RANK_THRESHOLDS, XP_PER_TIER_PAST_TOP};

/// XP at which Toque, the top named rank, begins.
const TOP_THRESHOLD: u32 = RANK_THRESHOLDS[RANK_THRESHOLDS.len() - 1];

/// The named ranks of the kitchen brigade, lowest first, in `game::RANK_THRESHOLDS` order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RankTitle {
    Novice,
    Commis,
    Cook,
    Saucier,
    Sous,
    Chef,
    Patron,
    Toque,
}

impl RankTitle {
    /// Every named rank, lowest first.
    pub const ALL: [RankTitle; 8] = [
        RankTitle::Novice,
        RankTitle::Commis,
        RankTitle::Cook,
        RankTitle::Saucier,
        RankTitle::Sous,
        RankTitle::Chef,
        RankTitle::Patron,
        RankTitle::Toque,
    ];

    /// The rank's permanent id, as the snapshot sends it and `library/ranks.json` names it.
    pub fn id(self) -> &'static str {
        match self {
            RankTitle::Novice => "novice",
            RankTitle::Commis => "commis",
            RankTitle::Cook => "cook",
            RankTitle::Saucier => "saucier",
            RankTitle::Sous => "sous",
            RankTitle::Chef => "chef",
            RankTitle::Patron => "patron",
            RankTitle::Toque => "toque",
        }
    }

    /// Where the rank stands on the ladder, 0 for the lowest.
    pub fn position(self) -> usize {
        self as usize
    }
}

/// A cook's rank for a total of XP. Computed from the cook log on demand, never stored.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rank {
    pub title: RankTitle,
    /// 1 for every named rank. Past Toque it counts on: Toque II is tier 2.
    pub tier: u32,
    pub xp: u32,
    /// XP at which this rank began.
    pub starts_at: u32,
    /// XP at which the next rank begins. There is always a next rank.
    pub next_at: u32,
}

impl Rank {
    /// The rank for a total of XP, following the named thresholds and then Toque tiers without end.
    pub fn for_xp(xp: u32) -> Rank {
        let named = RankTitle::ALL.into_iter().zip(RANK_THRESHOLDS.windows(2));
        for (title, bounds) in named {
            let &[starts_at, next_at] = bounds else {
                continue;
            };
            if xp < next_at {
                return Rank { title, tier: 1, xp, starts_at, next_at };
            }
        }
        Rank::tier_past_top(xp)
    }

    /// Toque, in the tier this much XP reaches: Toque II begins one step past the top threshold.
    fn tier_past_top(xp: u32) -> Rank {
        let tiers_past = (xp - TOP_THRESHOLD) / XP_PER_TIER_PAST_TOP;
        let starts_at = TOP_THRESHOLD + tiers_past * XP_PER_TIER_PAST_TOP;
        let next_at = starts_at.saturating_add(XP_PER_TIER_PAST_TOP);
        Rank { title: RankTitle::Toque, tier: tiers_past + 1, xp, starts_at, next_at }
    }

    /// The share of the way from this rank to the next, from 0 up to but never reaching 1.
    pub fn progress(&self) -> f64 {
        let earned = f64::from(self.xp - self.starts_at);
        let needed = f64::from(self.next_at - self.starts_at);
        earned / needed
    }
}
