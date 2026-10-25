//! The graph introduction is parsed once and emitted without rewriting its prose.

use super::tier::ContentTier;
use crate::graph::Channel;
use crate::kernel::language::Language;
use crate::library::LocalizedText;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct TechniqueSource {
    names: LocalizedText,
    tier: ContentTier,
    pub intro: Introduction,
}

#[derive(Debug, Serialize)]
pub struct Technique {
    id: String,
    name: String,
    tier: ContentTier,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Introduction {
    #[serde(rename(deserialize = "idea"))]
    concept: String,
    what: String,
    why: String,
    tier: ContentTier,
}

impl TechniqueSource {
    pub fn permitted(&self, channel: Channel, language: &Language) -> bool {
        self.tier.permitted(channel)
            && self.intro.permitted(channel)
            && !self.names.text(language.as_str()).trim().is_empty()
    }

    pub fn identity(&self, id: &str, language: &Language) -> Technique {
        Technique { id: id.to_string(), name: self.names.text(language.as_str()).to_string(), tier: self.tier }
    }
}

impl Introduction {
    fn permitted(&self, channel: Channel) -> bool {
        self.tier.permitted(channel) && nonblank(&[&self.concept, &self.what, &self.why])
    }
}

pub fn nonblank(texts: &[&str]) -> bool {
    texts.iter().all(|text| !text.trim().is_empty())
}
