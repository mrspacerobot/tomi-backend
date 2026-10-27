//! A first answer as the quiz keeps it, read back so progress can derive each cook's quiz result.

use uuid::Uuid;

/// The first answer a cook gave to one question of the quiz of one cook entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordedAnswer {
    /// The cook entry the answer belongs to: the id the phone made when cooking began.
    pub entry_id: Uuid,
    pub question_id: String,
    /// The recipe the quiz was answered for.
    pub recipe_id: String,
    pub correct: bool,
    /// Every question of the quiz when this question was first answered.
    pub quiz_question_ids: Vec<String>,
}
