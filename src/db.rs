//! The database: connecting and migrating.
//!
//! The queries live with their feature, in `<feature>/db.rs`. They are plain SQL
//! with every table named in full, so they work whatever the connection's search
//! path is.

use sqlx::postgres::PgPoolOptions;
use sqlx::{Executor, PgPool};

pub static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations");

pub async fn connect(url: &str, max_connections: u32) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(max_connections)
        // Keeps sqlx's own migration table out of the public schema, which
        // Supabase's Data API exposes.
        .after_connect(|connection, _meta| {
            Box::pin(async move {
                connection.execute("set search_path = tomi, public").await?;
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
