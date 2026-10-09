//! One error shape for every route: `{"error": {"code", "message", ...}}`.
//!
//! Messages never echo a request body, so nothing a cook sent ends up in
//! an error log.

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
    #[error("this change removes restrictions; send allowWeakerRestrictions to confirm")]
    WeakensRestrictions { removed: Vec<String> },
    #[error("the database failed")]
    Database(#[from] sqlx::Error),
    #[error("stored data could not be read")]
    Corrupt(#[from] serde_json::Error),
}

impl AppError {
    /// The code the app acts on. Codes are permanent once released.
    fn code(&self) -> &'static str {
        match self {
            AppError::Auth(AuthError::KeysUnavailable) => "auth_unavailable",
            AppError::Auth(AuthError::Anonymous) => "anonymous_not_allowed",
            AppError::Auth(_) => "unauthenticated",
            AppError::AccountDeleted => "account_deleted",
            AppError::Invalid(_) => "invalid_request",
            AppError::VersionConflict { .. } => "version_conflict",
            AppError::WeakensRestrictions { .. } => "weakens_restrictions",
            AppError::Database(_) | AppError::Corrupt(_) => "internal",
        }
    }

    fn status(&self) -> StatusCode {
        match self {
            AppError::Auth(AuthError::KeysUnavailable) => StatusCode::SERVICE_UNAVAILABLE,
            AppError::Auth(AuthError::Anonymous) => StatusCode::FORBIDDEN,
            AppError::Auth(_) => StatusCode::UNAUTHORIZED,
            AppError::AccountDeleted => StatusCode::FORBIDDEN,
            AppError::Invalid(_) => StatusCode::UNPROCESSABLE_ENTITY,
            AppError::VersionConflict { .. } => StatusCode::CONFLICT,
            AppError::WeakensRestrictions { .. } => StatusCode::CONFLICT,
            AppError::Database(_) | AppError::Corrupt(_) => StatusCode::INTERNAL_SERVER_ERROR,
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

    fn body(&self) -> Value {
        let mut error = json!({ "code": self.code(), "message": self.to_string() });
        if let Some(details) = self.details() {
            error["details"] = details;
        }
        json!({ "error": error })
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status();
        if status.is_server_error() {
            self.log();
        }
        (status, Json(self.body())).into_response()
    }
}

impl AppError {
    /// The error text is ours, never the cook's data.
    #[expect(
        clippy::cognitive_complexity,
        reason = "tracing's macro expands into branches no reader sees"
    )]
    fn log(&self) {
        tracing::error!(error = %self, code = self.code(), "request failed");
    }
}
