//! Keeping each question's first answer, once per cook, Supabase account and question.

use super::pick::{Checked, FirstAnswer, Pick};
use crate::error::AppError;
use sqlx::PgPool;
use uuid::Uuid;

// The first insert wins: a later pick of the same question changes nothing.
const INSERT_FIRST: &str = "
    insert into tomi.quiz_answers
        (user_id, entry_id, question_id, recipe_id, graph_version, choice, correct, quiz_question_ids)
    values ($1, $2, $3, $4, $5, $6, $7, $8)
    on conflict (user_id, entry_id, question_id) do nothing";

const READ_FIRST: &str = "
    select recipe_id, graph_version, choice from tomi.quiz_answers
    where user_id = $1 and entry_id = $2 and question_id = $3";

/// A checked pick of one cook, to keep when it is the first answer to its question.
pub struct Answered<'a> {
    pub user_id: Uuid,
    pub pick: &'a Pick,
    pub checked: &'a Checked,
}

/// Keeps the pick when it is the first answer to its question, and returns the first answer now stored.
pub async fn keep_first(pool: &PgPool, answered: &Answered<'_>) -> Result<FirstAnswer, AppError> {
    let (pick, checked) = (answered.pick, answered.checked);
    sqlx::query(INSERT_FIRST)
        .bind(answered.user_id)
        .bind(pick.entry_id)
        .bind(&pick.question_id)
        .bind(&pick.recipe_id)
        .bind(&pick.graph_version)
        .bind(checked.choice)
        .bind(checked.correct)
        .bind(&checked.quiz_question_ids)
        .execute(pool)
        .await?;
    read_first(pool, answered).await
}

async fn read_first(pool: &PgPool, answered: &Answered<'_>) -> Result<FirstAnswer, AppError> {
    let query = sqlx::query_as(READ_FIRST).bind(answered.user_id).bind(answered.pick.entry_id);
    let row: (String, String, i32) = query.bind(&answered.pick.question_id).fetch_one(pool).await?;
    let (recipe_id, graph_version, choice) = row;
    Ok(FirstAnswer { recipe_id, graph_version, choice })
}
