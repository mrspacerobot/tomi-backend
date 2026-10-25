//! Complete graph quizzes with valid stable identities, answers, and content policy.

use super::source::nonblank;
use super::tier::ContentTier;
use crate::graph::Channel;
use crate::kernel::ids::SavedId;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeSet;

#[derive(Deserialize)]
struct QuestionSource {
    id: String,
    question: String,
    choices: Vec<String>,
    answer: usize,
    because: String,
    tier: ContentTier,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Question {
    id: String,
    question: String,
    choices: Vec<String>,
    answer_index: usize,
    explanation: String,
    tier: ContentTier,
}

pub fn read(thing: &Value, technique: &str, channel: Channel) -> Option<Vec<Question>> {
    let sources: Vec<QuestionSource> = serde_json::from_value(thing.get("quiz")?.clone()).ok()?;
    if sources.is_empty() {
        return None;
    }
    let questions = sources.into_iter().map(|source| checked(source, technique, channel));
    let questions = questions.collect::<Option<Vec<_>>>()?;
    let mut ids = BTreeSet::new();
    questions.iter().all(|question| ids.insert(&question.id)).then_some(questions)
}

fn checked(source: QuestionSource, technique: &str, channel: Channel) -> Option<Question> {
    if !source.valid(channel) {
        return None;
    }
    Some(Question {
        id: format!("{technique}/quiz/{}", source.id),
        question: source.question,
        choices: source.choices,
        answer_index: source.answer,
        explanation: source.because,
        tier: source.tier,
    })
}

impl QuestionSource {
    fn valid(&self, channel: Channel) -> bool {
        self.tier.permitted(channel)
            && SavedId::parse(&self.id).is_ok()
            && nonblank(&[&self.question, &self.because])
            && self.choices.len() >= 2
            && self.answer < self.choices.len()
            && self.valid_choices()
    }

    fn valid_choices(&self) -> bool {
        let mut unique = BTreeSet::new();
        self.choices.iter().all(|choice| !choice.trim().is_empty() && unique.insert(choice))
    }
}
