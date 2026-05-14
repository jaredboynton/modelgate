use axum::{http::StatusCode, Json};
use serde_json::{json, Value};

pub async fn unsupported_generation() -> (StatusCode, Json<Value>) {
    unsupported("gpt-image-2 not supported by ump-v2 v0.1; use the default Gemini painter path")
}

pub async fn unsupported_edit() -> (StatusCode, Json<Value>) {
    unsupported("gpt-image-2 image edits are not implemented in ump-v2 v0.1")
}

fn unsupported(message: &str) -> (StatusCode, Json<Value>) {
    (
        StatusCode::BAD_REQUEST,
        Json(json!({
            "error": {
                "type": "model_not_supported",
                "message": message,
            }
        })),
    )
}
