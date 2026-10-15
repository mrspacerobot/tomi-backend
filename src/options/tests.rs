use super::*;
use crate::library::tests::shipped_files;
use crate::special::links::tests::fixture_graph;
use serde_json::{json, Value};

fn shipped_options(language: &str) -> Value {
    let graph = fixture_graph();
    let mut library = Library::parse(&shipped_files()).unwrap().0;
    library.agree_with(&graph).unwrap();
    let language = Language::from_accept_language(Some(language));
    serde_json::to_value(options(&library, &graph.foods, &language)).unwrap()
}

#[test]
fn food_lists_come_with_their_words() {
    let options = shipped_options("en");
    assert_eq!(options["diets"][0], json!({"id": "vegetarian", "name": "Vegetarian", "summary": "No meat or fish"}));
    assert_eq!(options["avoidItems"][0], json!({"id": "nuts", "name": "Nuts", "kind": "allergen"}));
    assert_eq!(options["cuisines"][0], json!({"id": "italian", "name": "Italian"}));
}

#[test]
fn kitchen_and_answer_lists_come_with_their_words() {
    let options = shipped_options("en");
    assert_eq!(options["equipment"][0]["assumed"], true);
    assert_eq!(options["units"][1]["name"], "Cups");
    assert_eq!(options["experience"].as_array().unwrap().len(), 3);
    assert_eq!(options["ingredients"].as_array().unwrap().len(), fixture_graph().foods.listed().len());
}

#[test]
fn courses_hold_their_skills_in_menu_order() {
    let options = shipped_options("en");
    let first_course = &options["courses"][0];
    assert_eq!(first_course["id"], "foundations");
    assert_eq!(first_course["skills"][0]["id"], "knife-cuts");
    assert_eq!(first_course["skills"][0]["levels"][0], json!({"number": 1, "name": "Mirepoix"}));
}

#[test]
fn a_language_without_words_falls_back_to_english() {
    assert_eq!(shipped_options("de")["diets"][0]["name"], "Vegetarian");
}

fn avoid_item(options: &Value, id: &str) -> Value {
    let found = options["avoidItems"].as_array().unwrap().iter().find(|item| item["id"] == id);
    found.unwrap().clone()
}

#[test]
fn avoid_items_say_whether_they_are_allergens_or_choices() {
    let options = shipped_options("en");
    assert_eq!(avoid_item(&options, "dairy")["kind"], "allergen");
    assert_eq!(avoid_item(&options, "pork")["kind"], "choice");
    assert_eq!(avoid_item(&options, "alcohol")["kind"], "choice");
}

#[test]
fn the_allergens_the_graph_adds_are_listed_after_the_written_ones() {
    let options = shipped_options("en");
    let ids: Vec<&str> =
        options["avoidItems"].as_array().unwrap().iter().map(|item| item["id"].as_str().unwrap()).collect();
    assert_eq!(&ids[..3], ["nuts", "dairy", "gluten"]);
    assert_eq!(&ids[ids.len() - 5..], ["buckwheat", "celery", "lupin", "mustard", "sulphites"]);
    assert_eq!(avoid_item(&options, "celery"), json!({"id": "celery", "name": "Celery", "kind": "allergen"}));
}

#[test]
fn nuts_can_also_be_chosen_as_peanuts_or_tree_nuts_alone() {
    let options = shipped_options("en");
    assert_eq!(avoid_item(&options, "peanuts")["name"], "Peanuts");
    assert_eq!(avoid_item(&options, "tree-nuts")["name"], "Tree nuts");
}
