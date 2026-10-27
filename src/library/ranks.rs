//! The names of the rank ladder, in every language they exist in.
//!
//! The server keeps `library/ranks.json` (see `library/SOURCE.md`): the progress snapshot sends each rank's
//! permanent id with its name in the cook's language, so the app holds no rank words of its own.

use super::catalog::check_schema;
use super::LocalizedText;
use crate::kernel::language::Language;
use crate::progress::RankTitle;
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct RanksFile {
    schema_version: i64,
    ranks: Vec<RankEntry>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RankEntry {
    id: String,
    name: LocalizedText,
}

/// A name for every rank of the ladder, lowest first, in `RankTitle` order.
pub struct RankNames(Vec<LocalizedText>);

impl RankNames {
    /// The names, or why the file is refused: it must name every rank of the ladder, once, in ladder order.
    pub fn parse(text: &str) -> Result<Self, String> {
        let file: RanksFile = serde_json::from_str(text).map_err(|error| error.to_string())?;
        check_schema(file.schema_version)?;
        let listed: Vec<&str> = file.ranks.iter().map(|rank| rank.id.as_str()).collect();
        let ladder: Vec<&str> = RankTitle::ALL.iter().map(|title| title.id()).collect();
        if listed != ladder {
            return Err(format!("ranks must be {ladder:?} in this order, not {listed:?}"));
        }
        Ok(RankNames(file.ranks.into_iter().map(|rank| rank.name).collect()))
    }

    /// The rank's name in this language, or in English when it has none there.
    pub fn name(&self, title: RankTitle, language: &Language) -> &str {
        match self.0.get(title.position()) {
            Some(name) => name.text(language.as_str()),
            None => "",
        }
    }
}
