//! The consents a cook has given, one row per purpose.

use crate::domain::consent::Purpose;
use crate::error::AppError;
use sqlx::PgPool;
use uuid::Uuid;

const GIVE: &str = "
    insert into tomi.consents (user_id, purpose, version) values ($1, $2, $3)
    on conflict (user_id, purpose) do update set version = excluded.version, given_at = now()";

const VERSION: &str = "select version from tomi.consents where user_id = $1 and purpose = $2";

const WITHDRAW: &str = "delete from tomi.consents where user_id = $1 and purpose = $2";

pub async fn give(pool: &PgPool, user_id: Uuid, purpose: Purpose) -> Result<(), AppError> {
    let query = sqlx::query(GIVE).bind(user_id).bind(purpose.slug());
    query.bind(purpose.current_version()).execute(pool).await?;
    Ok(())
}

/// Whether the cook agreed to the current text for this purpose.
pub async fn is_current(pool: &PgPool, user_id: Uuid, purpose: Purpose) -> Result<bool, AppError> {
    let query = sqlx::query_scalar(VERSION).bind(user_id).bind(purpose.slug());
    let version: Option<i32> = query.fetch_optional(pool).await?;
    Ok(version == Some(purpose.current_version()))
}

pub async fn withdraw(pool: &PgPool, user_id: Uuid, purpose: Purpose) -> Result<(), AppError> {
    sqlx::query(WITHDRAW).bind(user_id).bind(purpose.slug()).execute(pool).await?;
    Ok(())
}
