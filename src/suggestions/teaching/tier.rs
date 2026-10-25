//! Graph content tiers preserve the graph vocabulary and obey its production review threshold.

use crate::graph::Channel;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ContentTier {
    Drafted,
    Sourced,
    Reviewed,
    Verified,
    CookTested,
}

impl ContentTier {
    pub fn permitted(self, channel: Channel) -> bool {
        match channel {
            Channel::Draft => true,
            Channel::Production => matches!(self, Self::Reviewed | Self::Verified | Self::CookTested),
        }
    }
}
