//! Tomi's own user records, one per Supabase user.

use crate::error::AppError;
use sqlx::PgPool;
use uuid::Uuid;

// The no-op update makes RETURNING give back an existing row too.
const FIND_OR_CREATE: &str = "
    insert into tomi.users (auth_subject) values ($1)
    on conflict (auth_subject) do update set auth_subject = excluded.auth_subject
    returning id, deleted_at is not null";

/// Finds or creates the Tomi user for a Supabase user, and returns our own id.
pub async fn for_subject(pool: &PgPool, subject: Uuid) -> Result<Uuid, AppError> {
    let (id, deleted): (Uuid, bool) = sqlx::query_as(FIND_OR_CREATE).bind(subject).fetch_one(pool).await?;
    if deleted {
        return Err(AppError::AccountDeleted);
    }
    Ok(id)
}
