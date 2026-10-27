//! A cook's quiz pick, read once at the edge, and checked against the pinned graph's answer key.

use crate::graph::Graph;
use crate::suggestions::{answer_key, KeyedQuestion};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// One pick as the phone sends it. Unknown fields are refused, so the phone can never send a result of its own.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Pick {
    /// The id the phone made when cooking began; the cook entry later carries the same id.
    pub entry_id: Uuid,
    pub recipe_id: String,
    pub graph_version: String,
    pub question_id: String,
    pub choice: usize,
}

/// What the graph says about a pick: whether it is right, and what the stored first answer would hold.
pub struct Checked {
    pub choice: i32,
    pub correct: bool,
    /// Sent only when the pick is right, so a wrong pick gives nothing away.
    pub explanation: Option<String>,
    /// The whole quiz the question belongs to, kept with the first answer.
    pub quiz_question_ids: Vec<String>,
}

/// Why a pick is refused. None of them names the answer.
#[derive(Debug, PartialEq, Eq)]
pub enum PickError {
    /// The pick was made against another graph than the one this server serves.
    GraphVersionChanged,
    /// The question is not part of the quiz taught for that recipe today, or the recipe is unknown.
    QuestionNotInQuiz,
    /// The choice is not one of the question's choices.
    ChoiceOutOfRange,
}

/// The answer the phone gets back. It never holds the correct choice.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QuizAnswer {
    pub question_id: String,
    pub correct: bool,
    pub scored: bool,
    pub explanation: Option<String>,
}

impl Checked {
    /// What the phone is told about its pick of this question.
    pub fn into_answer(self, question_id: String, scored: bool) -> QuizAnswer {
        QuizAnswer { question_id, correct: self.correct, scored, explanation: self.explanation }
    }
}

pub fn check(pick: &Pick, graph: &Graph) -> Result<Checked, PickError> {
    if pick.graph_version != graph.version {
        return Err(PickError::GraphVersionChanged);
    }
    let Some(key) = answer_key(&pick.recipe_id, &pick.question_id, graph) else {
        return Err(PickError::QuestionNotInQuiz);
    };
    let Some(question) = key.question(&pick.question_id) else {
        return Err(PickError::QuestionNotInQuiz);
    };
    let choice = stored_choice(question, pick.choice)?;
    let correct = question.is_answer(pick.choice);
    let explanation = correct.then(|| question.explanation().to_string());
    Ok(Checked { choice, correct, explanation, quiz_question_ids: key.question_ids() })
}

/// The choice as the database stores it, when it is one of the question's choices.
fn stored_choice(question: &KeyedQuestion, choice: usize) -> Result<i32, PickError> {
    if !question.has_choice(choice) {
        return Err(PickError::ChoiceOutOfRange);
    }
    match i32::try_from(choice) {
        Ok(choice) => Ok(choice),
        Err(_) => Err(PickError::ChoiceOutOfRange),
    }
}

/// The first answer stored for a question, as far as it decides whether this pick is that first answer.
#[derive(Debug, PartialEq, Eq)]
pub struct FirstAnswer {
    pub recipe_id: String,
    pub graph_version: String,
    pub choice: i32,
}

/// A pick is scored when it is the stored first answer: the first pick itself, or a retry of exactly that pick,
/// so a retried request is answered the same. Any other pick only teaches.
pub fn is_scored(first: &FirstAnswer, pick: &Pick, checked: &Checked) -> bool {
    first.recipe_id == pick.recipe_id && first.graph_version == pick.graph_version && first.choice == checked.choice
}
