//! Storing cook entries. The first accepted copy of an entry id is the one kept; nothing here ever overwrites it.

use crate::error::AppError;
use crate::progress::received::{Dish, ReceivedEntry};
use crate::progress::{CookEntry, QuizAnswers, StoredEntry};
use jiff::Timestamp;
use sqlx::PgPool;
use uuid::Uuid;

const INSERT: &str = "
    insert into tomi.cook_entries (
        user_id, id, payload_sha256, schema_version, skill, level, lesson_id, recipe_id, graph_version,
        outcome, weekly_goal, cooked_at, time_zone, received_at)
    values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12::timestamptz, $13, $14::timestamptz)
    on conflict (user_id, id) do nothing";

const STORED_SHA256: &str = "select payload_sha256 from tomi.cook_entries where user_id = $1 and id = $2";

/// Every entry of the cook, in no particular order: the rules give the same answer in any.
const COOK_LOG: &str = "
    select id, skill, level, recipe_id, outcome, weekly_goal,
        (extract(epoch from cooked_at) * 1000000)::bigint as cooked_at_microseconds, time_zone
    from tomi.cook_entries where user_id = $1";

type LogRow = (Uuid, String, i16, Option<String>, String, i16, i64, String);

/// An entry to keep for this cook, with its fingerprint and the time it arrived.
pub struct EntryWrite<'a> {
    pub user_id: Uuid,
    pub entry: &'a ReceivedEntry,
    pub payload_sha256: &'a str,
    pub received_at: Timestamp,
}

/// Stores the entry unless the cook already has one with its id. Returns false when one was there first.
pub async fn insert(pool: &PgPool, write: &EntryWrite<'_>) -> Result<bool, AppError> {
    let entry = write.entry;
    let (lesson_id, recipe_id, graph_version) = dish_columns(&entry.dish);
    let query = sqlx::query(INSERT).bind(write.user_id).bind(entry.id).bind(write.payload_sha256);
    let query = query.bind(i32::from(entry.schema_version)).bind(entry.skill.as_str());
    let query = query.bind(i16::from(entry.level)).bind(lesson_id).bind(recipe_id).bind(graph_version);
    let query = query.bind(entry.outcome.as_str()).bind(i16::from(entry.weekly_goal));
    let query = query.bind(entry.cooked_at.to_string()).bind(&entry.time_zone).bind(write.received_at.to_string());
    let result = query.execute(pool).await?;
    Ok(result.rows_affected() == 1)
}

/// The fingerprint of the copy the cook already has under this id.
pub async fn stored_sha256(pool: &PgPool, user_id: Uuid, id: Uuid) -> Result<String, AppError> {
    let query = sqlx::query_scalar(STORED_SHA256).bind(user_id).bind(id);
    Ok(query.fetch_one(pool).await?)
}

/// The lesson, recipe and graph version columns: one kind of dish is filled, the other left empty.
fn dish_columns(dish: &Dish) -> (Option<&str>, Option<&str>, Option<&str>) {
    match dish {
        Dish::Recipe { recipe_id, graph_version } => (None, Some(recipe_id.as_str()), Some(graph_version.as_str())),
        Dish::Lesson { lesson_id } => (Some(lesson_id.as_str()), None, None),
    }
}

/// The cook's whole log, read fresh on every request with the cook's recorded quiz answers: progress and the quiz
/// result are never kept.
pub async fn cook_log(pool: &PgPool, user_id: Uuid) -> Result<Vec<CookEntry>, AppError> {
    let answers = QuizAnswers::new(crate::quiz::first_answers(pool, user_id).await?);
    let rows: Vec<LogRow> = sqlx::query_as(COOK_LOG).bind(user_id).fetch_all(pool).await?;
    let mut log = Vec::with_capacity(rows.len());
    for (id, skill, level, recipe_id, outcome, weekly_goal, cooked_at_microseconds, time_zone) in rows {
        let stored =
            StoredEntry { id, skill, level, recipe_id, outcome, weekly_goal, cooked_at_microseconds, time_zone };
        log.push(stored.read(&answers)?);
    }
    Ok(log)
}
