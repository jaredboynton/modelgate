use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

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
            Self::Upstream(_) | Self::Io(_) | Self::Json(_) => StatusCode::BAD_GATEWAY,
        }
    }

    pub fn error_type(&self) -> &'static str {
        match self {
            Self::MissingCredential(_) => "missing_credential",
            Self::ModelNotSupported(_) => "model_not_supported",
            Self::BadRequest(_) => "invalid_request",
            Self::NotFound(_) => "not_found",
            Self::Upstream(_) => "upstream_error",
            Self::Io(_) | Self::Json(_) => "proxy_error",
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status();
        let body = json!({
            "error": {
                "type": self.error_type(),
                "message": self.to_string(),
            }
        });
        (status, Json(body)).into_response()
    }
}
