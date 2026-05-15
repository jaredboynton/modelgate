use axum::{
    body::{to_bytes, Body},
    http::{header, header::HeaderName, HeaderMap, HeaderValue, Method, Request, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{any, get, post},
    Json, Router,
};
use serde_json::json;
use tower_http::trace::TraceLayer;

use crate::{
    failure_capture,
    model_alias::{resolve_model, Provider},
    request_body, route,
    upstream_response::UpstreamResponseMetadata,
    AppError, AppState,
};

use std::time::Instant;

pub const REQUEST_ID_HEADER: &str = "x-request-id";

#[derive(Clone, Debug)]
pub struct RequestId(pub String);

#[derive(Debug)]
struct RequestObservation {
    method: Method,
    path: String,
    provider: Option<&'static str>,
    model: Option<String>,
}

const OBSERVABILITY_BODY_LIMIT: usize = 64 * 1024;

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(route::health::health))
        .route("/config", get(route::config::config_page))
        .route("/config/assets/config.css", get(route::config::config_css))
        .route("/config/assets/config.js", get(route::config::config_js))
        .route(
            "/api/config",
            get(route::config::get_config).put(route::config::put_config),
        )
        .route(
            "/api/config/graph",
            get(route::config::get_config_graph).post(route::config::post_config_graph),
        )
        .route("/news.rss", get(route::internal::news_rss))
        .route("/api/internal", post(route::internal::amp_internal))
        .route(
            "/api/internal/bitbucket-instance-url",
            get(route::internal::bitbucket_instance_url),
        )
        .route(
            "/api/internal/github-auth-status",
            get(route::internal::github_auth_status),
        )
        .route(
            "/api/internal/github-proxy/*path",
            any(route::internal::github_proxy),
        )
        .route("/api/telemetry", post(route::internal::telemetry))
        .route("/transcribe", post(route::audio::transcribe))
        .route("/api/attachments", post(route::internal::attachment_post))
        .route("/api/attachments/:id", get(route::internal::attachment_get))
        .route("/api/threads/find", get(route::internal::threads_find))
        .route("/api/threads/:file", get(route::internal::thread_markdown))
        .route("/v1/models", get(route::models::models))
        .route("/api/provider/openai/v1/models", get(route::models::models))
        .route(
            "/api/provider/anthropic/v1/messages",
            post(route::messages::messages),
        )
        .route("/v1/messages", post(route::messages::messages))
        .route(
            "/api/provider/anthropic/v1/messages/count_tokens",
            post(route::messages::count_tokens),
        )
        .route(
            "/api/provider/openai/v1/responses",
            post(route::responses::responses).get(route::websocket::responses_ws),
        )
        .route(
            "/api/provider/openai/v1/responses/:response_id",
            get(route::responses::retrieve_response),
        )
        .route(
            "/api/provider/openai/v1/responses/:response_id/input_items",
            get(route::responses::response_input_items),
        )
        .route(
            "/v1/responses",
            post(route::responses::responses).get(route::websocket::responses_ws),
        )
        .route(
            "/v1/responses/:response_id",
            get(route::responses::retrieve_response),
        )
        .route(
            "/v1/responses/:response_id/input_items",
            get(route::responses::response_input_items),
        )
        .route(
            "/api/provider/openai/v1/chat/completions",
            post(route::chat::chat_completions),
        )
        .route("/v1/chat/completions", post(route::chat::chat_completions))
        .route(
            "/api/provider/openai/v1/realtime",
            get(route::websocket::realtime_ws),
        )
        .route(
            "/api/provider/openai/v1/realtime/transcription_sessions",
            post(route::audio::realtime_transcription_sessions),
        )
        .route(
            "/api/provider/openai/v1/realtime/client_secrets",
            post(route::audio::realtime_client_secrets),
        )
        .route(
            "/api/provider/openai/v1/realtime/calls",
            post(route::audio::realtime_calls),
        )
        .route(
            "/api/provider/openai/v1/realtime/calls/:call_id/accept",
            post(route::audio::realtime_unsupported_descendant),
        )
        .route(
            "/api/provider/openai/v1/realtime/calls/:call_id/reject",
            post(route::audio::realtime_unsupported_descendant),
        )
        .route(
            "/api/provider/openai/v1/realtime/calls/:call_id/hangup",
            post(route::audio::realtime_unsupported_descendant),
        )
        .route(
            "/api/provider/openai/v1/realtime/calls/:call_id/refer",
            post(route::audio::realtime_unsupported_descendant),
        )
        .route(
            "/api/provider/openai/v1/realtime/calls/:call_id/*path",
            any(route::audio::realtime_unsupported_descendant),
        )
        .route(
            "/api/provider/openai/v1/realtime/*path",
            any(route::audio::realtime_unsupported_descendant),
        )
        .route("/v1/realtime", get(route::websocket::realtime_ws))
        .route(
            "/v1/realtime/transcription_sessions",
            post(route::audio::realtime_transcription_sessions),
        )
        .route(
            "/v1/realtime/client_secrets",
            post(route::audio::realtime_client_secrets),
        )
        .route("/v1/realtime/calls", post(route::audio::realtime_calls))
        .route(
            "/v1/realtime/calls/:call_id/accept",
            post(route::audio::realtime_unsupported_descendant),
        )
        .route(
            "/v1/realtime/calls/:call_id/reject",
            post(route::audio::realtime_unsupported_descendant),
        )
        .route(
            "/v1/realtime/calls/:call_id/hangup",
            post(route::audio::realtime_unsupported_descendant),
        )
        .route(
            "/v1/realtime/calls/:call_id/refer",
            post(route::audio::realtime_unsupported_descendant),
        )
        .route(
            "/v1/realtime/calls/:call_id/*path",
            any(route::audio::realtime_unsupported_descendant),
        )
        .route(
            "/v1/realtime/*path",
            any(route::audio::realtime_unsupported_descendant),
        )
        .route(
            "/api/provider/openai/v1/audio/speech",
            post(route::audio::audio_speech),
        )
        .route("/v1/audio/speech", post(route::audio::audio_speech))
        .route(
            "/api/provider/openai/v1/audio/transcriptions",
            post(route::audio::transcribe),
        )
        .route(
            "/api/provider/openai/v1/audio/translations",
            post(route::audio::audio_translations),
        )
        .route("/v1/audio/transcriptions", post(route::audio::transcribe))
        .route(
            "/v1/audio/translations",
            post(route::audio::audio_translations),
        )
        .route(
            "/api/provider/openai/v1/audio/*path",
            any(route::audio::audio_unsupported_descendant),
        )
        .route(
            "/v1/audio/*path",
            any(route::audio::audio_unsupported_descendant),
        )
        .route(
            "/api/provider/openai/v1/images/generations",
            post(route::images::unsupported_generation),
        )
        .route(
            "/v1/images/generations",
            post(route::images::unsupported_generation),
        )
        .route(
            "/api/provider/openai/v1/images/edits",
            post(route::images::unsupported_edit),
        )
        .route("/v1/images/edits", post(route::images::unsupported_edit))
        .route(
            "/api/provider/openai/v1/files",
            any(route::unsupported_public_openai_route),
        )
        .route(
            "/api/provider/openai/v1/files/:file_id",
            any(route::unsupported_public_openai_route),
        )
        .route("/v1/files", any(route::unsupported_public_openai_route))
        .route(
            "/v1/files/:file_id",
            any(route::unsupported_public_openai_route),
        )
        .route(
            "/v1beta/models/*path",
            post(route::google::generate_content),
        )
        .route("/v1/models/*path", post(route::google::generate_content))
        .route("/v1/projects/*path", post(route::google::generate_content))
        .route(
            "/v1beta1/projects/*path",
            post(route::google::generate_content),
        )
        .route("/api/provider/google/*path", any(route::google::google))
        .fallback(route::not_found)
        .layer(middleware::from_fn(request_id_middleware))
        .layer(middleware::from_fn(config_admin_guard))
        .layer(TraceLayer::new_for_http())
        .layer(middleware::from_fn(content_encoding_middleware))
        .with_state(state)
}

async fn content_encoding_middleware(mut request: Request<Body>, next: Next) -> Response {
    if !request_body::has_content_encoding(request.headers()) {
        return next.run(request).await;
    }

    let body = std::mem::replace(request.body_mut(), Body::empty());
    let bytes = match to_bytes(body, request_body::MAX_ENCODED_REQUEST_BODY_BYTES).await {
        Ok(bytes) => bytes,
        Err(error) => {
            return AppError::BadRequest(format!("failed to read encoded request body: {error}"))
                .into_response();
        }
    };

    match request_body::decode_content_encoded_body(request.headers_mut(), bytes) {
        Ok(decoded) => {
            *request.body_mut() = Body::from(decoded);
            next.run(request).await
        }
        Err(error) => error.into_response(),
    }
}

async fn config_admin_guard(request: Request<Body>, next: Next) -> Response {
    if !is_config_admin_path(request.uri().path()) {
        return next.run(request).await;
    }

    if !request
        .headers()
        .get(header::HOST)
        .and_then(|host| host.to_str().ok())
        .is_some_and(is_loopback_host_with_optional_port)
    {
        return config_guard_failure(request.uri().path());
    }

    if is_unsafe_method(request.method()) {
        if request
            .headers()
            .get("sec-fetch-site")
            .and_then(|value| value.to_str().ok())
            .is_some_and(|value| value.eq_ignore_ascii_case("cross-site"))
        {
            return config_guard_failure(request.uri().path());
        }

        if request
            .headers()
            .get(header::ORIGIN)
            .and_then(|origin| origin.to_str().ok())
            .is_some_and(|origin| !is_loopback_origin(origin))
        {
            return config_guard_failure(request.uri().path());
        }
    }

    next.run(request).await
}

fn config_guard_failure(path: &str) -> Response {
    let mut response = if path.starts_with("/api/config") {
        (
            StatusCode::FORBIDDEN,
            Json(json!({
                "error": {
                    "message": "config admin access is only allowed from loopback same-origin requests",
                    "type": "permission_error",
                    "param": null,
                    "code": "config_admin_forbidden",
                }
            })),
        )
            .into_response()
    } else {
        (StatusCode::FORBIDDEN, "config admin access forbidden").into_response()
    };
    add_config_no_store_headers(response.headers_mut());
    response
}

fn is_config_admin_path(path: &str) -> bool {
    matches!(
        path,
        "/config"
            | "/config/assets/config.css"
            | "/config/assets/config.js"
            | "/api/config"
            | "/api/config/graph"
    )
}

fn is_unsafe_method(method: &Method) -> bool {
    matches!(
        *method,
        Method::POST | Method::PUT | Method::PATCH | Method::DELETE
    )
}

fn is_loopback_origin(origin: &str) -> bool {
    let Some(after_scheme) = origin
        .strip_prefix("http://")
        .or_else(|| origin.strip_prefix("https://"))
    else {
        return false;
    };
    let host = after_scheme
        .split(['/', '?', '#'])
        .next()
        .filter(|host| !host.is_empty())
        .unwrap_or_default();
    is_loopback_host_with_optional_port(host)
}

fn is_loopback_host_with_optional_port(host: &str) -> bool {
    if host.is_empty() || host.contains('@') {
        return false;
    }

    if let Some(rest) = host.strip_prefix("[::1]") {
        return rest.is_empty() || parse_optional_port(rest);
    }

    let Some((name, port)) = host.split_once(':') else {
        return is_allowed_loopback_name(host);
    };
    is_allowed_loopback_name(name) && parse_port(port)
}

fn is_allowed_loopback_name(name: &str) -> bool {
    name.eq_ignore_ascii_case("localhost") || name == "127.0.0.1"
}

fn parse_optional_port(rest: &str) -> bool {
    rest.strip_prefix(':').is_some_and(parse_port)
}

fn parse_port(port: &str) -> bool {
    !port.is_empty() && port.parse::<u16>().is_ok()
}

fn add_config_no_store_headers(headers: &mut HeaderMap) {
    headers.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    headers.insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );
}

async fn request_id_middleware(mut request: Request<Body>, next: Next) -> Response {
    let started = Instant::now();
    let request_id = request
        .headers()
        .get(REQUEST_ID_HEADER)
        .and_then(|value| value.to_str().ok())
        .filter(|value| is_valid_request_id(value))
        .map(ToOwned::to_owned)
        .unwrap_or_else(failure_capture::generate_request_id);

    request
        .extensions_mut()
        .insert(RequestId(request_id.clone()));

    let observation = request_observation(&mut request).await;
    let mut response = next.run(request).await;
    log_request_completion(
        &request_id,
        &observation,
        &response,
        started.elapsed().as_millis(),
    );
    let header_name = HeaderName::from_static(REQUEST_ID_HEADER);
    if let Ok(value) = HeaderValue::from_str(&request_id) {
        response.headers_mut().insert(header_name, value);
    }
    response
}

async fn request_observation(request: &mut Request<Body>) -> RequestObservation {
    let method = request.method().clone();
    let path = request.uri().path().to_string();
    let mut model = None;

    if !should_skip_observation_body(&path) && should_read_json_body(request.headers()) {
        let replacement = Body::empty();
        let body = std::mem::replace(request.body_mut(), replacement);
        match to_bytes(body, OBSERVABILITY_BODY_LIMIT).await {
            Ok(bytes) => {
                model = model_from_json_bytes(&bytes);
                *request.body_mut() = Body::from(bytes);
            }
            Err(err) => {
                tracing::warn!(%err, path = %path, "failed to inspect request body for observability");
                *request.body_mut() = Body::empty();
            }
        }
    }

    let provider = provider_for_request_path_and_model(&path, model.as_deref());
    RequestObservation {
        method,
        path,
        provider,
        model,
    }
}

fn should_skip_observation_body(path: &str) -> bool {
    path == "/config" || path.starts_with("/api/config")
}

fn should_read_json_body(headers: &HeaderMap) -> bool {
    let Some(content_type) = headers.get(header::CONTENT_TYPE) else {
        return false;
    };
    if !content_type
        .to_str()
        .map(|value| value.starts_with("application/json"))
        .unwrap_or(false)
    {
        return false;
    }

    headers
        .get(header::CONTENT_LENGTH)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse::<usize>().ok())
        .is_some_and(|length| length <= OBSERVABILITY_BODY_LIMIT)
}

fn model_from_json_bytes(bytes: &[u8]) -> Option<String> {
    serde_json::from_slice::<serde_json::Value>(bytes)
        .ok()
        .and_then(|value| {
            value
                .get("model")
                .and_then(|model| model.as_str())
                .map(str::to_owned)
        })
}

fn provider_for_request_path_and_model(path: &str, model: Option<&str>) -> Option<&'static str> {
    if path.starts_with("/api/provider/google/")
        || path.starts_with("/v1beta/models/")
        || path.starts_with("/v1/models/")
        || path.starts_with("/v1/projects/")
        || path.starts_with("/v1beta1/projects/")
    {
        return Some("google");
    }

    if let Some(model) = model.and_then(resolve_model) {
        return Some(match model.provider {
            Provider::Bedrock => "bedrock",
            Provider::Codex => "codex",
            Provider::Google => "google",
            Provider::Unsupported => "unsupported",
        });
    }

    if path.contains("/anthropic/") || path == "/v1/messages" {
        Some("bedrock")
    } else if path.contains("/openai/")
        || path == "/v1/responses"
        || path == "/v1/realtime"
        || path.starts_with("/v1/realtime/")
        || path.starts_with("/v1/audio/")
    {
        Some("codex")
    } else {
        None
    }
}

fn log_request_completion(
    request_id: &str,
    observation: &RequestObservation,
    response: &Response,
    elapsed_ms: u128,
) {
    let status = response.status();
    let upstream = response.extensions().get::<UpstreamResponseMetadata>();
    let provider = upstream
        .map(|metadata| metadata.provider)
        .or(observation.provider)
        .unwrap_or("local");
    let upstream_status = upstream
        .and_then(|metadata| metadata.upstream_status)
        .unwrap_or(status);
    let latency_ms = upstream
        .and_then(|metadata| metadata.latency_ms)
        .unwrap_or(elapsed_ms);
    let model = observation.model.as_deref().unwrap_or("unknown");

    tracing::info!(
        request_id = %request_id,
        method = %observation.method,
        path = %observation.path,
        provider = %provider,
        model = %model,
        status = status.as_u16(),
        upstream_status = upstream_status.as_u16(),
        latency_ms = latency_ms,
        "request completed"
    );
}

fn is_valid_request_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b':'))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;
    use std::{
        io::{self, Write},
        sync::{Arc, Mutex},
    };
    use tracing_subscriber::fmt::MakeWriter;

    #[derive(Clone)]
    struct CapturedLogs(Arc<Mutex<Vec<u8>>>);

    struct CapturedLogWriter(Arc<Mutex<Vec<u8>>>);

    impl Write for CapturedLogWriter {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.0.lock().unwrap().extend_from_slice(buf);
            Ok(buf.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    impl<'a> MakeWriter<'a> for CapturedLogs {
        type Writer = CapturedLogWriter;

        fn make_writer(&'a self) -> Self::Writer {
            CapturedLogWriter(self.0.clone())
        }
    }

    #[test]
    fn validates_bounded_ascii_request_ids() {
        assert!(is_valid_request_id("req_123-abc.def:456"));
        assert!(!is_valid_request_id(""));
        assert!(!is_valid_request_id("snowman-\u{2603}"));
        assert!(!is_valid_request_id(&"a".repeat(129)));
    }

    #[test]
    fn infers_provider_from_path_and_model() {
        assert_eq!(
            provider_for_request_path_and_model(
                "/api/provider/openai/v1/responses",
                Some("openai:gpt-5.5")
            ),
            Some("codex")
        );
        assert_eq!(
            provider_for_request_path_and_model(
                "/v1/messages",
                Some("anthropic/claude-sonnet-4-6")
            ),
            Some("bedrock")
        );
        assert_eq!(
            provider_for_request_path_and_model(
                "/api/provider/google/v1beta/models/gemini-2.5-pro:generateContent",
                None
            ),
            Some("google")
        );
    }

    #[test]
    fn reads_model_from_small_json_body_only_when_content_length_is_bounded() {
        let body = br#"{"model":"openai:gpt-5.5"}"#;
        assert_eq!(
            model_from_json_bytes(body),
            Some("openai:gpt-5.5".to_string())
        );

        let mut headers = HeaderMap::new();
        headers.insert(header::CONTENT_TYPE, "application/json".parse().unwrap());
        headers.insert(
            header::CONTENT_LENGTH,
            body.len().to_string().parse().unwrap(),
        );
        assert!(should_read_json_body(&headers));

        headers.insert(
            header::CONTENT_LENGTH,
            (OBSERVABILITY_BODY_LIMIT + 1).to_string().parse().unwrap(),
        );
        assert!(!should_read_json_body(&headers));
    }

    #[test]
    fn request_completion_log_includes_required_observability_fields() {
        let metadata = UpstreamResponseMetadata {
            provider: "bedrock",
            upstream_status: Some(StatusCode::BAD_GATEWAY),
            latency_ms: Some(42),
        };
        let mut response = Response::new(Body::empty());
        *response.status_mut() = StatusCode::BAD_GATEWAY;
        response.extensions_mut().insert(metadata);

        let observation = RequestObservation {
            method: Method::POST,
            path: "/v1/messages".to_string(),
            provider: Some("bedrock"),
            model: Some("anthropic/claude-sonnet-4-6".to_string()),
        };

        let logs = Arc::new(Mutex::new(Vec::new()));
        let subscriber = tracing_subscriber::fmt()
            .with_ansi(false)
            .without_time()
            .with_writer(CapturedLogs(logs.clone()))
            .finish();

        tracing::subscriber::with_default(subscriber, || {
            log_request_completion("req", &observation, &response, 1);
        });

        let logs = String::from_utf8(logs.lock().unwrap().clone()).unwrap();
        assert!(logs.contains("request_id=req"));
        assert!(logs.contains("path=/v1/messages"));
        assert!(logs.contains("provider=bedrock"));
        assert!(logs.contains("model=anthropic/claude-sonnet-4-6"));
        assert!(logs.contains("upstream_status=502"));
        assert!(logs.contains("latency_ms=42"));
    }
}
