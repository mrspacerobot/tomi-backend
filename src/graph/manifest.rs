//! The manifest and the artifact as they are written by the graph repository's `./release`.

use super::{GraphError, Serving};
use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;

/// Only the schema version, read first so that an unknown layout is
/// refused before anything else in the file is looked at.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Header {
    pub artifact_schema_version: u64,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Manifest {
    pub graph_version: String,
    pub source_commit: String,
    pub channel: String,
    pub content_sha256: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Body {
    pub artifact_schema_version: u64,
    pub channel: String,
    pub things: HashMap<String, Value>,
    pub serving: Serving,
}

/// Reads `text` as JSON. The error never repeats the text.
pub fn read<T: DeserializeOwned>(text: &str) -> Result<T, GraphError> {
    serde_json::from_str(text).map_err(|error| GraphError::Unreadable(error.to_string()))
}
