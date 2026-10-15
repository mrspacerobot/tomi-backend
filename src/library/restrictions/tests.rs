use super::*;
use serde_json::json;

fn file() -> String {
    json!({
        "schemaVersion": 1,
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
    assert!(RestrictionList::parse(&file().replace("\"schemaVersion\":1", "\"schemaVersion\":2")).is_err());
    assert!(RestrictionList::parse(&file().replace("\"schemaVersion\":1", "\"schemaVersion\":1,\"x\":1")).is_err());
}

#[test]
fn every_fact_must_exist_in_the_graph_or_the_start_is_refused() {
    let known = facts(&["fact/milk", "fact/egg", "fact/tree-nuts"]);
    let problem = list().check_against(&known).unwrap_err();
    assert!(problem.contains("fact/peanuts"));
    assert!(list().check_against(&facts(&["fact/milk", "fact/egg", "fact/tree-nuts", "fact/peanuts"])).is_ok());
}
