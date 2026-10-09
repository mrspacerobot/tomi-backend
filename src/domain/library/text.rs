//! Words that travel with the content, in every language they exist in.

use serde::{Deserialize, Deserializer, Serialize};
use std::collections::BTreeMap;

/// Text by language code. English is required, so every reader has words.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(transparent)]
pub struct LocalizedText(BTreeMap<String, String>);

impl LocalizedText {
    pub fn english(&self) -> &str {
        match self.0.get("en") {
            Some(text) => text,
            None => "",
        }
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
