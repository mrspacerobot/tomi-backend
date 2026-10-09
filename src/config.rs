//! Settings from the environment. Secrets come only from here, never from
//! files in the repository.

use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub port: u16,
    pub database_url: String,
    pub db_max_connections: u32,
    pub run_migrations: bool,
    pub supabase_url: String,
    /// Only for projects still on the legacy shared secret, and for tests.
    pub supabase_jwt_secret: Option<String>,
    pub min_app_version: String,
}

impl Config {
    pub fn from_env() -> Result<Self, String> {
        Ok(Config {
            port: optional("PORT", "8080")
                .parse()
                .map_err(|_| "PORT must be a number")?,
            database_url: required("DATABASE_URL")?,
            db_max_connections: optional("DB_MAX_CONNECTIONS", "5")
                .parse()
                .map_err(|_| "DB_MAX_CONNECTIONS must be a number")?,
            run_migrations: optional("RUN_MIGRATIONS", "true") == "true",
            supabase_url: required("SUPABASE_URL")?,
            supabase_jwt_secret: env::var("SUPABASE_JWT_SECRET")
                .ok()
                .filter(|value| !value.is_empty()),
            min_app_version: optional("MIN_APP_VERSION", "1.0.0"),
        })
    }
}

fn required(name: &str) -> Result<String, String> {
    env::var(name)
        .ok()
        .filter(|value| !value.is_empty())
        .ok_or_else(|| format!("{name} is not set"))
}

fn optional(name: &str, default: &str) -> String {
    env::var(name)
        .ok()
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| default.to_string())
}
