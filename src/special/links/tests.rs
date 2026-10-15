use super::*;
use crate::config::GraphConfig;
use crate::graph::files;
use crate::kernel::ids::SavedId;
use crate::library::tests::shipped_files;
use crate::library::Library;

pub(crate) fn fixture_graph() -> Graph {
    let config = GraphConfig { dir: Some("tests/fixtures/graph".to_string()), allow_draft: true, required: true };
    files::start(include_str!("../../../tests/fixtures/graph/graph.lock"), &config).unwrap().unwrap()
}

fn lesson(name: &str) -> Lesson {
    let library = Library::parse(&shipped_files()).unwrap().0;
    library.lessons.get(&SavedId::parse(name).unwrap()).unwrap().clone()
}

#[test]
fn a_lesson_is_the_recipe_with_its_level_s_technique_and_its_exact_ingredients() {
    let graph = fixture_graph();
    let omelette = lesson("spinach-hard-cheese-omelette");
    assert_eq!(recipe_for(&omelette, &graph).unwrap(), "recipe/spinach-and-cheese-omelette");
    let pan_sauce = lesson("seared-chicken-lemon-butter-pan-sauce");
    let expected = "recipe/seared-chicken-thighs-with-lemon-butter-pan-sauce";
    assert_eq!(recipe_for(&pan_sauce, &graph).unwrap(), expected);
}

#[test]
fn a_lesson_whose_ingredients_differ_by_one_matches_no_recipe() {
    let mut omelette = lesson("spinach-hard-cheese-omelette");
    omelette.ingredients.pop();
    assert_eq!(recipe_for(&omelette, &fixture_graph()), Err(LinkProblem::NoRecipe));
}

#[test]
fn a_lesson_at_a_level_that_teaches_nothing_matches_no_technique() {
    let mut omelette = lesson("spinach-hard-cheese-omelette");
    omelette.level = 99;
    assert_eq!(recipe_for(&omelette, &fixture_graph()), Err(LinkProblem::NoTechnique));
}

#[test]
fn every_lesson_that_matches_no_recipe_is_reported_with_its_reason() {
    let library = Library::parse(&shipped_files()).unwrap().0;
    assert!(unlinked(&library, &fixture_graph()).is_empty());
    let mut graph = fixture_graph();
    graph.serving.recipes.clear();
    let problems = unlinked(&library, &graph);
    assert_eq!(problems.len(), 2);
    assert!(
        problems[0].contains("spinach-hard-cheese-omelette") || problems[1].contains("spinach-hard-cheese-omelette")
    );
}

#[test]
fn two_recipes_that_both_fit_are_never_picked_between() {
    let mut graph = fixture_graph();
    let twin = graph.serving.recipes["recipe/spinach-and-cheese-omelette"].clone();
    graph.serving.recipes.insert("recipe/another-omelette".to_string(), twin);
    let found = recipe_for(&lesson("spinach-hard-cheese-omelette"), &graph);
    assert!(matches!(found, Err(LinkProblem::Several(ids)) if ids.len() == 2));
}
