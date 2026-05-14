use axum::{http::StatusCode, Json};
use serde_json::Value;

use crate::error::openai_error_body;

pub async fn unsupported_generation() -> (StatusCode, Json<Value>) {
    unsupported("gpt-image-2 not supported by ump-v2 v0.1; use the default Gemini painter path")
}

pub async fn unsupported_edit() -> (StatusCode, Json<Value>) {
    unsupported("gpt-image-2 image edits are not implemented in ump-v2 v0.1")
}

fn unsupported(message: &str) -> (StatusCode, Json<Value>) {
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(openai_error_body(
            message,
            "invalid_request_error",
            None,
            Some("unsupported_route"),
        )),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn image_routes_return_unsupported_route_contract() {
        let (status, Json(body)) = unsupported_generation().await;

        assert_eq!(status, StatusCode::NOT_IMPLEMENTED);
        assert_eq!(body["error"]["type"], "invalid_request_error");
        assert_eq!(body["error"]["code"], "unsupported_route");
        assert!(body["error"]["param"].is_null());
    }
}
