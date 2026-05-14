use axum::{http::StatusCode, Json};
use serde_json::{json, Value};

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
        StatusCode::BAD_REQUEST,
        Json(json!({
            "error": {
                "type": "unsupported_route",
                "message": message,
            }
        })),
    )
}
