//! The answers to a quiz this recipe is taught with today, kept on the server to check the cook's picks.

use super::quiz::KeyedQuestion;
use super::{recipe_permitted, selection, taught_parts};
use crate::graph::Graph;
use crate::kernel::language::Language;

/// One whole quiz with its answers. Neither `Serialize` nor `Debug`: it never leaves this server.
pub struct AnswerKey {
    questions: Vec<KeyedQuestion>,
}

impl AnswerKey {
    pub fn question(&self, id: &str) -> Option<&KeyedQuestion> {
        self.questions.iter().find(|question| question.id() == id)
    }

    /// Every question of the quiz, in the order the cook is asked them.
    pub fn question_ids(&self) -> Vec<String> {
        self.questions.iter().map(|question| question.id().to_string()).collect()
    }
}

/// The quiz the question id belongs to, when the pinned graph teaches that recipe with it under today's teaching
/// rules: the recipe and its technique may be taught on this channel, and the technique is the one taught at one of
/// the recipe's skill levels. The question itself may still be missing from the quiz.
pub fn answer_key(recipe_id: &str, question_id: &str, graph: &Graph) -> Option<AnswerKey> {
    let source_recipe = graph.thing(recipe_id)?;
    if !recipe_permitted(recipe_id, source_recipe, graph) {
        return None;
    }
    let (technique, _) = question_id.rsplit_once("/quiz/")?;
    if !selection::taught(recipe_id, graph).contains(technique) {
        return None;
    }
    let (_, quiz) = taught_parts(technique, graph, &Language::english())?;
    Some(AnswerKey { questions: quiz? })
}
