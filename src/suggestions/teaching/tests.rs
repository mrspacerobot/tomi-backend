//! Adaptation compatibility is independent of ingredient safety and fails closed.

use super::*;
use crate::graph::test_helpers::fixture_graph;
use crate::kernel::ids::SavedId;
use crate::library::{files, Library};
use crate::profile::Profile;
use crate::progress::OpenLevels;
use crate::suggestions::{candidates, Tonight};
use std::collections::BTreeSet;
use std::path::Path;

fn example() -> (Candidate, Graph) {
    let graph = fixture_graph();
    let library = Library::parse(&files::read(Path::new("library")).unwrap()).unwrap();
    let skill = SavedId::parse("eggs").unwrap();
    let open = OpenLevels::from_levels([(skill, 1)]);
    let profile = Profile::default();
    let tonight = Tonight { profile: &profile, available: BTreeSet::new(), confirmed: BTreeSet::new(), open: &open };
    let candidate = candidates(&library, &graph, &tonight).remove(0);
    (candidate, graph)
}

#[test]
fn teaching_keeps_schema_one() {
    let (candidate, graph) = example();
    let teaching = for_candidate(&candidate, &graph, &Language::english()).unwrap();
    assert_eq!(teaching.schema_version, 1);
}

#[test]
fn an_omitted_part_requires_reviewed_teaching_compatibility() {
    let (mut candidate, graph) = example();
    candidate.recipe.left_out.push(crate::recipe::OmittedPart { part: "garnish".into(), foods: vec![] });
    assert!(for_candidate(&candidate, &graph, &Language::english()).is_none());
}

#[test]
fn production_requires_a_quiz_and_a_draft_does_not() {
    assert!(!quiz_requirement_met(Channel::Production, false));
    assert!(quiz_requirement_met(Channel::Production, true));
    assert!(quiz_requirement_met(Channel::Draft, false));
    assert!(quiz_requirement_met(Channel::Draft, true));
}
