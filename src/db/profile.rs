//! Reading and saving the cook's profile, with a version for each save.

use crate::domain::profile::{from_stored, Profile, PROFILE_SCHEMA_VERSION};
use crate::error::AppError;
use sqlx::PgPool;
use uuid::Uuid;

const READ: &str = "select version, schema_version, data from tomi.profiles where user_id = $1";

const INSERT_FIRST: &str = "
    insert into tomi.profiles (user_id, version, schema_version, data)
    values ($1, $2 + 1, $3, $4)
    on conflict (user_id) do nothing";

const UPDATE_AT_VERSION: &str = "
    update tomi.profiles
    set version = $2 + 1, schema_version = $3, data = $4, updated_at = now()
    where user_id = $1 and version = $2";

pub struct StoredProfile {
    /// 0 means the cook has never saved a profile.
    pub version: i64,
    pub profile: Profile,
}

pub async fn read(pool: &PgPool, user_id: Uuid) -> Result<StoredProfile, AppError> {
    let row: Option<(i64, i32, serde_json::Value)> = sqlx::query_as(READ).bind(user_id).fetch_optional(pool).await?;
    let Some((version, schema_version, data)) = row else {
        return Ok(StoredProfile { version: 0, profile: Profile::default() });
    };
    let profile = from_stored(schema_version, data)?;
    Ok(StoredProfile { version, profile })
}

/// A profile to save as the version after `expected_version`.
pub struct ProfileWrite {
    pub user_id: Uuid,
    pub expected_version: i64,
    pub profile: Profile,
}

/// Saves only if the stored version is still the expected one.
/// Returns false when someone else wrote first.
pub async fn write(pool: &PgPool, write: &ProfileWrite) -> Result<bool, AppError> {
    let query = match write.expected_version {
        0 => INSERT_FIRST,
        _ => UPDATE_AT_VERSION,
    };
    let result = sqlx::query(query)
        .bind(write.user_id)
        .bind(write.expected_version)
        .bind(PROFILE_SCHEMA_VERSION)
        .bind(serde_json::to_value(&write.profile)?)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() == 1)
}
