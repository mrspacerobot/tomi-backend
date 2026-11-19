//! The server's rule must give the answers the graph's reference gives, for every case the release ships.
//!
//! Needs a fetched graph (`scripts/fetch-graph.sh`), so it runs on request: `cargo test -- --ignored graph_vectors`.

#![expect(clippy::unwrap_used, reason = "a failed unwrap in a test is a failed test, not a crash")]

use serde::Deserialize;
use std::collections::BTreeSet;
use std::path::Path;
use tomi_api::graph::files;
use tomi_api::graph::serving::{CookFacts, PlannedSwap, RecipePlan};
use tomi_api::graph::{Graph, GraphPin, GraphPolicy};
use tomi_api::recipe::{adapt, Basis};

#[derive(Deserialize)]
struct Vectors {
    #[serde(default)]
    cases: Vec<Case>,
    #[serde(default)]
    step_text_cases: Vec<StepTextCase>,
}

#[derive(Deserialize)]
struct Case {
    avoid: BTreeSet<String>,
    offered: Vec<String>,
}

fn load() -> (Graph, Vectors) {
    load_from(Path::new("graph"), include_str!("../graph.lock"))
}

fn load_from(folder: &Path, lock: &str) -> (Graph, Vectors) {
    let pin = GraphPin::parse(lock).unwrap().ok_or("graph.lock pins no graph").unwrap();
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
    let mut library = tomi_api::library::Library::parse(&files).unwrap();
    assert_eq!(library.agree_with(&graph), Ok(()));
    assert_eq!(
        released_items(),
        current_items(&library),
        "the avoid items differ from library/avoid-items.released.json"
    );
}

type Items = std::collections::BTreeMap<String, BTreeSet<String>>;

fn released_items() -> Items {
    serde_json::from_str(&std::fs::read_to_string("library/avoid-items.released.json").unwrap()).unwrap()
}

fn current_items(library: &tomi_api::library::Library) -> Items {
    let items = library.restrictions.item_facts();
    items.iter().map(|(id, facts)| (id.to_string(), facts.clone())).collect()
}

#[derive(Deserialize)]
struct StepTextCase {
    recipe: String,
    left_out: Vec<String>,
    swaps: Vec<CaseSwap>,
    steps: Vec<CaseStep>,
}

#[derive(Deserialize)]
struct CaseSwap {
    part: String,
    thing: String,
    instead: String,
}

#[derive(Deserialize)]
struct CaseStep {
    text: String,
}

/// Every case: the recipe as written, with each optional part left out, and with each swap made.
fn check_step_text_cases(graph: &Graph, cases: &[StepTextCase]) {
    assert!(!cases.is_empty());
    for case in cases {
        let written = written_steps(graph, case);
        let expected: Vec<&str> = case.steps.iter().map(|step| step.text.as_str()).collect();
        assert_eq!(
            written.iter().map(String::as_str).collect::<Vec<_>>(),
            expected,
            "{} with {} swaps, leaving out {:?}",
            case.recipe,
            case.swaps.len(),
            case.left_out
        );
    }
}

fn written_steps(graph: &Graph, case: &StepTextCase) -> Vec<String> {
    let swaps = case.swaps.iter().map(|swap| PlannedSwap {
        part: swap.part.clone(),
        original: swap.thing.clone(),
        instead: swap.instead.clone(),
        ratio: None,
        caveat: None,
    });
    let plan = RecipePlan { swaps: swaps.collect(), left_out: case.left_out.clone() };
    let nothing_avoided = BTreeSet::new();
    let made = adapt(&case.recipe, &plan, &Basis { graph, avoided: &nothing_avoided }).unwrap();
    made.steps.into_iter().map(|step| step.text).collect()
}

#[test]
fn graph_vectors_the_sample_recipes_are_written_as_the_reference_writes_them() {
    let (graph, vectors) = load_from(Path::new("tests/fixtures/graph"), include_str!("fixtures/graph/graph.lock"));
    check_step_text_cases(&graph, &vectors.step_text_cases);
}

#[test]
#[ignore = "needs the pinned graph: run scripts/fetch-graph.sh first"]
fn graph_vectors_every_recipe_is_written_as_the_reference_writes_it() {
    let (graph, vectors) = load();
    check_step_text_cases(&graph, &vectors.step_text_cases);
}

#[test]
#[ignore = "needs the pinned graph: run scripts/fetch-graph.sh first"]
fn graph_vectors_the_dish_photo_manifest_references_the_pinned_graph() {
    use tomi_api::dish_photos::{Manifest, SHIPPED};
    let (graph, _) = load();
    Manifest::parse(SHIPPED).unwrap().validate(&graph).unwrap();
}
