use super::*;
use crate::graph::Facts;
use serde_json::{json, Value};
use std::collections::HashMap;

fn file() -> String {
    json!({
        "schemaVersion": 2,
        "allergensFromGraph": true,
        "avoidItems": [
            { "id": "dairy", "name": { "en": "Dairy" }, "avoids": ["fact/milk"] },
            { "id": "nuts", "name": { "en": "Nuts" }, "avoids": ["fact/tree-nuts", "fact/peanuts"] }
        ],
        "diets": [
            { "id": "vegan", "name": { "en": "Vegan" }, "summary": { "en": "Plants only" },
              "avoids": ["fact/milk", "fact/egg"] }
        ]
    })
    .to_string()
}

fn list() -> RestrictionList {
    RestrictionList::parse(&file()).unwrap()
}

fn id(text: &str) -> SavedId {
    SavedId::parse(text).unwrap()
}

fn facts(texts: &[&str]) -> BTreeSet<String> {
    texts.iter().map(|text| text.to_string()).collect()
}

#[test]
fn an_avoid_item_stands_for_every_graph_fact_it_lists() {
    let avoid = BTreeSet::from([id("nuts")]);
    let asked = Restrictions { diet: None, avoid: &avoid };
    assert_eq!(list().facts_to_avoid(&asked).unwrap(), facts(&["fact/tree-nuts", "fact/peanuts"]));
}

#[test]
fn a_diet_and_avoid_items_add_up() {
    let avoid = BTreeSet::from([id("nuts")]);
    let diet = id("vegan");
    let asked = Restrictions { diet: Some(&diet), avoid: &avoid };
    let found = list().facts_to_avoid(&asked).unwrap();
    assert_eq!(found, facts(&["fact/milk", "fact/egg", "fact/tree-nuts", "fact/peanuts"]));
}

#[test]
fn nothing_asked_avoids_nothing() {
    let avoid = BTreeSet::new();
    assert!(list().facts_to_avoid(&Restrictions { diet: None, avoid: &avoid }).unwrap().is_empty());
}

#[test]
fn a_diet_the_list_does_not_have_is_an_error_never_no_restriction() {
    let avoid = BTreeSet::new();
    let diet = id("paleo");
    let result = list().facts_to_avoid(&Restrictions { diet: Some(&diet), avoid: &avoid });
    assert_eq!(result, Err(Refusal::UnknownDiet(diet)));
}

#[test]
fn an_avoid_item_the_list_does_not_have_is_an_error_never_no_restriction() {
    let avoid = BTreeSet::from([id("celery")]);
    let result = list().facts_to_avoid(&Restrictions { diet: None, avoid: &avoid });
    assert_eq!(result, Err(Refusal::UnknownAvoidItem(id("celery"))));
}

#[test]
fn an_item_that_avoids_nothing_rejects_the_file() {
    let text = file().replace(r#""avoids":["fact/milk"]"#, r#""avoids":[]"#);
    assert!(RestrictionList::parse(&text).is_err());
}

#[test]
fn a_fact_that_is_not_a_graph_fact_id_rejects_the_file() {
    let text = file().replace("fact/milk", "dairy");
    assert!(RestrictionList::parse(&text).unwrap_err().contains("dairy"));
}

#[test]
fn an_unknown_field_or_schema_version_rejects_the_file() {
    assert!(RestrictionList::parse(&file().replace("\"schemaVersion\":2", "\"schemaVersion\":3")).is_err());
    assert!(RestrictionList::parse(&file().replace("\"schemaVersion\":2", "\"schemaVersion\":2,\"x\":1")).is_err());
}

#[test]
fn every_fact_must_exist_in_the_graph_or_the_start_is_refused() {
    let known = facts(&["fact/milk", "fact/egg", "fact/tree-nuts"]);
    let problem = list().check_against(&known).unwrap_err();
    assert!(problem.contains("fact/peanuts"));
    assert!(list().check_against(&facts(&["fact/milk", "fact/egg", "fact/tree-nuts", "fact/peanuts"])).is_ok());
}

fn graph_facts() -> Facts {
    let fact = |english: &str, concern: &str| json!({ "names": { "en": english }, "concern": concern });
    let things: HashMap<String, Value> = HashMap::from([
        ("fact/milk".to_string(), fact("Milk", "allergen")),
        ("fact/egg".to_string(), fact("Egg", "allergen")),
        ("fact/tree-nuts".to_string(), fact("Tree nuts", "allergen")),
        ("fact/peanuts".to_string(), fact("Peanuts", "allergen")),
        ("fact/celery".to_string(), fact("Celery", "allergen")),
        ("fact/lupin".to_string(), fact("Lupin", "allergen")),
        ("fact/pork".to_string(), fact("Pork", "diet")),
    ]);
    Facts::from_things(&things)
}

fn extended() -> RestrictionList {
    let mut list = list();
    list.extend_from_graph(&graph_facts()).unwrap();
    list
}

fn listed_ids(list: &RestrictionList) -> Vec<String> {
    list.avoid_items.iter().map(|choice| choice.id.to_string()).collect()
}

#[test]
fn an_allergen_no_item_covers_becomes_an_item_after_the_person_written_ones() {
    // Egg is only ruled out by the vegan diet here, and a diet is not an avoid item, so egg gets one too.
    assert_eq!(listed_ids(&extended()), vec!["dairy", "nuts", "celery", "egg", "lupin"]);
    let celery = extended().avoid_items.remove(2);
    assert_eq!(celery.name.english(), "Celery");
}

#[test]
fn a_fact_a_person_item_already_rules_out_gets_no_item_of_its_own() {
    assert!(!listed_ids(&extended()).contains(&"peanuts".to_string()));
    assert!(!listed_ids(&extended()).contains(&"milk".to_string()));
}

#[test]
fn diet_choices_are_never_made_into_items() {
    assert!(!listed_ids(&extended()).contains(&"pork".to_string()));
}

#[test]
fn a_derived_item_rules_out_its_fact() {
    let avoid = BTreeSet::from([id("celery")]);
    let found = extended().facts_to_avoid(&Restrictions { diet: None, avoid: &avoid }).unwrap();
    assert_eq!(found, facts(&["fact/celery"]));
}

#[test]
fn nuts_still_means_peanuts_and_tree_nuts_whatever_the_graph_calls_a_fact() {
    let avoid = BTreeSet::from([id("nuts")]);
    let found = extended().facts_to_avoid(&Restrictions { diet: None, avoid: &avoid }).unwrap();
    assert_eq!(found, facts(&["fact/tree-nuts", "fact/peanuts"]));
}

#[test]
fn nothing_is_derived_unless_the_file_asks_for_it() {
    let mut list =
        RestrictionList::parse(&file().replace("\"allergensFromGraph\":true", "\"allergensFromGraph\":false")).unwrap();
    list.extend_from_graph(&graph_facts()).unwrap();
    assert_eq!(listed_ids(&list), vec!["dairy", "nuts"]);
}

#[test]
fn a_saved_id_that_is_a_graph_fact_is_read_as_that_fact_even_when_no_item_has_it() {
    let mut list =
        RestrictionList::parse(&file().replace("\"allergensFromGraph\":true", "\"allergensFromGraph\":false")).unwrap();
    list.extend_from_graph(&graph_facts()).unwrap();
    let avoid = BTreeSet::from([id("celery")]);
    let found = list.facts_to_avoid(&Restrictions { diet: None, avoid: &avoid }).unwrap();
    assert_eq!(found, facts(&["fact/celery"]));
}

#[test]
fn an_item_named_like_a_fact_must_rule_that_fact_out() {
    let text = file().replace(r#""id":"dairy""#, r#""id":"egg""#);
    let mut list = RestrictionList::parse(&text).unwrap();
    let problem = list.extend_from_graph(&graph_facts()).unwrap_err();
    assert!(problem.contains("egg"), "{problem}");
}

#[test]
fn an_item_is_an_allergen_when_any_fact_it_rules_out_is_one_and_a_choice_otherwise() {
    let list = extended();
    assert_eq!(list.kind_of(&id("dairy")), Kind::Allergen);
    assert_eq!(list.kind_of(&id("celery")), Kind::Allergen);
    let text = file().replace(r#""avoids":["fact/milk"]"#, r#""avoids":["fact/pork"]"#);
    let mut diet = RestrictionList::parse(&text).unwrap();
    diet.extend_from_graph(&graph_facts()).unwrap();
    assert_eq!(diet.kind_of(&id("dairy")), Kind::Choice);
}

#[test]
fn an_item_nobody_has_heard_of_is_read_as_an_allergen() {
    assert_eq!(list().kind_of(&id("mango")), Kind::Allergen);
}
