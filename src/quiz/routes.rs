//! `POST /v1/quiz-answers`: one pick in, right or wrong out.

use super::db::{self as store, Answered};
use super::pick::{self, PickError, QuizAnswer};
use crate::error::AppError;
use crate::http::{json_body, Caller};
use crate::AppState;
use axum::extract::rejection::JsonRejection;
use axum::routing::post;
use axum::{Json, Router};

pub fn router() -> Router<AppState> {
    Router::new().route("/v1/quiz-answers", post(answer))
}

async fn answer(caller: Caller, body: Result<Json<pick::Pick>, JsonRejection>) -> Result<Json<QuizAnswer>, AppError> {
    let pick = json_body(body)?;
    let Some(graph) = caller.state.graph.as_deref() else {
        return Err(AppError::GraphUnavailable);
    };
    let checked = pick::check(&pick, graph).map_err(refusal)?;
    let answered = Answered { user_id: caller.user_id, pick: &pick, checked: &checked };
    let first = store::keep_first(&caller.state.db, &answered).await?;
    let scored = pick::is_scored(&first, &pick, &checked);
    Ok(Json(checked.into_answer(pick.question_id, scored)))
}

/// The refusal names a problem code, never the answer.
fn refusal(error: PickError) -> AppError {
    match error {
        PickError::GraphVersionChanged => AppError::GraphVersionChanged,
        PickError::QuestionNotInQuiz => AppError::Invalid(vec!["question_not_in_quiz".into()]),
        PickError::ChoiceOutOfRange => AppError::Invalid(vec!["choice_out_of_range".into()]),
    }
}
