//! A pick is checked against the quiz the pinned graph teaches for its recipe, and only its first answer scores.

use super::pick::{check, is_scored, FirstAnswer, Pick, PickError};
use crate::graph::test_helpers::{change_thing, fixture_graph};
use uuid::Uuid;

const RECIPE: &str = "recipe/soft-scrambled-eggs";
const QUESTION: &str = "technique/scramble/quiz/purpose";

fn pick(question_id: &str, choice: usize) -> Pick {
    let (recipe_id, graph_version, question_id) = (RECIPE.into(), "fixture".into(), question_id.into());
    Pick { entry_id: Uuid::nil(), recipe_id, graph_version, question_id, choice }
}

#[test]
fn a_right_pick_carries_the_explanation_and_the_whole_quiz() {
    let checked = check(&pick(QUESTION, 0), &fixture_graph()).unwrap();
    assert!(checked.correct);
    assert!(checked.explanation.unwrap().contains("creamy curds"));
    let ids = ["purpose", "method", "observation", "correction"].map(|id| format!("technique/scramble/quiz/{id}"));
    assert_eq!(checked.quiz_question_ids, ids.to_vec());
}

#[test]
fn a_wrong_pick_carries_no_explanation() {
    let checked = check(&pick(QUESTION, 2), &fixture_graph()).unwrap();
    assert!(!checked.correct);
    assert!(checked.explanation.is_none());
}

#[test]
fn a_pick_outside_the_choices_or_the_recipes_quiz_is_refused() {
    let graph = fixture_graph();
    assert_eq!(check(&pick(QUESTION, 3), &graph).err(), Some(PickError::ChoiceOutOfRange));
    let other_recipe = pick("technique/roll-an-omelette/quiz/purpose", 1);
    assert_eq!(check(&other_recipe, &graph).err(), Some(PickError::QuestionNotInQuiz));
    let unknown = pick("technique/scramble/quiz/unknown", 0);
    assert_eq!(check(&unknown, &graph).err(), Some(PickError::QuestionNotInQuiz));
    assert_eq!(check(&pick("scramble/purpose", 0), &graph).err(), Some(PickError::QuestionNotInQuiz));
}

#[test]
fn a_pick_against_another_graph_is_refused_first() {
    let mut stale = pick("technique/scramble/quiz/unknown", 9);
    stale.graph_version = "2026.01.01".into();
    assert_eq!(check(&stale, &fixture_graph()).err(), Some(PickError::GraphVersionChanged));
}

#[test]
fn a_malformed_quiz_is_not_taught_and_so_not_answered() {
    let mut graph = fixture_graph();
    change_thing(&mut graph, "technique/scramble", |technique| technique["quiz"][1]["answer"] = 7.into());
    assert_eq!(check(&pick(QUESTION, 0), &graph).err(), Some(PickError::QuestionNotInQuiz));
}

#[test]
fn only_the_stored_first_pick_scores() {
    let graph = fixture_graph();
    let first = FirstAnswer { recipe_id: RECIPE.into(), graph_version: "fixture".into(), choice: 1 };
    let retry = pick(QUESTION, 1);
    assert!(is_scored(&first, &retry, &check(&retry, &graph).unwrap()));
    let later = pick(QUESTION, 0);
    assert!(!is_scored(&first, &later, &check(&later, &graph).unwrap()));
    let mut other_recipe = pick(QUESTION, 1);
    other_recipe.recipe_id = "recipe/spinach-and-cheese-omelette".into();
    assert!(!is_scored(&first, &other_recipe, &check(&retry, &graph).unwrap()));
}
