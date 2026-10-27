//! Whether a finished cook passed its quiz, derived from the first answers recorded under its entry id.

use crate::quiz::RecordedAnswer;
use std::collections::{BTreeMap, BTreeSet};
use uuid::Uuid;

#[cfg(test)]
mod tests;

/// Every first answer of one cook, grouped by the cook entry it belongs to.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct QuizAnswers {
    by_entry: BTreeMap<Uuid, Vec<RecordedAnswer>>,
}

impl QuizAnswers {
    pub fn new(answers: Vec<RecordedAnswer>) -> Self {
        let mut by_entry: BTreeMap<Uuid, Vec<RecordedAnswer>> = BTreeMap::new();
        for answer in answers {
            by_entry.entry(answer.entry_id).or_default().push(answer);
        }
        QuizAnswers { by_entry }
    }

    /// Whether the cook entry with this id, of this recipe, passed its quiz: it has first answers, all for its own
    /// recipe, every question of the quiz has one, and every one is right. A lesson entry names no recipe, so no
    /// quiz of its own.
    pub fn passed(&self, entry_id: Uuid, recipe_id: Option<&str>) -> bool {
        let Some(recipe_id) = recipe_id else {
            return false;
        };
        let Some(answers) = self.by_entry.get(&entry_id) else {
            return false;
        };
        let for_this_recipe = answers.iter().all(|answer| answer.recipe_id == recipe_id);
        let all_right = answers.iter().all(|answer| answer.correct);
        for_this_recipe && all_right && every_question_answered(answers)
    }
}

/// Whether each question of the quiz has a first answer. The quiz is every question any first answer was kept
/// with, so a quiz that grew between two answers still needs all of its questions.
fn every_question_answered(answers: &[RecordedAnswer]) -> bool {
    let answered: BTreeSet<&str> = answers.iter().map(|answer| answer.question_id.as_str()).collect();
    let mut quiz: BTreeSet<&str> = BTreeSet::new();
    for answer in answers {
        quiz.extend(answer.quiz_question_ids.iter().map(String::as_str));
    }
    quiz.is_subset(&answered)
}
