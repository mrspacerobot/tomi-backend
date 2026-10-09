use super::*;
use crate::domain::library::tests::shipped_files;
use serde_json::{json, Value};

fn shipped_options(language: &str) -> Value {
    let library = Library::parse(&shipped_files()).unwrap().0;
    let language = Language::from_accept_language(Some(language));
    serde_json::to_value(options(&library, &language)).unwrap()
}

#[test]
fn food_lists_come_with_their_words() {
    let options = shipped_options("en");
    assert_eq!(options["diets"][0], json!({"id": "vegetarian", "name": "Vegetarian", "summary": "No meat or fish"}));
    assert_eq!(options["avoidItems"][0], json!({"id": "nuts", "name": "Nuts"}));
    assert_eq!(options["cuisines"][0], json!({"id": "italian", "name": "Italian"}));
}

#[test]
fn kitchen_and_answer_lists_come_with_their_words() {
    let options = shipped_options("en");
    assert_eq!(options["equipment"][0]["assumed"], true);
    assert_eq!(options["units"][1]["name"], "Cups");
    assert_eq!(options["experience"].as_array().unwrap().len(), 3);
    assert_eq!(options["ingredients"].as_array().unwrap().len(), 23);
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
