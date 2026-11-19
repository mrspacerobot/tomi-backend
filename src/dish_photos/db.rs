//! Reads approved and published dish photos for a set of suggestion recipes.

use sqlx::PgPool;

#[derive(sqlx::FromRow)]
pub struct PublishedPhoto {
    pub recipe_id: String,
    pub recipe_fingerprint: String,
    pub object_name: String,
    pub shows: Vec<String>,
}

pub async fn published(pool: &PgPool, recipe_ids: &[String]) -> Vec<PublishedPhoto> {
    if recipe_ids.is_empty() {
        return Vec::new();
    }
    let query = "select recipe_id, recipe_fingerprint, object_name, shows from tomi.dish_photos
                 where recipe_id = any($1) and status = 'approved' and published_at is not null";
    match sqlx::query_as(query).bind(recipe_ids).fetch_all(pool).await {
        Ok(photos) => photos,
        Err(_) => {
            log_unavailable();
            Vec::new()
        }
    }
}

#[expect(clippy::cognitive_complexity, reason = "tracing's macro expands into branches no reader sees")]
fn log_unavailable() {
    tracing::warn!("published dish photos unavailable");
}
