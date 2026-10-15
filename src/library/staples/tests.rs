use super::*;
use crate::graph::Foods;
use serde_json::{json, Value};
use std::collections::HashMap;

fn id(text: &str) -> SavedId {
    SavedId::parse(text).unwrap()
}

fn file() -> String {
    json!({
        "schemaVersion": 1,
        "staples": [
            { "ingredient": "salt", "assumed": true },
            { "ingredient": "rice", "assumed": false }
        ]
    })
    .to_string()
}

fn foods(slugs: &[&str]) -> Foods {
    let things: HashMap<String, Value> =
        slugs.iter().map(|slug| (format!("ingredient/{slug}"), json!({ "names": { "en": *slug } }))).collect();
    Foods::from_things(&things)
}

#[test]
fn the_staples_come_in_the_files_order_with_whether_each_is_assumed() {
    let list = StapleList::parse(&file()).unwrap();
    let found: Vec<(String, bool)> =
        list.listed().iter().map(|one| (one.ingredient.to_string(), one.assumed)).collect();
    assert_eq!(found, vec![("salt".to_string(), true), ("rice".to_string(), false)]);
}

#[test]
fn the_assumed_ones_are_the_ones_counted_before_the_cook_answers() {
    let assumed = StapleList::parse(&file()).unwrap().assumed();
    assert_eq!(assumed, [id("salt")].into_iter().collect());
}

#[test]
fn a_staple_the_graph_does_not_know_refuses_the_start() {
    let list = StapleList::parse(&file()).unwrap();
    assert!(list.check_against(&foods(&["salt", "rice"])).is_ok());
    let problem = list.check_against(&foods(&["salt"])).unwrap_err();
    assert!(problem.contains("rice"), "{problem}");
}

#[test]
fn a_staple_listed_twice_an_empty_list_an_unknown_field_or_version_rejects_the_file() {
    let twice = file().replace("\"rice\"", "\"salt\"");
    assert!(StapleList::parse(&twice).unwrap_err().contains("twice"));
    assert!(StapleList::parse(&json!({ "schemaVersion": 1, "staples": [] }).to_string()).is_err());
    assert!(StapleList::parse(&file().replace("\"assumed\":true", "\"assumed\":true,\"x\":1")).is_err());
    assert!(StapleList::parse(&file().replace("\"schemaVersion\":1", "\"schemaVersion\":2")).is_err());
}
