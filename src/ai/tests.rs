use super::gemini::{answer_text, parse_ranking};
use super::prompts::{fridge_request, rank_request};
use super::*;
use crate::library::LocalizedText;
use crate::scans::photo::Photo;
use serde_json::json;

fn id(raw: &str) -> SavedId {
    SavedId::parse(raw).unwrap()
}

#[test]
fn the_fridge_request_carries_the_photo_and_a_closed_list_of_ids_once() {
    let photo = Photo::parse(Some("image/jpeg"), vec![0xFF, 0xD8, 0xFF, 0xD9]).unwrap();
    let ingredients = vec![id("egg"), id("leek")];
    let body = fridge_request(&FridgeQuestion { photo: &photo, ingredients: &ingredients });
    let parts = &body["contents"][0]["parts"];
    assert_eq!(parts[1]["inline_data"]["mime_type"], "image/jpeg");
    assert_eq!(parts[1]["inline_data"]["data"], "/9j/2Q==");
    let schema = &body["generationConfig"]["response_schema"];
    assert!(schema["properties"]["items"]["items"]["properties"]["ingredient"].get("enum").is_none());
    let text = parts[0]["text"].as_str().unwrap();
    assert!(text.contains(r#"["egg","leek"]"#), "the closed vocabulary belongs in the prompt");
    assert!(text.contains("allowed values"));
}

#[test]
fn the_rank_request_lists_only_the_candidates() {
    let candidate = candidate();
    let profile = Profile::default();
    let body = rank_request(&RankQuestion { candidates: &[candidate], profile: &profile });
    let schema = &body["generationConfig"]["response_schema"];
    assert_eq!(schema["properties"]["ranking"]["items"]["properties"]["recipeId"]["enum"], json!(["recipe/omelette"]));
}

fn candidate() -> Candidate {
    Candidate {
        recipe_id: "recipe/omelette".into(),
        source_recipe: json!({}),
        title: serde_json::from_value::<LocalizedText>(json!({"en": "Omelette"})).unwrap(),
        skill: id("eggs"),
        level: 3,
        ingredients: vec!["ingredient/egg".into()],
        swaps: vec![],
        omitted: vec![],
        missing: vec![],
        equipment: vec![],
    }
}

#[test]
fn the_answer_text_is_read_from_the_first_candidate() {
    let answer = json!({ "candidates": [{ "content": { "parts": [{ "text": "{\"items\": []}" }] } }] });
    assert_eq!(answer_text(&answer).as_deref(), Some("{\"items\": []}"));
    assert_eq!(answer_text(&json!({ "promptFeedback": { "blockReason": "SAFETY" } })), None);
}

#[test]
fn a_ranking_reads_graph_recipe_ids_as_strings() {
    let text = r#"{"ranking": [{"recipeId": "Not A Recipe", "reason": "x"}, {"recipeId": "recipe/omelette", "reason": "Quick"}]}"#;
    let ranked = parse_ranking(text).unwrap();
    assert_eq!(
        ranked,
        vec![
            Ranked { recipe_id: "Not A Recipe".into(), reason: Some("x".into()) },
            Ranked { recipe_id: "recipe/omelette".into(), reason: Some("Quick".into()) },
        ]
    );
    assert!(parse_ranking("not json").is_err());
}
