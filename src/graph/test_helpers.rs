//! Shared graph fixtures for unit tests.

use super::Graph;
use crate::config::GraphConfig;
use crate::graph::files;

pub(crate) fn fixture_graph() -> Graph {
    let config = GraphConfig { dir: Some("tests/fixtures/graph".to_string()), allow_draft: true, required: true };
    files::start(include_str!("../../tests/fixtures/graph/graph.lock"), &config).unwrap().unwrap()
}
