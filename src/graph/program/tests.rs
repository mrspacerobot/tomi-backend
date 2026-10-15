use super::*;
use serde_json::json;

fn things() -> HashMap<String, Value> {
    let program = json!({
        "courses": [{
            "slug": "foundations",
            "skills": [{
                "slug": "eggs",
                "levels": [
                    { "number": 3, "teaches": [{ "thing": "technique/roll-an-omelette", "tier": "drafted" }] },
                    { "number": 4, "teaches": [] }
                ]
            }]
        }]
    });
    HashMap::from([("program/french".to_string(), program), ("technique/sear".to_string(), json!({}))])
}

#[test]
fn a_level_names_the_techniques_it_teaches() {
    let found = Program::from_things(&things());
    assert_eq!(found.teaches("eggs", 3), vec!["technique/roll-an-omelette".to_string()]);
}

#[test]
fn a_level_that_teaches_nothing_or_does_not_exist_names_no_technique() {
    let found = Program::from_things(&things());
    assert!(found.teaches("eggs", 4).is_empty());
    assert!(found.teaches("eggs", 9).is_empty());
    assert!(found.teaches("soups", 1).is_empty());
}

#[test]
fn a_program_this_server_cannot_read_teaches_nothing_rather_than_something_wrong() {
    let broken = HashMap::from([("program/bad".to_string(), json!({ "courses": "oops" }))]);
    assert!(Program::from_things(&broken).teaches("eggs", 3).is_empty());
}
