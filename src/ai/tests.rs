use super::gemini::{answer_text, parse_ranking};
use super::prompts::{fridge_request, rank_request};
use super::*;
use crate::domain::photo::Photo;
use serde_json::json;

fn id(raw: &str) -> SavedId {
    SavedId::parse(raw).unwrap()
}

#[test]
fn the_fridge_request_carries_the_photo_and_a_closed_list_of_ids() {
    let photo = Photo::parse(Some("image/jpeg"), vec![0xFF, 0xD8, 0xFF, 0xD9]).unwrap();
    let ingredients = BTreeMap::from([(id("egg"), "Egg".to_string()), (id("leek"), "Leek".to_string())]);
    let body = fridge_request(&FridgeQuestion { photo: &photo, ingredients: &ingredients });
    let parts = &body["contents"][0]["parts"];
    assert_eq!(parts[1]["inline_data"]["mime_type"], "image/jpeg");
    assert_eq!(parts[1]["inline_data"]["data"], "/9j/2Q==");
    let schema = &body["generationConfig"]["response_schema"];
    assert_eq!(schema["properties"]["items"]["items"]["properties"]["ingredient"]["enum"], json!(["egg", "leek"]));
    assert!(parts[0]["text"].as_str().unwrap().contains("egg: Egg"));
}

#[test]
fn the_rank_request_lists_only_the_candidates() {
    let candidate = Candidate {
        lesson_id: id("omelette"),
        revision: 1,
        title: "Omelette".into(),
        minutes: 10,
        skill: id("eggs"),
        level: 3,
        ingredients: vec![id("egg")],
        swaps: vec![],
        missing: vec![],
    };
    let profile = Profile::default();
    let body = rank_request(&RankQuestion { candidates: &[candidate], profile: &profile });
    let schema = &body["generationConfig"]["response_schema"];
    assert_eq!(schema["properties"]["ranking"]["items"]["properties"]["lessonId"]["enum"], json!(["omelette"]));
}

#[test]
fn the_answer_text_is_read_from_the_first_candidate() {
    let answer = json!({ "candidates": [{ "content": { "parts": [{ "text": "{\"items\": []}" }] } }] });
    assert_eq!(answer_text(&answer).as_deref(), Some("{\"items\": []}"));
    assert_eq!(answer_text(&json!({ "promptFeedback": { "blockReason": "SAFETY" } })), None);
}

#[test]
fn a_ranking_skips_ids_that_cannot_be_ids() {
    let text =
        r#"{"ranking": [{"lessonId": "Not An Id", "reason": "x"}, {"lessonId": "omelette", "reason": "Quick"}]}"#;
    let ranked = parse_ranking(text).unwrap();
    assert_eq!(ranked, vec![Ranked { lesson_id: id("omelette"), reason: Some("Quick".into()) }]);
    assert!(parse_ranking("not json").is_err());
}
