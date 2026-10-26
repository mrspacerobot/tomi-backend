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

/// A question as the phone sees it: never its answer or explanation.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Question {
    id: String,
    question: String,
    choices: Vec<String>,
    tier: ContentTier,
}

/// A question with its answer and explanation, for checking a pick on this server.
/// Deliberately neither `Serialize` nor `Debug`, so the answer cannot reach a reply or a log by accident.
pub struct KeyedQuestion {
    question: Question,
    answer: usize,
    explanation: String,
}

impl KeyedQuestion {
    pub fn id(&self) -> &str {
        &self.question.id
    }

    pub fn has_choice(&self, choice: usize) -> bool {
        choice < self.question.choices.len()
    }

    pub fn is_answer(&self, choice: usize) -> bool {
        choice == self.answer
    }

    pub fn explanation(&self) -> &str {
        &self.explanation
    }

    pub fn into_question(self) -> Question {
        self.question
    }
}

pub fn read(thing: &Value, technique: &str, channel: Channel) -> Option<Vec<KeyedQuestion>> {
    let sources: Vec<QuestionSource> = serde_json::from_value(thing.get("quiz")?.clone()).ok()?;
    if sources.is_empty() {
        return None;
    }
    let questions = sources.into_iter().map(|source| checked(source, technique, channel));
    let questions = questions.collect::<Option<Vec<_>>>()?;
    let mut ids = BTreeSet::new();
    questions.iter().all(|question| ids.insert(question.id().to_string())).then_some(questions)
}

fn checked(source: QuestionSource, technique: &str, channel: Channel) -> Option<KeyedQuestion> {
    if !source.valid(channel) {
        return None;
    }
    let id = format!("{technique}/quiz/{}", source.id);
    let question = Question { id, question: source.question, choices: source.choices, tier: source.tier };
    Some(KeyedQuestion { question, answer: source.answer, explanation: source.because })
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
