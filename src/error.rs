//! One error shape for every route: `{"error": {"code", "message", ...}}`.
//!
//! Messages never echo a request body, so no ingredient or restriction a
//! cook sent ends up in an error or a log.

use crate::auth::AuthError;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::{json, Value};

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error(transparent)]
    Auth(#[from] AuthError),
    #[error("this account has been deleted")]
    AccountDeleted,
    #[error("the request is not valid")]
    Invalid(Vec<String>),
    #[error("the profile changed since version {expected}")]
    VersionConflict { expected: i64, current: Value },
    #[error("this change removes restrictions; send allow_weaker_restrictions to confirm")]
    WeakensRestrictions { removed: Vec<String> },
    #[error("the database failed")]
    Database(#[from] sqlx::Error),
    #[error("stored data could not be read")]
    Corrupt(#[from] serde_json::Error),
}

impl AppError {
    fn status_and_code(&self) -> (StatusCode, &'static str) {
        match self {
            AppError::Auth(AuthError::KeysUnavailable) => {
                (StatusCode::SERVICE_UNAVAILABLE, "auth_unavailable")
            }
            AppError::Auth(AuthError::Anonymous) => {
                (StatusCode::FORBIDDEN, "anonymous_not_allowed")
            }
            AppError::Auth(_) => (StatusCode::UNAUTHORIZED, "unauthenticated"),
            AppError::AccountDeleted => (StatusCode::FORBIDDEN, "account_deleted"),
            AppError::Invalid(_) => (StatusCode::UNPROCESSABLE_ENTITY, "invalid_request"),
            AppError::VersionConflict { .. } => (StatusCode::CONFLICT, "version_conflict"),
            AppError::WeakensRestrictions { .. } => (StatusCode::CONFLICT, "weakens_restrictions"),
            AppError::Database(_) | AppError::Corrupt(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "internal")
            }
        }
    }

    fn details(&self) -> Option<Value> {
        match self {
            AppError::Invalid(problems) => Some(json!({ "problems": problems })),
            AppError::VersionConflict { current, .. } => Some(json!({ "current": current })),
            AppError::WeakensRestrictions { removed } => Some(json!({ "removed": removed })),
            _ => None,
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code) = self.status_and_code();
        if status.is_server_error() {
            // The error text is ours, never the cook's data.
            tracing::error!(error = %self, code, "request failed");
        }
        let mut error = json!({ "code": code, "message": self.to_string() });
        if let Some(details) = self.details() {
            error["details"] = details;
        }
        (status, Json(json!({ "error": error }))).into_response()
    }
}
