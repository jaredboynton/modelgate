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
    #[error("not found: {0}")]
    NotFound(String),
    #[error("upstream error: {0}")]
    Upstream(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
}

impl AppError {
    pub fn status(&self) -> StatusCode {
        match self {
            Self::MissingCredential(_) => StatusCode::UNAUTHORIZED,
            Self::ModelNotSupported(_) | Self::BadRequest(_) => StatusCode::BAD_REQUEST,
            Self::NotFound(_) => StatusCode::NOT_FOUND,
            Self::Upstream(message) if is_upstream_forbidden(message) => StatusCode::FORBIDDEN,
            Self::Upstream(_) | Self::Io(_) | Self::Json(_) => StatusCode::BAD_GATEWAY,
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
            Self::BadRequest(_) => "invalid_request",
            Self::NotFound(_) => "not_found",
            Self::Upstream(message) if is_upstream_forbidden(message) => "permission_error",
            Self::Upstream(_) => "upstream_error",
            Self::Io(_) | Self::Json(_) => "proxy_error",
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
            Self::Upstream(message) if is_upstream_forbidden(message) => Some("upstream_forbidden"),
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
}
