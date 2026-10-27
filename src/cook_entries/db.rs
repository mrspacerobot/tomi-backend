//! Storing cook entries. The first accepted copy of an entry id is the one kept; nothing here ever overwrites it.

use super::ReceivedEntry;
use crate::error::AppError;
use jiff::Timestamp;
use sqlx::PgPool;
use uuid::Uuid;

const INSERT: &str = "
    insert into tomi.cook_entries (
        user_id, id, payload_sha256, schema_version, skill, level, recipe_id, graph_version,
        outcome, weekly_goal, cooked_at, time_zone, received_at)
    values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11::timestamptz, $12, $13::timestamptz)
    on conflict (user_id, id) do nothing";

const STORED_SHA256: &str = "select payload_sha256 from tomi.cook_entries where user_id = $1 and id = $2";

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
    let query = sqlx::query(INSERT).bind(write.user_id).bind(entry.id).bind(write.payload_sha256);
    let query = query.bind(i32::from(entry.schema_version)).bind(entry.skill.as_str());
    let query = query.bind(i16::from(entry.level)).bind(&entry.dish.recipe_id).bind(&entry.dish.graph_version);
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
