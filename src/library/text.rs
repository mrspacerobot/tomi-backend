//! Words that travel with the content, in every language they exist in.

use crate::kernel::ids::SavedId;
use serde::{Deserialize, Deserializer};
use std::collections::BTreeMap;

/// Text by language code. English is required, so every reader has words.
#[derive(Debug, Clone, PartialEq)]
pub struct LocalizedText(BTreeMap<String, String>);

impl LocalizedText {
    pub fn english(&self) -> &str {
        self.text("en")
    }

    /// The text in this language, or in English when it has none.
    pub fn text(&self, language: &str) -> &str {
        if let Some(text) = self.0.get(language) {
            return text;
        }
        match self.0.get("en") {
            Some(text) => text,
            None => "",
        }
    }

    /// Every text in every language, for matching what a cook typed.
    pub fn all(&self) -> impl Iterator<Item = &str> {
        self.0.values().map(String::as_str)
    }
}

impl<'de> Deserialize<'de> for LocalizedText {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let texts = BTreeMap::<String, String>::deserialize(deserializer)?;
        let has_english = texts.get("en").is_some_and(|text| !text.trim().is_empty());
        if !has_english {
            return Err(serde::de::Error::custom("English text is missing"));
        }
        Ok(LocalizedText(texts))
    }
}

/// Something the cook can pick, with its words.
#[derive(Debug, Clone)]
pub struct Choice {
    pub id: SavedId,
    pub name: LocalizedText,
    pub summary: Option<LocalizedText>,
}

impl Choice {
    pub fn named(id: &SavedId, name: &LocalizedText) -> Self {
        Choice { id: id.clone(), name: name.clone(), summary: None }
    }

    pub fn described(id: &SavedId, name: &LocalizedText, summary: &LocalizedText) -> Self {
        Choice { id: id.clone(), name: name.clone(), summary: Some(summary.clone()) }
    }
}
