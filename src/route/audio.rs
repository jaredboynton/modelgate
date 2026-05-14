use axum::{http::StatusCode, Json};
use serde_json::Value;

use crate::error::openai_error_body;

pub async fn realtime_transcription_sessions() -> (StatusCode, Json<Value>) {
    feature_gated(
        "realtime transcription sessions are feature-gated in ump-v2; the adapter will not proxy Codex bearer realtime auth until the realtime bridge is implemented",
    )
}

pub async fn audio_speech() -> (StatusCode, Json<Value>) {
    feature_gated(
        "audio speech is not available through Codex bearer in ump-v2; live probes showed missing public TTS scope",
    )
}

pub async fn transcribe() -> (StatusCode, Json<Value>) {
    feature_gated(
        "dictation transcription is feature-gated in ump-v2; use the dedicated transcription bridge once implemented",
    )
}

fn feature_gated(message: &str) -> (StatusCode, Json<Value>) {
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(openai_error_body(
            message,
            "invalid_request_error",
            None,
            Some("unsupported_feature"),
        )),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn audio_feature_gates_return_unsupported_feature_contract() {
        let (status, Json(body)) = audio_speech().await;

        assert_eq!(status, StatusCode::NOT_IMPLEMENTED);
        assert_eq!(body["error"]["type"], "invalid_request_error");
        assert_eq!(body["error"]["code"], "unsupported_feature");
        assert!(body["error"]["param"].is_null());
    }
}
