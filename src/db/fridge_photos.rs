//! Records of fridge photos kept for the dataset.

use crate::domain::fridge::SeenItem;
use crate::domain::ids::SavedId;
use crate::error::AppError;
use sqlx::PgPool;
use uuid::Uuid;

const INSERT: &str = "
    insert into tomi.fridge_photos (id, user_id, object_path, mime_type, model, seen)
    values ($1, $2, $3, $4, $5, $6)";

const CONFIRM: &str = "update tomi.fridge_photos set confirmed = $3 where id = $1 and user_id = $2";

/// A kept photo, as recorded next to the stored file.
pub struct KeptPhoto<'a> {
    pub id: Uuid,
    pub user_id: Uuid,
    pub object_path: &'a str,
    pub mime_type: &'a str,
    pub model: &'a str,
    pub seen: &'a [SeenItem],
}

pub async fn insert(pool: &PgPool, photo: &KeptPhoto<'_>) -> Result<(), AppError> {
    let query = sqlx::query(INSERT).bind(photo.id).bind(photo.user_id).bind(photo.object_path);
    let query = query.bind(photo.mime_type).bind(photo.model);
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
