//! What the cook's cooks are worth: XP from the cook entries, and the rank ladder it climbs.

mod rank;
mod xp;

#[cfg(test)]
mod tests;

pub use rank::{Rank, RankTitle};
pub use xp::{earned_xp, xp_for};
