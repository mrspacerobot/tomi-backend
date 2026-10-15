use super::*;
use serde_json::json;

fn things() -> HashMap<String, Value> {
    let mut things = HashMap::new();
    things.insert("ingredient/olive-oil".into(), json!({ "names": { "en": "Olive oil", "fr": "Huile d'olive" } }));
    things.insert("ingredient/egg".into(), json!({ "names": { "en": "Egg" } }));
    things.insert("preparation/roux".into(), json!({ "names": { "en": "Roux" } }));
    things.insert("technique/sear".into(), json!({ "names": { "en": "Sear" } }));
    things
}

fn id(text: &str) -> SavedId {
    SavedId::parse(text).unwrap()
}

#[test]
fn only_ingredients_are_foods_listed_by_their_slug_in_id_order() {
    let foods = Foods::from_things(&things());
    let ids: Vec<&str> = foods.listed().iter().map(|choice| choice.id.as_str()).collect();
    assert_eq!(ids, vec!["egg", "olive-oil"]);
}

#[test]
fn a_food_has_its_names_in_every_language_the_graph_has() {
    let foods = Foods::from_things(&things());
    let oil = foods.choice(&id("olive-oil")).unwrap();
    assert_eq!(oil.name.text("fr"), "Huile d'olive");
    assert_eq!(oil.name.text("de"), "Olive oil");
}

#[test]
fn the_ids_for_the_model_are_every_food_in_id_order() {
    assert_eq!(Foods::from_things(&things()).ids(), vec![id("egg"), id("olive-oil")]);
}

#[test]
fn a_thing_with_an_id_or_name_this_server_cannot_read_is_left_out_and_counted() {
    let mut things = things();
    things.insert("ingredient/Bad_Id".into(), json!({ "names": { "en": "Bad" } }));
    things.insert("ingredient/no-english".into(), json!({ "names": { "fr": "Sans" } }));
    let foods = Foods::from_things(&things);
    assert_eq!(foods.listed().len(), 2);
    assert_eq!(foods.left_out(), 2);
}

#[test]
fn an_empty_list_knows_nothing() {
    let foods = Foods::empty();
    assert!(!foods.knows(&id("egg")));
    assert!(foods.listed().is_empty());
}
