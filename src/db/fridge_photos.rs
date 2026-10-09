//! Records of fridge photos kept for the dataset.

use crate::domain::fridge::SeenItem;
use crate::domain::ids::SavedId;
use crate::error::AppError;
use sqlx::PgPool;
use uuid::Uuid;

const INSERT: &str = "
    insert into tomi.fridge_photos (id, user_id, object_path, mime_type, consent_version, model, seen)
    values ($1, $2, $3, $4, $5, $6, $7)";

const CONFIRM: &str = "update tomi.fridge_photos set confirmed = $3 where id = $1 and user_id = $2";

const PATHS: &str = "select object_path from tomi.fridge_photos where user_id = $1";

const DELETE_ALL: &str = "delete from tomi.fridge_photos where user_id = $1";

/// A kept photo, as recorded next to the stored file.
pub struct KeptPhoto<'a> {
    pub id: Uuid,
    pub user_id: Uuid,
    pub object_path: &'a str,
    pub mime_type: &'a str,
    pub consent_version: i32,
    pub model: &'a str,
    pub seen: &'a [SeenItem],
}

pub async fn insert(pool: &PgPool, photo: &KeptPhoto<'_>) -> Result<(), AppError> {
    let query = sqlx::query(INSERT).bind(photo.id).bind(photo.user_id).bind(photo.object_path);
    let query = query.bind(photo.mime_type).bind(photo.consent_version).bind(photo.model);
    query.bind(serde_json::to_value(photo.seen)?).execute(pool).await?;
    Ok(())
}

/// What the cook said they have, after one of their scans.
pub struct Confirmation<'a> {
    pub user_id: Uuid,
    pub scan_id: Uuid,
    pub ingredients: &'a [SavedId],
}

/// Records the confirmation with the kept photo. Does nothing for a scan
/// that was not kept or belongs to someone else.
pub async fn confirm(pool: &PgPool, confirmation: &Confirmation<'_>) -> Result<(), AppError> {
    let query = sqlx::query(CONFIRM).bind(confirmation.scan_id).bind(confirmation.user_id);
    query.bind(serde_json::to_value(confirmation.ingredients)?).execute(pool).await?;
    Ok(())
}

pub async fn paths(pool: &PgPool, user_id: Uuid) -> Result<Vec<String>, AppError> {
    let paths = sqlx::query_scalar(PATHS).bind(user_id).fetch_all(pool).await?;
    Ok(paths)
}

pub async fn delete_all(pool: &PgPool, user_id: Uuid) -> Result<(), AppError> {
    sqlx::query(DELETE_ALL).bind(user_id).execute(pool).await?;
    Ok(())
}
