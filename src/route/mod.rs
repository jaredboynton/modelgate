pub mod audio;
pub mod chat;
pub mod config;
pub mod dispatch;
pub mod google;
pub mod health;
pub mod images;
pub mod internal;
pub mod messages;
pub mod models;
pub mod responses;
pub mod responses_compaction;
pub mod responses_executor;
pub mod websocket;

use axum::{
    http::{Method, StatusCode, Uri},
    Json,
};
use serde_json::json;

use crate::error::openai_error_body;

pub async fn not_found(method: Method, uri: Uri) -> (StatusCode, Json<serde_json::Value>) {
    tracing::warn!(%method, %uri, "unmatched route");
    (
        StatusCode::NOT_FOUND,
        Json(json!({
            "error": {
                "type": "not_found",
                "message": format!("route not found: {method} {uri}"),
            }
        })),
    )
}

pub async fn unsupported_public_openai_route() -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(openai_error_body(
            "public OpenAI route is not implemented for the Codex OAuth facade",
            "invalid_request_error",
            None,
            Some("unsupported_route"),
        )),
    )
}
