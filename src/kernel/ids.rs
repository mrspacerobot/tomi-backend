//! Ids as Tomi saves them, such as `nuts`, `vegetarian` or `olive-oil`.

use serde::{Deserialize, Deserializer, Serialize};
use std::fmt;

#[cfg(test)]
mod tests;

/// Longer than any id in Tomi's library, short enough to refuse junk.
const MAX_ID_LENGTH: usize = 64;

/// A diet, avoid item, cuisine, ingredient, equipment or skill id.
///
/// Parsing checks the shape once, at the edge, so the rest of the code can
/// trust it: lowercase letters and digits in parts joined by single hyphens.
/// Whether the library knows the id is a separate question: an id it does
/// not know is kept, never dropped, because a newer app may have saved it.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(transparent)]
pub struct SavedId(String);

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
#[error("`{0}` is not an id: use lowercase letters, digits and single hyphens")]
pub struct ParseIdError(String);

impl SavedId {
    pub fn parse(raw: &str) -> Result<Self, ParseIdError> {
        if raw.len() > MAX_ID_LENGTH || !is_slug(raw) {
            return Err(ParseIdError(raw.to_string()));
        }
        Ok(SavedId(raw.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for SavedId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for SavedId {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let raw = String::deserialize(deserializer)?;
        SavedId::parse(&raw).map_err(serde::de::Error::custom)
    }
}

fn is_slug(text: &str) -> bool {
    if text.is_empty() {
        return false;
    }
    text.split('-').all(is_slug_part)
}

fn is_slug_part(part: &str) -> bool {
    let allowed = |character: char| character.is_ascii_lowercase() || character.is_ascii_digit();
    !part.is_empty() && part.chars().all(allowed)
}
