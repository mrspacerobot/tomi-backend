//! Changes to a pinned excerpt are repinned in memory for deterministic boundary tests.

use super::*;
use tomi_api::graph::{sha256_hex, Graph, GraphFiles, GraphPin, GraphPolicy};

pub fn changed(change: impl FnOnce(&mut Value)) -> Graph {
    let mut body: Value = serde_json::from_str(include_str!("../../fixtures/graph/graph-artifact.json")).unwrap();
    change(&mut body);
    load(body)
}

fn load(body: Value) -> Graph {
    let artifact = body.to_string();
    let checksum = sha256_hex(&artifact);
    let manifest = json!({"artifactSchemaVersion": 3, "channel": body["channel"], "contentSha256": checksum,
        "graphVersion": "fixture", "sourceCommit": "fixture"})
    .to_string();
    let files = GraphFiles { manifest, artifact };
    let pin = GraphPin { version: "fixture".into(), sha256: checksum };
    Graph::load(&files, &pin, &GraphPolicy { allow_draft: true }).unwrap()
}

pub fn expected() -> Value {
    serde_json::from_str(include_str!("../../../docs/examples/teaching.json")).unwrap()
}

pub fn duplicate_question(body: &mut Value) {
    let question = body["things"][TECHNIQUE]["quiz"][0].clone();
    body["things"][TECHNIQUE]["quiz"].as_array_mut().unwrap().push(question);
}

pub fn multiple_techniques(body: &mut Value) {
    let program = body["things"].as_object_mut().unwrap().values_mut().find(|thing| thing["courses"].is_array());
    let program = program.unwrap();
    let skills = program["courses"][0]["skills"].as_array_mut().unwrap();
    let level = &mut skills.iter_mut().find(|skill| skill["slug"] == "eggs").unwrap()["levels"][0];
    level["teaches"] = json!([{"thing": "technique/whisk"}, {"thing": TECHNIQUE}]);
    body["serving"]["recipes"][RECIPE]["techniques"] = json!(["technique/whisk", TECHNIQUE]);
}

pub fn reviewed(body: &mut Value) {
    body["channel"] = json!("production");
    body["things"][RECIPE]["tier"] = json!("reviewed");
    body["serving"]["recipes"][RECIPE]["tier"] = json!("reviewed");
    body["things"][TECHNIQUE]["tier"] = json!("reviewed");
    body["things"][TECHNIQUE]["intro"]["tier"] = json!("reviewed");
    for question in body["things"][TECHNIQUE]["quiz"].as_array_mut().unwrap() {
        question["tier"] = json!("reviewed");
    }
}

pub fn variant(body: &mut Value) {
    let id = "technique/scramble-variant";
    body["things"][id] = body["things"][TECHNIQUE].clone();
    body["things"][id]["variant_of"] = json!([{"thing": TECHNIQUE}]);
    body["things"][id]["id"] = json!(id);
    body["serving"]["recipes"][RECIPE]["techniques"] = json!(["technique/whisk", id]);
    for step in body["things"][RECIPE]["steps"].as_array_mut().unwrap() {
        if step["technique"] == TECHNIQUE {
            step["technique"] = json!(id);
        }
    }
}

pub fn tier_paths() -> [Vec<&'static str>; 4] {
    [
        vec!["things", RECIPE, "tier"],
        vec!["serving", "recipes", RECIPE, "tier"],
        vec!["things", TECHNIQUE, "tier"],
        vec!["things", TECHNIQUE, "intro", "tier"],
    ]
}

pub fn below_reviewed(body: &mut Value, path: &[&str]) {
    reviewed(body);
    let mut current = body;
    for key in path {
        current = &mut current[*key];
    }
    *current = json!("sourced");
}

pub fn invalid_quiz_fields() -> [(&'static str, Value); 7] {
    [
        ("choices", json!(["one", "one"])),
        ("choices", json!([" ", "two"])),
        ("choices", json!(["one"])),
        ("question", json!(" ")),
        ("because", json!("")),
        ("tier", json!("future")),
        ("answer", json!(-1)),
    ]
}
