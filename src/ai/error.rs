//! Safe AI failure categories for operational logs, without provider bodies or credentials.

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum AiError {
    #[error("no AI model is configured")]
    NotConfigured,
    #[error("the model request timed out")]
    TimedOut,
    #[error("the model could not be reached")]
    Unreachable,
    #[error("the provider rejected the model request with HTTP {status}")]
    Rejected { status: u16 },
    #[error("the provider response was not readable JSON")]
    Unreadable,
    #[error("the model returned invalid structured content")]
    InvalidContent,
    #[error("the model returned no text")]
    EmptyResponse,
    #[error("the provider blocked the model response")]
    Blocked,
    #[error("the model exhausted its output token limit")]
    OutputLimit,
}

impl AiError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::NotConfigured => "ai_not_configured",
            Self::TimedOut => "ai_timeout",
            Self::Unreachable => "ai_transport_error",
            Self::Rejected { status } => rejection_code(*status),
            Self::Unreadable => "ai_invalid_response",
            Self::InvalidContent => "ai_invalid_content",
            Self::EmptyResponse => "ai_empty_response",
            Self::Blocked => "ai_response_blocked",
            Self::OutputLimit => "ai_output_limit",
        }
    }

    pub fn provider_status(&self) -> Option<u16> {
        match self {
            Self::Rejected { status } => Some(*status),
            _ => None,
        }
    }

    pub(super) fn transport(error: reqwest::Error) -> Self {
        if error.is_timeout() {
            return Self::TimedOut;
        }
        match error.status() {
            Some(status) => Self::Rejected { status: status.as_u16() },
            None => Self::Unreachable,
        }
    }

    pub(super) fn response(error: reqwest::Error) -> Self {
        if error.is_decode() {
            return Self::Unreadable;
        }
        Self::transport(error)
    }

    #[expect(clippy::cognitive_complexity, reason = "tracing's macro expands into branches no reader sees")]
    pub(crate) fn log(&self, model: &str, operation: &str) {
        tracing::warn!(
            model,
            operation,
            error_code = self.code(),
            provider_http_status = self.provider_status(),
            "AI request failed"
        );
    }
}

fn rejection_code(status: u16) -> &'static str {
    match status {
        400 | 422 => "ai_provider_invalid_request",
        401 | 403 => "ai_provider_auth_error",
        404 => "ai_provider_model_not_found",
        429 => "ai_provider_rate_limited",
        500..=599 => "ai_provider_unavailable",
        _ => "ai_provider_rejected",
    }
}
