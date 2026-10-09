//! One error shape for every route: `{"error": {"code", "message", ...}}`.
//!
//! Messages never echo a request body, so nothing a cook sent ends up in
//! an error log.

use crate::auth::AuthError;
use crate::domain::consent::Purpose;
use crate::domain::photo::PhotoError;
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
    #[error("nothing is here")]
    NotFound,
    #[error("the profile changed since version {expected}")]
    VersionConflict { expected: i64, current: Value },
    #[error("this change removes restrictions; send allowWeakerRestrictions to confirm")]
    WeakensRestrictions { removed: Vec<String> },
    #[error("the cook has not agreed to the current text for {}", .0.slug())]
    ConsentRequired(Purpose),
    #[error(transparent)]
    Photo(#[from] PhotoError),
    #[error("the AI model is unavailable; the cook can type what they have")]
    AiUnavailable,
    #[error("photo storage is unavailable")]
    StorageUnavailable,
    #[error("the database failed")]
    Database(#[from] sqlx::Error),
    #[error("stored data could not be read")]
    Corrupt(#[from] serde_json::Error),
}

impl AppError {
    /// The code the app acts on. Codes are permanent once released.
    #[expect(
        clippy::too_many_lines,
        reason = "a flat match from each error to its value, the long shape AGENTS.md allows"
    )]
    fn code(&self) -> &'static str {
        match self {
            AppError::Auth(AuthError::KeysUnavailable) => "auth_unavailable",
            AppError::Auth(AuthError::Anonymous) => "anonymous_not_allowed",
            AppError::Auth(_) => "unauthenticated",
            AppError::AccountDeleted => "account_deleted",
            AppError::Invalid(_) => "invalid_request",
            AppError::NotFound => "not_found",
            AppError::VersionConflict { .. } => "version_conflict",
            AppError::WeakensRestrictions { .. } => "weakens_restrictions",
            AppError::ConsentRequired(_) => "consent_required",
            AppError::Photo(PhotoError::TooLarge) => "photo_too_large",
            AppError::Photo(_) => "photo_unreadable",
            AppError::AiUnavailable => "ai_unavailable",
            AppError::StorageUnavailable => "storage_unavailable",
            AppError::Database(_) | AppError::Corrupt(_) => "internal",
        }
    }

    #[expect(
        clippy::too_many_lines,
        reason = "a flat match from each error to its value, the long shape AGENTS.md allows"
    )]
    fn status(&self) -> StatusCode {
        match self {
            AppError::Auth(AuthError::KeysUnavailable) => StatusCode::SERVICE_UNAVAILABLE,
            AppError::Auth(AuthError::Anonymous) => StatusCode::FORBIDDEN,
            AppError::Auth(_) => StatusCode::UNAUTHORIZED,
            AppError::AccountDeleted => StatusCode::FORBIDDEN,
            AppError::Invalid(_) => StatusCode::UNPROCESSABLE_ENTITY,
            AppError::NotFound => StatusCode::NOT_FOUND,
            AppError::VersionConflict { .. } => StatusCode::CONFLICT,
            AppError::WeakensRestrictions { .. } => StatusCode::CONFLICT,
            AppError::ConsentRequired(_) => StatusCode::FORBIDDEN,
            AppError::Photo(PhotoError::TooLarge) => StatusCode::PAYLOAD_TOO_LARGE,
            AppError::Photo(PhotoError::UnsupportedType) => StatusCode::UNSUPPORTED_MEDIA_TYPE,
            AppError::Photo(_) => StatusCode::UNPROCESSABLE_ENTITY,
            AppError::AiUnavailable | AppError::StorageUnavailable => StatusCode::SERVICE_UNAVAILABLE,
            AppError::Database(_) | AppError::Corrupt(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn details(&self) -> Option<Value> {
        match self {
            AppError::Invalid(problems) => Some(json!({ "problems": problems })),
            AppError::VersionConflict { current, .. } => Some(json!({ "current": current })),
            AppError::WeakensRestrictions { removed } => Some(json!({ "removed": removed })),
            AppError::ConsentRequired(purpose) => Some(consent_details(*purpose)),
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

    /// The error text is ours, never the cook's data.
    #[expect(clippy::cognitive_complexity, reason = "tracing's macro expands into branches no reader sees")]
    fn log(&self) {
        tracing::error!(error = %self, code = self.code(), "request failed");
    }
}

fn consent_details(purpose: Purpose) -> Value {
    json!({ "purpose": purpose.slug(), "version": purpose.current_version() })
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
