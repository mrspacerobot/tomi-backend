//! Database access. Queries are plain SQL with every table named in full,
//! so they work whatever the connection's search path is.

use crate::error::AppError;
use crate::profile::{Profile, PROFILE_SCHEMA_VERSION};
use sqlx::postgres::PgPoolOptions;
use sqlx::{Executor, PgPool};
use uuid::Uuid;

pub static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations");

pub async fn connect(url: &str, max_connections: u32) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(max_connections)
        // Keeps sqlx's own migration table out of the public schema, which
        // Supabase's Data API exposes.
        .after_connect(|conn, _meta| {
            Box::pin(async move {
                conn.execute("set search_path = tomi, public").await?;
                Ok(())
            })
        })
        .connect(url)
        .await
}

pub async fn migrate(pool: &PgPool) -> Result<(), sqlx::Error> {
    pool.execute("create schema if not exists tomi").await?;
    MIGRATOR.run(pool).await?;
    Ok(())
}

/// Finds or creates the Tomi user for a Supabase user, and returns our own id.
pub async fn user_for_subject(pool: &PgPool, subject: Uuid) -> Result<Uuid, AppError> {
    // The no-op update makes RETURNING give back the existing row too.
    let (id, deleted): (Uuid, bool) = sqlx::query_as(
        "insert into tomi.users (auth_subject) values ($1)
         on conflict (auth_subject) do update set auth_subject = excluded.auth_subject
         returning id, deleted_at is not null",
    )
    .bind(subject)
    .fetch_one(pool)
    .await?;
    if deleted {
        return Err(AppError::AccountDeleted);
    }
    Ok(id)
}

pub struct StoredProfile {
    pub version: i64,
    pub profile: Profile,
}

/// Version 0 means the cook has never saved a profile.
pub async fn read_profile(pool: &PgPool, user_id: Uuid) -> Result<StoredProfile, AppError> {
    let row: Option<(i64, serde_json::Value)> =
        sqlx::query_as("select version, data from tomi.profiles where user_id = $1")
            .bind(user_id)
            .fetch_optional(pool)
            .await?;
    stored(row)
}

/// Saves `profile` as the version after `expected`, only if the stored
/// version is still `expected`. Returns false when someone wrote first.
pub async fn write_profile(
    pool: &PgPool,
    user_id: Uuid,
    expected: i64,
    profile: &Profile,
) -> Result<bool, AppError> {
    let data = serde_json::to_value(profile)?;
    let query = if expected == 0 {
        sqlx::query(
            "insert into tomi.profiles (user_id, version, schema_version, data)
             values ($1, $2 + 1, $3, $4)
             on conflict (user_id) do nothing",
        )
    } else {
        sqlx::query(
            "update tomi.profiles
             set version = $2 + 1, schema_version = $3, data = $4, updated_at = now()
             where user_id = $1 and version = $2",
        )
    };
    let result = query
        .bind(user_id)
        .bind(expected)
        .bind(PROFILE_SCHEMA_VERSION)
        .bind(data)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() == 1)
}

fn stored(row: Option<(i64, serde_json::Value)>) -> Result<StoredProfile, AppError> {
    match row {
        None => Ok(StoredProfile {
            version: 0,
            profile: Profile::default(),
        }),
        Some((version, data)) => Ok(StoredProfile {
            version,
            profile: serde_json::from_value(data)?,
        }),
    }
}
