//! The cooking graph artifact: one pinned, checksummed bundle, loaded into memory at start.
//!
//! The graph repository builds it (`./release`), this repository pins one
//! version in `graph.lock`, and the image carries it. Everything here is
//! plain data in and out. A graph that is not exactly the pinned one is
//! refused, never served.

use serde_json::Value;
use std::collections::HashMap;

mod checksum;
pub mod files;
mod manifest;
mod pin;
#[cfg(test)]
mod tests;

pub use checksum::sha256_hex;
pub use pin::{GraphPin, PinError};

/// The only artifact layout this build can read.
pub const ARTIFACT_SCHEMA_VERSION: u64 = 1;

/// The graph's files as text. Reading them from disk is the caller's job.
pub struct GraphFiles {
    pub manifest: String,
    pub artifact: String,
}

/// What the owner allows this server to load.
pub struct GraphPolicy {
    pub allow_draft: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Channel {
    Draft,
    Production,
}

impl Channel {
    pub fn as_str(self) -> &'static str {
        match self {
            Channel::Draft => "draft",
            Channel::Production => "production",
        }
    }

    fn parse(text: &str) -> Result<Channel, GraphError> {
        match text {
            "draft" => Ok(Channel::Draft),
            "production" => Ok(Channel::Production),
            other => Err(GraphError::UnknownChannel(other.to_string())),
        }
    }
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum GraphError {
    #[error("artifact schema version {0} is not supported")]
    UnsupportedSchema(u64),
    #[error("the graph files cannot be read: {0}")]
    Unreadable(String),
    #[error("the channel `{0}` is not known")]
    UnknownChannel(String),
    #[error("a draft graph is refused unless GRAPH_ALLOW_DRAFT is true")]
    DraftRefused,
    #[error("the artifact does not match the checksum in its manifest")]
    ChecksumMismatch,
    #[error("the manifest and the artifact name different channels")]
    ChannelMismatch,
    #[error("the graph is not the one graph.lock pins")]
    NotPinned,
}

/// The graph in memory: every thing by its id.
#[derive(Debug)]
pub struct Graph {
    pub version: String,
    pub channel: Channel,
    pub source_commit: String,
    things: HashMap<String, Value>,
}

impl Graph {
    pub fn load(files: &GraphFiles, pin: &GraphPin, policy: &GraphPolicy) -> Result<Graph, GraphError> {
        let header: manifest::Header = manifest::read(&files.manifest)?;
        check_schema(header.artifact_schema_version)?;
        let manifest: manifest::Manifest = manifest::read(&files.manifest)?;
        let channel = Channel::parse(&manifest.channel)?;
        if channel == Channel::Draft && !policy.allow_draft {
            return Err(GraphError::DraftRefused);
        }
        verify(files, &manifest, pin)?;
        let things = read_things(&files.artifact, &manifest.channel)?;
        Ok(Graph { version: manifest.graph_version, channel, source_commit: manifest.source_commit, things })
    }

    pub fn thing(&self, id: &str) -> Option<&Value> {
        self.things.get(id)
    }

    pub fn len(&self) -> usize {
        self.things.len()
    }

    pub fn is_empty(&self) -> bool {
        self.things.is_empty()
    }
}

fn check_schema(found: u64) -> Result<(), GraphError> {
    if found == ARTIFACT_SCHEMA_VERSION {
        return Ok(());
    }
    Err(GraphError::UnsupportedSchema(found))
}

/// The artifact is what its manifest says, and the manifest is what the pin says.
fn verify(files: &GraphFiles, manifest: &manifest::Manifest, pin: &GraphPin) -> Result<(), GraphError> {
    if sha256_hex(&files.artifact) != manifest.content_sha256 {
        return Err(GraphError::ChecksumMismatch);
    }
    if manifest.content_sha256 != pin.sha256 || manifest.graph_version != pin.version {
        return Err(GraphError::NotPinned);
    }
    Ok(())
}

/// The artifact's own channel is covered by its checksum. The manifest's is not,
/// so the two must agree, or a manifest could promote a draft graph.
fn read_things(artifact: &str, manifest_channel: &str) -> Result<HashMap<String, Value>, GraphError> {
    let body: manifest::Body = manifest::read(artifact)?;
    check_schema(body.artifact_schema_version)?;
    if body.channel != manifest_channel {
        return Err(GraphError::ChannelMismatch);
    }
    Ok(body.things)
}
