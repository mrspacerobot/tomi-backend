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
    /// Only for projects still on the legacy shared secret.
    pub supabase_jwt_secret: Option<String>,
    pub min_app_version: String,
}

impl Config {
    pub fn from_env() -> Result<Self, String> {
        Ok(Config {
            port: number("PORT", 8080)?,
            database_url: required("DATABASE_URL")?,
            db_max_connections: number("DB_MAX_CONNECTIONS", 5)?,
            run_migrations: optional("RUN_MIGRATIONS", "true") == "true",
            supabase_url: required("SUPABASE_URL")?,
            supabase_jwt_secret: read("SUPABASE_JWT_SECRET"),
            min_app_version: optional("MIN_APP_VERSION", "1.0.0"),
        })
    }
}

/// The value of a variable, treating an empty one as unset.
fn read(name: &str) -> Option<String> {
    match env::var(name) {
        Ok(value) if !value.is_empty() => Some(value),
        _ => None,
    }
}

fn required(name: &str) -> Result<String, String> {
    match read(name) {
        Some(value) => Ok(value),
        None => Err(format!("{name} is not set")),
    }
}

fn optional(name: &str, default: &str) -> String {
    match read(name) {
        Some(value) => value,
        None => default.to_string(),
    }
}

fn number<T: std::str::FromStr>(name: &str, default: T) -> Result<T, String> {
    let Some(value) = read(name) else {
        return Ok(default);
    };
    match value.parse() {
        Ok(number) => Ok(number),
        Err(_) => Err(format!("{name} must be a number")),
    }
}
