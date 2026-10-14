//! What Tomi asks the model, and the JSON shape the answer must take.
//!
//! The shapes use closed lists of ids, so the model cannot name an
//! ingredient or a lesson that does not exist. `domain` checks again.

use super::{FridgeQuestion, RankQuestion};
use crate::special::Candidate;
use base64::Engine;
use serde_json::{json, Value};

const FRIDGE_INSTRUCTIONS: &str = "You look at a photo of the inside of a fridge, a cupboard or a kitchen counter. \
List each food item you can clearly see. For each item give: a short English label, \
the matching ingredient id from the list below only if the item is clearly that ingredient (otherwise null), \
and how sure you are, from 0 to 1. Do not guess at hidden or unreadable items. \
Never judge freshness, doneness or whether food is safe to eat.";

const RANK_INSTRUCTIONS: &str = "You help a home cook choose tonight's cooking lesson. \
Rank the lessons below, best first. Prefer lessons that use what the cook has and need little shopping, \
then ones that fit the cook's experience, loved cuisines and focus skill. \
For each lesson give a reason of at most 15 words, warm and plain, spoken to the cook. \
Never say a dish is safe, healthy or cooked through. Use only the lesson ids given.";

pub fn fridge_request(question: &FridgeQuestion) -> Value {
    let list: Vec<String> = question.ingredients.iter().map(|(id, name)| format!("{id}: {name}")).collect();
    let text = format!("{FRIDGE_INSTRUCTIONS}\n\nIngredient ids:\n{}", list.join("\n"));
    let ids: Vec<&str> = question.ingredients.keys().map(|id| id.as_str()).collect();
    let image = base64::engine::general_purpose::STANDARD.encode(&question.photo.bytes);
    let image_part = json!({ "inline_data": { "mime_type": question.photo.kind.mime_type(), "data": image } });
    request(vec![json!({ "text": text }), image_part], fridge_schema(&ids))
}

fn fridge_schema(ingredient_ids: &[&str]) -> Value {
    let item = json!({
        "type": "OBJECT",
        "properties": {
            "label": { "type": "STRING" },
            "ingredient": { "type": "STRING", "enum": ingredient_ids, "nullable": true },
            "confidence": { "type": "NUMBER" }
        },
        "required": ["label", "confidence"]
    });
    json!({ "type": "OBJECT", "properties": { "items": { "type": "ARRAY", "items": item } }, "required": ["items"] })
}

pub fn rank_request(question: &RankQuestion) -> Value {
    let lessons: Vec<Value> = question.candidates.iter().map(lesson_summary).collect();
    let cook = json!({
        "experience": question.profile.experience,
        "cuisinesLoved": question.profile.cuisines_loved,
        "focusSkill": question.profile.focus_skill,
    });
    let text = format!("{RANK_INSTRUCTIONS}\n\nThe cook: {cook}\n\nLessons: {}", Value::from(lessons));
    let ids: Vec<&str> = question.candidates.iter().map(|candidate| candidate.lesson_id.as_str()).collect();
    request(vec![json!({ "text": text })], rank_schema(&ids))
}

fn lesson_summary(candidate: &Candidate) -> Value {
    json!({
        "lessonId": candidate.lesson_id,
        "title": candidate.title,
        "minutes": candidate.minutes,
        "skill": candidate.skill,
        "level": candidate.level,
        "toBuy": candidate.missing,
        "swaps": candidate.swaps,
    })
}

fn rank_schema(lesson_ids: &[&str]) -> Value {
    let choice = json!({
        "type": "OBJECT",
        "properties": {
            "lessonId": { "type": "STRING", "enum": lesson_ids },
            "reason": { "type": "STRING" }
        },
        "required": ["lessonId", "reason"]
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
