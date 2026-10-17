use super::*;
use serde_json::json;

fn things() -> HashMap<String, Value> {
    let fact = |english: &str, concern: &str| json!({ "names": { "en": english }, "concern": concern });
    HashMap::from([
        ("fact/milk".to_string(), fact("Milk", "allergen")),
        ("fact/pork".to_string(), fact("Pork", "diet")),
        ("fact/celery".to_string(), fact("Celery", "allergen")),
        ("fact/odd".to_string(), fact("Odd", "something-new")),
        ("ingredient/egg".to_string(), json!({ "names": { "en": "Egg" } })),
    ])
}

#[test]
fn a_fact_has_its_name_and_whether_it_is_an_allergen_or_a_diet_choice() {
    let facts = Facts::from_things(&things());
    assert_eq!(facts.get("fact/milk").unwrap().concern, Concern::Allergen);
    assert_eq!(facts.get("fact/pork").unwrap().concern, Concern::Diet);
    assert_eq!(facts.get("fact/pork").unwrap().name.english(), "Pork");
    assert!(facts.get("ingredient/egg").is_none());
}

#[test]
fn a_concern_this_build_does_not_know_is_read_as_an_allergen() {
    assert_eq!(Facts::from_things(&things()).get("fact/odd").unwrap().concern, Concern::Allergen);
}

#[test]
fn the_allergens_come_in_id_order_and_leave_out_diet_choices() {
    let ids: Vec<String> = Facts::from_things(&things()).allergens().map(|(id, _)| id.clone()).collect();
    assert_eq!(ids, vec!["fact/celery", "fact/milk", "fact/odd"]);
}

#[test]
fn a_fact_without_a_readable_name_is_left_out_and_counted() {
    let mut things = things();
    things.insert("fact/nameless".to_string(), json!({ "concern": "allergen" }));
    let facts = Facts::from_things(&things);
    assert!(facts.get("fact/nameless").is_none());
    assert_eq!(facts.left_out(), 1);
}
