//! Typed ids such as `fact/peanuts` or `equipment/wok`.
//!
//! An id is a kind and a lowercase English slug, the same shape the cooking
//! graph uses. The kind is checked so a pantry cannot hold a diet.

use serde::{Deserialize, Deserializer, Serialize};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(transparent)]
pub struct TypedId(String);

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum IdError {
    #[error("`{0}` is not an id of the form kind/slug")]
    Malformed(String),
    #[error("`{id}` is not of kind {expected}")]
    WrongKind { id: String, expected: &'static str },
}

impl TypedId {
    pub fn parse(raw: &str) -> Result<Self, IdError> {
        let malformed = || IdError::Malformed(raw.to_string());
        let (kind, slug) = raw.split_once('/').ok_or_else(malformed)?;
        if !is_word(kind) || !is_slug(slug) || raw.len() > 120 {
            return Err(malformed());
        }
        Ok(TypedId(raw.to_string()))
    }

    pub fn kind(&self) -> &str {
        self.0.split_once('/').map(|(kind, _)| kind).unwrap_or("")
    }

    pub fn expect_kind(self, expected: &'static str) -> Result<Self, IdError> {
        if self.kind() == expected {
            Ok(self)
        } else {
            Err(IdError::WrongKind {
                id: self.0,
                expected,
            })
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for TypedId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for TypedId {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let raw = String::deserialize(deserializer)?;
        TypedId::parse(&raw).map_err(serde::de::Error::custom)
    }
}

fn is_word(text: &str) -> bool {
    !text.is_empty() && text.chars().all(|c| c.is_ascii_lowercase() || c == '-')
}

/// Lowercase letters and digits in parts joined by single hyphens.
fn is_slug(text: &str) -> bool {
    !text.is_empty()
        && text.split('-').all(|part| {
            !part.is_empty()
                && part
                    .chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_graph_shaped_ids() {
        for raw in [
            "fact/peanuts",
            "fact/tree-nuts",
            "equipment/baking-sheet",
            "cuisine/french",
            "skill/knife-1",
        ] {
            assert!(TypedId::parse(raw).is_ok(), "{raw}");
        }
    }

    #[test]
    fn refuses_malformed_ids() {
        for raw in [
            "peanuts",
            "fact/",
            "/peanuts",
            "Fact/peanuts",
            "fact/Peanuts",
            "fact/tree--nuts",
            "fact/-nuts",
            "fact/a/b",
            "fact/pea nuts",
        ] {
            assert!(TypedId::parse(raw).is_err(), "{raw}");
        }
    }

    #[test]
    fn checks_the_kind() {
        let id = TypedId::parse("diet/vegan").unwrap();
        assert!(id.clone().expect_kind("diet").is_ok());
        assert_eq!(
            id.expect_kind("fact"),
            Err(IdError::WrongKind {
                id: "diet/vegan".into(),
                expected: "fact"
            })
        );
    }
}
