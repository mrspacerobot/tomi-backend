//! What Tomi asks the model, and the JSON shape the answer must take.
//!
//! Allowed ids go in the prompt for scans and the schema for short recipe lists.
//! The server checks returned ids against the graph before anyone trusts them.

use super::{FridgeQuestion, RankQuestion};
use crate::suggestions::Candidate;
use base64::Engine;
use serde_json::{json, Value};

const FRIDGE_INSTRUCTIONS: &str = "You look at a photo of the inside of a fridge, a cupboard or a kitchen counter. \
List each food item you can clearly see. For each item give: a short English label, \
the matching ingredient id, one of the allowed values, only if the item is clearly that ingredient (otherwise null), \
and how sure you are, from 0 to 1. Do not guess at hidden or unreadable items. \
Never judge freshness, doneness or whether food is safe to eat.";

const RANK_INSTRUCTIONS: &str = "You help a home cook choose tonight's recipe. \
Rank the recipes below, best first. Prefer recipes that use what the cook has and need little shopping, \
then ones that fit the cook's experience, loved cuisines and focus skill. \
For each recipe give a reason of at most 15 words, warm and plain, spoken to the cook. \
Never say a dish is safe, healthy or cooked through. Use only the recipe ids given.";

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

pub fn rank_request(question: &RankQuestion) -> Value {
    let recipes: Vec<Value> = question.candidates.iter().map(recipe_summary).collect();
    let cook = json!({
        "experience": question.profile.experience,
        "cuisinesLoved": question.profile.cuisines_loved,
        "focusSkill": question.profile.focus_skill,
    });
    let text = format!("{RANK_INSTRUCTIONS}\n\nThe cook: {cook}\n\nRecipes: {}", Value::from(recipes));
    let ids: Vec<&str> = question.candidates.iter().map(|candidate| candidate.recipe_id.as_str()).collect();
    request(vec![json!({ "text": text })], rank_schema(&ids))
}

fn recipe_summary(candidate: &Candidate) -> Value {
    json!({
        "recipeId": candidate.recipe_id,
        "title": candidate.title.english(),
        "skill": candidate.skill,
        "level": candidate.level,
        "toBuy": candidate.missing,
        "swaps": candidate.swaps,
    })
}

fn rank_schema(recipe_ids: &[&str]) -> Value {
    let choice = json!({
        "type": "OBJECT",
        "properties": {
            "recipeId": { "type": "STRING", "enum": recipe_ids },
            "reason": { "type": "STRING" }
        },
        "required": ["recipeId", "reason"]
    });
    json!({ "type": "OBJECT", "properties": { "ranking": { "type": "ARRAY", "items": choice } }, "required": ["ranking"] })
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
