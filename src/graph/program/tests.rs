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
fn recipe_eligibility_expands_technique_variants_downward_only_and_terminates_cycles() {
    let graph = Program::from_things(&variant_things());
    assert_eq!(graph.levels_for(&["technique/base".into()]), vec![("eggs".into(), 1)]);
    assert_eq!(graph.levels_for(&["technique/leaf".into()]), vec![("eggs".into(), 1), ("eggs".into(), 2)]);
    assert!(graph.levels_for(&["technique/unrelated".into()]).is_empty());
    assert_eq!(graph.levels_for(&["technique/cycle-a".into()]), vec![("eggs".into(), 1)]);
}

fn variant_things() -> HashMap<String, Value> {
    let program = json!({"courses": [{"slug": "foundations", "skills": [{
        "slug": "eggs", "levels": [
            {"number": 1, "teaches": [{"thing": "technique/base"}]},
            {"number": 2, "teaches": [{"thing": "technique/leaf"}]}
        ]
    }]}]});
    let mut things = HashMap::from([("program/french".into(), program)]);
    add_variants(&mut things);
    things
}

fn add_variants(things: &mut HashMap<String, Value>) {
    things.insert("technique/child".into(), json!({"variant_of": [{"thing": "technique/base"}]}));
    things.insert("technique/leaf".into(), json!({"variant_of": [{"thing": "technique/child"}]}));
    things.insert("technique/cycle-a".into(), json!({"variant_of": [{"thing": "technique/cycle-b"}]}));
    things.insert(
        "technique/cycle-b".into(),
        json!({"variant_of": [{"thing": "technique/cycle-a"}, {"thing": "technique/base"}]}),
    );
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
