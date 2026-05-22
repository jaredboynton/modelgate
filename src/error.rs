use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::{json, Value};

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("missing credential: {0}")]
    MissingCredential(&'static str),
    #[error("model not supported: {0}")]
    ModelNotSupported(String),
    #[error("bad request: {0}")]
    BadRequest(String),
    #[error("{message}")]
    BadRequestCode { code: &'static str, message: String },
    #[error("not found: {0}")]
    NotFound(String),
    #[error("upstream error: {0}")]
    Upstream(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("{0}")]
    Compaction(#[from] CompactionHttpError),
    #[error("rate limited: {message}")]
    TooManyRequests {
        message: String,
        retry_after_secs: Option<u64>,
    },
}

#[derive(Debug, Clone, thiserror::Error)]
#[error("{message}")]
pub struct CompactionHttpError {
    status: StatusCode,
    code: &'static str,
    error_type: &'static str,
    message: String,
}

impl CompactionHttpError {
    pub fn new(
        status: StatusCode,
        code: &'static str,
        error_type: &'static str,
        message: impl Into<String>,
    ) -> Self {
        Self {
            status,
            code,
            error_type,
            message: message.into(),
        }
    }

    pub fn unsupported_item_for_target(target: &crate::model_alias::ResolvedTarget) -> Self {
        Self::new(
            StatusCode::BAD_REQUEST,
            "unsupported_compaction_item_for_target",
            "invalid_request",
            format!(
                "provider-native compaction item is not supported for {:?} {:?}; switch back to a compatible target or use local fallback",
                target.provider, target.target_format
            ),
        )
    }

    pub fn invalid_pack(message: impl Into<String>) -> Self {
        Self::new(
            StatusCode::BAD_REQUEST,
            "invalid_ump_compaction_pack",
            "invalid_request",
            message,
        )
    }

    pub fn unsupported_schema() -> Self {
        Self::new(
            StatusCode::BAD_REQUEST,
            "unsupported_ump_compaction_schema",
            "invalid_request",
            "unsupported UMP compaction pack schema",
        )
    }

    pub fn status(&self) -> StatusCode {
        self.status
    }

    pub fn error_type(&self) -> &'static str {
        self.error_type
    }

    pub fn code(&self) -> &'static str {
        self.code
    }
}

impl AppError {
    pub fn status(&self) -> StatusCode {
        match self {
            Self::MissingCredential(_) => StatusCode::UNAUTHORIZED,
            Self::ModelNotSupported(_) | Self::BadRequest(_) | Self::BadRequestCode { .. } => {
                StatusCode::BAD_REQUEST
            }
            Self::NotFound(_) => StatusCode::NOT_FOUND,
            Self::Upstream(message) if is_upstream_forbidden(message) => StatusCode::FORBIDDEN,
            Self::Upstream(_) | Self::Io(_) | Self::Json(_) => StatusCode::BAD_GATEWAY,
            Self::Compaction(error) => error.status(),
            Self::TooManyRequests { .. } => StatusCode::TOO_MANY_REQUESTS,
        }
    }

    pub fn error_type(&self) -> &'static str {
        match self {
            Self::MissingCredential(message) if is_codex_auth_missing(message) => {
                "authentication_error"
            }
            Self::MissingCredential(_) => "missing_credential",
            Self::ModelNotSupported(_) => "invalid_request_error",
            Self::BadRequest(message) if is_codex_unsupported_request(message) => {
                "invalid_request_error"
            }
            Self::BadRequest(_) | Self::BadRequestCode { .. } => "invalid_request",
            Self::NotFound(_) => "not_found",
            Self::Upstream(message) if is_upstream_forbidden(message) => "permission_error",
            Self::Upstream(_) => "upstream_error",
            Self::Io(_) | Self::Json(_) => "proxy_error",
            Self::Compaction(error) => error.error_type(),
            Self::TooManyRequests { .. } => "rate_limit_exceeded",
        }
    }

    pub fn code(&self) -> Option<&'static str> {
        match self {
            Self::MissingCredential(message) if is_codex_auth_missing(message) => {
                Some("invalid_api_key")
            }
            Self::ModelNotSupported(_) => Some("model_not_supported"),
            Self::BadRequest(message) if is_codex_unsupported_request(message) => {
                Some("unsupported_feature")
            }
            Self::BadRequestCode { code, .. } => Some(code),
            Self::Upstream(message) if is_upstream_forbidden(message) => Some("upstream_forbidden"),
            Self::Compaction(error) => Some(error.code()),
            _ => None,
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status();
        let body = openai_error_body(self.to_string(), self.error_type(), None, self.code());
        (status, Json(body)).into_response()
    }
}

pub fn openai_error_body(
    message: impl Into<String>,
    error_type: &'static str,
    param: Option<&str>,
    code: Option<&str>,
) -> Value {
    json!({
        "error": {
            "message": message.into(),
            "type": error_type,
            "param": param.map_or(Value::Null, |param| json!(param)),
            "code": code.map_or(Value::Null, |code| json!(code)),
        }
    })
}

fn is_codex_unsupported_request(message: &str) -> bool {
    message.contains("not supported for Codex responses")
}

fn is_upstream_forbidden(message: &str) -> bool {
    message.contains("403") || message.contains("Forbidden") || message.contains("forbidden")
}

fn is_codex_auth_missing(message: &str) -> bool {
    message.contains("codex") || message.contains("Codex") || message.contains(".codex")
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::to_bytes, response::IntoResponse};
    use serde_json::Value;

    async fn response_body(error: AppError) -> (StatusCode, Value) {
        let response = error.into_response();
        let status = response.status();
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        (status, serde_json::from_slice(&body).unwrap())
    }

    #[tokio::test]
    async fn missing_credential_returns_openai_authentication_error() {
        let (status, body) = response_body(AppError::MissingCredential("codex access token")).await;

        assert_eq!(status, StatusCode::UNAUTHORIZED);
        assert_eq!(body["error"]["type"], "authentication_error");
        assert_eq!(body["error"]["code"], "invalid_api_key");
        assert!(body["error"]["param"].is_null());
    }

    #[tokio::test]
    async fn unsupported_model_returns_openai_invalid_request_error() {
        let (status, body) = response_body(AppError::ModelNotSupported("gpt-x".into())).await;

        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(body["error"]["type"], "invalid_request_error");
        assert_eq!(body["error"]["code"], "model_not_supported");
        assert!(body["error"]["param"].is_null());
    }

    #[tokio::test]
    async fn upstream_forbidden_returns_permission_error() {
        let (status, body) = response_body(AppError::Upstream(
            "Codex HTTP returned 403 Forbidden".into(),
        ))
        .await;

        assert_eq!(status, StatusCode::FORBIDDEN);
        assert_eq!(body["error"]["type"], "permission_error");
        assert_eq!(body["error"]["code"], "upstream_forbidden");
        assert!(body["error"]["param"].is_null());
    }

    #[tokio::test]
    async fn compaction_error_returns_embedded_shape() {
        let (status, body) = response_body(AppError::Compaction(CompactionHttpError::new(
            StatusCode::PAYLOAD_TOO_LARGE,
            "compaction_pack_too_large",
            "invalid_request",
            "too large",
        )))
        .await;

        assert_eq!(status, StatusCode::PAYLOAD_TOO_LARGE);
        assert_eq!(body["error"]["type"], "invalid_request");
        assert_eq!(body["error"]["code"], "compaction_pack_too_large");
        assert_eq!(body["error"]["message"], "too large");
    }
}
