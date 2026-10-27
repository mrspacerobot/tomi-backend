//! The cook's rank as the snapshot sends it: a permanent id, a name in the cook's language, and its bounds.

use super::SnapshotInputs;
use crate::progress::Rank;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RankAnswer {
    /// Permanent, the same in every language.
    pub id: &'static str,
    /// In the language of the request, English when the rank has no name in it.
    pub name: String,
    /// 1 for every named rank; past Toque it counts on.
    pub tier: u32,
    /// XP at which this rank began.
    pub starts_at: u32,
    /// XP at which the next rank begins.
    pub next_at: u32,
}

impl RankAnswer {
    pub fn for_xp(xp: u32, inputs: &SnapshotInputs) -> RankAnswer {
        let rank = Rank::for_xp(xp);
        RankAnswer {
            id: rank.title.id(),
            name: inputs.rank_names.name(rank.title, inputs.language).to_string(),
            tier: rank.tier,
            starts_at: rank.starts_at,
            next_at: rank.next_at,
        }
    }
}
