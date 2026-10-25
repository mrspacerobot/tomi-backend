//! What Tomi asks the model to identify in fridge photos and the expected JSON shape.

use super::FridgeQuestion;
use base64::Engine;
use serde_json::{json, Value};

const FRIDGE_INSTRUCTIONS: &str = "You look at a photo of the inside of a fridge, a cupboard or a kitchen counter. \
List each food item you can clearly see. For each item give: a short English label, \
the matching ingredient id, one of the allowed values, only if the item is clearly that ingredient (otherwise null), \
and how sure you are, from 0 to 1. Do not guess at hidden or unreadable items. \
Never judge freshness, doneness or whether food is safe to eat.";

pub fn fridge_request(question: &FridgeQuestion) -> Value {
    let ids: Vec<&str> = question.ingredients.iter().map(|id| id.as_str()).collect();
    let text = format!("{FRIDGE_INSTRUCTIONS}\n\nAllowed ingredient ids: {}", json!(ids));
    let image = base64::engine::general_purpose::STANDARD.encode(&question.photo.bytes);
    let image_part = json!({ "inline_data": { "mime_type": question.photo.kind.mime_type(), "data": image } });
    let mut body = request(vec![json!({ "text": text }), image_part], fridge_schema());
    body["generationConfig"]["thinkingConfig"] = json!({ "thinkingLevel": "low" });
    body
}

fn fridge_schema() -> Value {
    let item = json!({
        "type": "OBJECT",
        "properties": {
            "label": { "type": "STRING" },
            "ingredient": { "type": "STRING", "nullable": true },
            "confidence": { "type": "NUMBER" }
        },
        "required": ["label", "confidence"]
    });
    json!({ "type": "OBJECT", "properties": { "items": { "type": "ARRAY", "items": item } }, "required": ["items"] })
}

fn request(parts: Vec<Value>, schema: Value) -> Value {
    json!({
        "contents": [{ "role": "user", "parts": parts }],
        "generationConfig": {
            "response_mime_type": "application/json",
            "response_schema": schema,
            "temperature": 0.2
        }
    })
}
