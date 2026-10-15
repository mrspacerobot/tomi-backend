//! The server's rule must give the answers the graph's reference gives, for every case the release ships.
//!
//! Needs a fetched graph (`scripts/fetch-graph.sh`), so it runs on request: `cargo test -- --ignored graph_vectors`.

#![expect(clippy::unwrap_used, reason = "a failed unwrap in a test is a failed test, not a crash")]

use serde::Deserialize;
use std::collections::BTreeSet;
use std::path::Path;
use tomi_api::graph::files;
use tomi_api::graph::serving::CookFacts;
use tomi_api::graph::{Graph, GraphPin, GraphPolicy};

#[derive(Deserialize)]
struct Vectors {
    cases: Vec<Case>,
}

#[derive(Deserialize)]
struct Case {
    avoid: BTreeSet<String>,
    offered: Vec<String>,
}

fn load() -> (Graph, Vectors) {
    let folder = Path::new("graph");
    let pin = GraphPin::parse(include_str!("../graph.lock")).unwrap().ok_or("graph.lock pins no graph").unwrap();
    let graph = Graph::load(&files::read(folder).unwrap(), &pin, &GraphPolicy { allow_draft: true }).unwrap();
    let vectors = std::fs::read_to_string(folder.join("serving-vectors.json")).unwrap();
    (graph, serde_json::from_str(&vectors).unwrap())
}

#[test]
#[ignore = "needs the pinned graph: run scripts/fetch-graph.sh first"]
fn graph_vectors_the_server_offers_exactly_what_the_reference_offers() {
    let (graph, vectors) = load();
    assert!(!vectors.cases.is_empty());
    for case in vectors.cases {
        let cook = CookFacts { avoided: case.avoid.clone(), at_hand: BTreeSet::new() };
        let offered: Vec<String> = graph
            .serving
            .recipes
            .keys()
            .filter(|recipe| graph.serving.plan(recipe, &cook).unwrap().is_some())
            .cloned()
            .collect();
        assert_eq!(offered, case.offered, "avoiding {:?}", case.avoid);
    }
}

#[test]
#[ignore = "needs the pinned graph: run scripts/fetch-graph.sh first"]
fn graph_vectors_the_shipped_library_agrees_with_the_graph() {
    let (graph, _) = load();
    let files = tomi_api::library::files::read(Path::new("library")).unwrap();
    let (library, left_out) = tomi_api::library::Library::parse(&files).unwrap();
    assert!(left_out.is_empty(), "{left_out:?}");
    assert_eq!(library.restrictions.check_against(&graph.serving.facts), Ok(()));
    let unlinked = tomi_api::special::links::unlinked(&library, &graph);
    assert!(unlinked.is_empty(), "{unlinked:?}");
}
