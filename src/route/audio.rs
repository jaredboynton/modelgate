use axum::{
    body::{to_bytes, Body, Bytes},
    extract::State,
    http::{header, HeaderMap, StatusCode, Uri},
    response::{IntoResponse, Response},
    Json,
};
use serde_json::Value;

use crate::{
    error::openai_error_body,
    upstream::{openai_audio, openai_public, openai_realtime},
    AppError, AppState, UpstreamResponse,
};

const REALTIME_TRANSCRIPTION_SESSION_MAX_BYTES: usize = 64 * 1024;
const AUDIO_TRANSCRIPTION_BODY_LIMIT_BYTES: usize = 25 * 1024 * 1024;
const UNSUPPORTED_FEATURE_MESSAGE: &str =
    "This route family is not supported by the Codex OAuth public OpenAI facade.";
const UNSUPPORTED_ROUTE_MESSAGE: &str =
    "This OpenAI route is not implemented for the Codex OAuth public OpenAI facade.";
const MISSING_CODEX_AUTH_MESSAGE: &str = "Missing Codex OAuth credentials at ~/.codex/auth.json.";

type RouteResponseResult<T> = Result<T, Box<Response>>;

pub async fn realtime_transcription_sessions(
    State(state): State<AppState>,
    uri: Uri,
    headers: HeaderMap,
    body: Body,
) -> Response {
    if !is_realtime_transcription_sessions_path(uri.path()) {
        return unsupported_feature().into_response();
    }

    match realtime_transcription_sessions_impl(&state, &headers, body).await {
        Ok(response) => response.into_response(),
        Err(response) => *response,
    }
}

pub async fn realtime_client_secrets() -> Response {
    unsupported_feature().into_response()
}

pub async fn realtime_calls() -> Response {
    unsupported_feature().into_response()
}

pub async fn realtime_unsupported_descendant() -> Response {
    unsupported_route().into_response()
}

pub async fn audio_speech() -> (StatusCode, Json<Value>) {
    unsupported_feature()
}

pub async fn audio_translations() -> (StatusCode, Json<Value>) {
    unsupported_feature()
}

pub async fn audio_unsupported_descendant() -> (StatusCode, Json<Value>) {
    unsupported_route()
}

pub async fn transcribe(
    State(state): State<AppState>,
    uri: Uri,
    headers: HeaderMap,
    body: Body,
) -> Response {
    if !is_audio_transcriptions_path(uri.path()) {
        return unsupported_feature().into_response();
    }

    match audio_transcriptions_impl(&state, &headers, body).await {
        Ok(response) => response.into_response(),
        Err(response) => *response,
    }
}

async fn realtime_transcription_sessions_impl(
    state: &AppState,
    headers: &HeaderMap,
    body: Body,
) -> RouteResponseResult<UpstreamResponse> {
    ensure_codex_auth_present(state)?;
    require_json_content_type(headers)?;
    let body = limited_body(
        body,
        REALTIME_TRANSCRIPTION_SESSION_MAX_BYTES,
        "realtime_body_too_large",
    )
    .await?;
    serde_json::from_slice::<Value>(&body).map_err(|_| {
        Box::new(openai_error_response(
            StatusCode::BAD_REQUEST,
            "Request body must be valid JSON.",
            "invalid_request_error",
            Some("invalid_json"),
        ))
    })?;

    openai_realtime::forward_realtime_transcription_session(state, headers.clone(), body)
        .await
        .map_err(app_error_response)
}

async fn audio_transcriptions_impl(
    state: &AppState,
    headers: &HeaderMap,
    body: Body,
) -> RouteResponseResult<UpstreamResponse> {
    require_multipart_content_type(headers)?;
    ensure_codex_auth_present(state)?;
    let body = limited_body(
        body,
        AUDIO_TRANSCRIPTION_BODY_LIMIT_BYTES,
        "audio_body_too_large",
    )
    .await?;

    openai_audio::forward_audio_transcriptions(state, headers.clone(), body)
        .await
        .map_err(app_error_response)
}

fn ensure_codex_auth_present(state: &AppState) -> RouteResponseResult<()> {
    openai_public::require_public_openai_auth(state).map_err(app_error_response)
}

fn require_json_content_type(headers: &HeaderMap) -> RouteResponseResult<()> {
    let Some(content_type) = headers.get(header::CONTENT_TYPE) else {
        return Err(Box::new(openai_error_response(
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
            "Realtime transcription sessions require application/json.",
            "invalid_request_error",
            Some("invalid_content_type"),
        )));
    };
    let Ok(content_type) = content_type.to_str() else {
        return Err(Box::new(openai_error_response(
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
            "Realtime transcription sessions require application/json.",
            "invalid_request_error",
            Some("invalid_content_type"),
        )));
    };
    let media_type = content_type
        .split(';')
        .next()
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase();
    if media_type != "application/json" {
        return Err(Box::new(openai_error_response(
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
            "Realtime transcription sessions require application/json.",
            "invalid_request_error",
            Some("invalid_content_type"),
        )));
    }
    Ok(())
}

fn require_multipart_content_type(headers: &HeaderMap) -> RouteResponseResult<()> {
    let Some(content_type) = headers.get(header::CONTENT_TYPE) else {
        return Err(Box::new(openai_error_response(
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
            "Audio transcriptions require multipart/form-data.",
            "invalid_request_error",
            Some("invalid_content_type"),
        )));
    };
    let content_type_text = content_type.to_str().map_err(|_| {
        Box::new(openai_error_response(
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
            "Audio transcriptions require multipart/form-data.",
            "invalid_request_error",
            Some("invalid_content_type"),
        ))
    })?;
    let mut parts = content_type_text.split(';');
    let media_type = parts.next().unwrap_or_default().trim().to_ascii_lowercase();
    if media_type != "multipart/form-data" {
        return Err(Box::new(openai_error_response(
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
            "Audio transcriptions require multipart/form-data.",
            "invalid_request_error",
            Some("invalid_content_type"),
        )));
    }
    let has_boundary = parts.any(|part| {
        part.trim()
            .to_ascii_lowercase()
            .strip_prefix("boundary=")
            .is_some_and(|boundary| !boundary.trim_matches('"').trim().is_empty())
    });
    if !has_boundary {
        return Err(Box::new(openai_error_response(
            StatusCode::BAD_REQUEST,
            "Audio transcriptions require a multipart boundary.",
            "invalid_request_error",
            Some("missing_multipart_boundary"),
        )));
    }
    Ok(())
}

async fn limited_body(body: Body, limit: usize, code: &'static str) -> RouteResponseResult<Bytes> {
    to_bytes(body, limit).await.map_err(|_| {
        Box::new(openai_error_response(
            StatusCode::PAYLOAD_TOO_LARGE,
            "Request body exceeds the supported size limit.",
            "invalid_request_error",
            Some(code),
        ))
    })
}

fn app_error_response(err: AppError) -> Box<Response> {
    Box::new(match err {
        AppError::MissingCredential(message)
            if message.contains("codex")
                || message.contains("Codex")
                || message.contains(".codex") =>
        {
            openai_error_response(
                StatusCode::UNAUTHORIZED,
                MISSING_CODEX_AUTH_MESSAGE,
                "authentication_error",
                Some("invalid_api_key"),
            )
        }
        other => openai_error_response(
            other.status(),
            other.to_string(),
            other.error_type(),
            other.code(),
        ),
    })
}

fn is_realtime_transcription_sessions_path(path: &str) -> bool {
    matches!(
        path,
        "/v1/realtime/transcription_sessions"
            | "/api/provider/openai/v1/realtime/transcription_sessions"
    )
}

fn is_audio_transcriptions_path(path: &str) -> bool {
    matches!(
        path,
        "/v1/audio/transcriptions" | "/api/provider/openai/v1/audio/transcriptions"
    )
}

fn unsupported_feature() -> (StatusCode, Json<Value>) {
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(openai_error_body(
            UNSUPPORTED_FEATURE_MESSAGE,
            "invalid_request_error",
            None,
            Some("unsupported_feature"),
        )),
    )
}

fn unsupported_route() -> (StatusCode, Json<Value>) {
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(openai_error_body(
            UNSUPPORTED_ROUTE_MESSAGE,
            "invalid_request_error",
            None,
            Some("unsupported_route"),
        )),
    )
}

fn openai_error_response(
    status: StatusCode,
    message: impl Into<String>,
    error_type: &'static str,
    code: Option<&str>,
) -> Response {
    (
        status,
        Json(openai_error_body(message, error_type, None, code)),
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::to_bytes, http::HeaderValue};

    async fn response_json(response: Response) -> (StatusCode, Value) {
        let status = response.status();
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        (status, serde_json::from_slice(&body).unwrap())
    }

    #[tokio::test]
    async fn audio_feature_gates_return_exact_unsupported_feature_contract() {
        let (status, Json(body)) = audio_speech().await;

        assert_eq!(status, StatusCode::NOT_IMPLEMENTED);
        assert_eq!(body["error"]["message"], UNSUPPORTED_FEATURE_MESSAGE);
        assert_eq!(body["error"]["type"], "invalid_request_error");
        assert_eq!(body["error"]["code"], "unsupported_feature");
        assert!(body["error"]["param"].is_null());
    }

    #[tokio::test]
    async fn unsupported_descendants_return_exact_unsupported_route_contract() {
        let (status, Json(body)) = audio_unsupported_descendant().await;

        assert_eq!(status, StatusCode::NOT_IMPLEMENTED);
        assert_eq!(body["error"]["message"], UNSUPPORTED_ROUTE_MESSAGE);
        assert_eq!(body["error"]["type"], "invalid_request_error");
        assert_eq!(body["error"]["code"], "unsupported_route");
        assert!(body["error"]["param"].is_null());
    }

    #[test]
    fn realtime_transcription_sessions_path_excludes_gated_realtime_routes() {
        assert!(is_realtime_transcription_sessions_path(
            "/v1/realtime/transcription_sessions"
        ));
        assert!(is_realtime_transcription_sessions_path(
            "/api/provider/openai/v1/realtime/transcription_sessions"
        ));
        assert!(!is_realtime_transcription_sessions_path(
            "/v1/realtime/client_secrets"
        ));
        assert!(!is_realtime_transcription_sessions_path(
            "/v1/realtime/calls"
        ));
    }

    #[test]
    fn audio_transcriptions_path_is_not_legacy_transcribe() {
        assert!(is_audio_transcriptions_path("/v1/audio/transcriptions"));
        assert!(is_audio_transcriptions_path(
            "/api/provider/openai/v1/audio/transcriptions"
        ));
        assert!(!is_audio_transcriptions_path("/transcribe"));
        assert!(!is_audio_transcriptions_path("/v1/audio/translations"));
    }

    #[test]
    fn multipart_content_type_requires_boundary() {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("multipart/form-data; boundary=abc"),
        );
        assert!(require_multipart_content_type(&headers).is_ok());

        headers.insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("multipart/form-data"),
        );
        let err = require_multipart_content_type(&headers).unwrap_err();
        assert_eq!(err.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn missing_codex_auth_maps_to_exact_openai_401() {
        let response = app_error_response(AppError::MissingCredential("~/.codex/auth.json"));
        let (status, body) = response_json(*response).await;

        assert_eq!(status, StatusCode::UNAUTHORIZED);
        assert_eq!(body["error"]["message"], MISSING_CODEX_AUTH_MESSAGE);
        assert_eq!(body["error"]["type"], "authentication_error");
        assert_eq!(body["error"]["code"], "invalid_api_key");
        assert!(body["error"]["param"].is_null());
    }
}
