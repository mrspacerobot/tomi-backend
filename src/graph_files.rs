//! Starts the graph: reads its two files from disk and loads them against the pin in `graph.lock`.

use crate::config::GraphConfig;
use crate::domain::graph::{Graph, GraphFiles, GraphPin, GraphPolicy};
use std::fs;
use std::io;
use std::path::Path;

const MANIFEST_FILE: &str = "manifest.json";
const ARTIFACT_FILE: &str = "graph-artifact.json";

pub fn read(folder: &Path) -> io::Result<GraphFiles> {
    Ok(GraphFiles {
        manifest: fs::read_to_string(folder.join(MANIFEST_FILE))?,
        artifact: fs::read_to_string(folder.join(ARTIFACT_FILE))?,
    })
}

/// The pinned graph, none when `lock` pins none, or an error that stops the
/// start: a pinned graph that cannot be loaded must never be replaced by a guess.
pub fn start(lock: &str, config: &GraphConfig) -> Result<Option<Graph>, String> {
    let pin = GraphPin::parse(lock).map_err(|error| error.to_string())?;
    let Some(pin) = pin else {
        return Ok(None);
    };
    let Some(folder) = &config.dir else {
        return Err("graph.lock pins a graph but GRAPH_DIR is not set".to_string());
    };
    let files = read(Path::new(folder)).map_err(|error| format!("graph in {folder}: {error}"))?;
    let policy = GraphPolicy { allow_draft: config.allow_draft };
    let graph = Graph::load(&files, &pin, &policy).map_err(|error| format!("graph: {error}"))?;
    log_loaded(&graph);
    Ok(Some(graph))
}

#[expect(clippy::cognitive_complexity, reason = "tracing's macro expands into branches no reader sees")]
fn log_loaded(graph: &Graph) {
    tracing::info!(version = %graph.version, channel = graph.channel.as_str(), things = graph.len(), "graph loaded");
}
