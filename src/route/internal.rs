use axum::{
    body::Bytes,
    extract::{Path, Query, RawQuery, State},
    http::{header, HeaderMap, HeaderValue, Method, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

use crate::{
    amp_compat::{self, ThreadsFindQuery, ThreadsMarkdownQuery},
    AppState,
};

pub async fn amp_internal(
    State(state): State<AppState>,
    RawQuery(query): RawQuery,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    let decoded = amp_compat::decode_request_body(&headers, body);
    amp_compat::dispatch_internal(
        &state.amp_store,
        query.as_deref().unwrap_or_default(),
        decoded,
    )
    .await
}

pub async fn telemetry(_body: Bytes) -> Response {
    let mut response = StatusCode::NO_CONTENT.into_response();
    response.headers_mut().insert(
        header::ACCESS_CONTROL_ALLOW_ORIGIN,
        HeaderValue::from_static("*"),
    );
    response
}

pub async fn bitbucket_instance_url() -> Json<serde_json::Value> {
    Json(json!({}))
}

pub async fn github_auth_status() -> Json<serde_json::Value> {
    Json(json!({ "authenticated": false }))
}

pub async fn github_proxy(
    method: Method,
    Path(path): Path<String>,
    RawQuery(query): RawQuery,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    let _ = (method, path, query, headers, body);
    Json(json!({
        "ok": false,
        "error": {
            "code": "provider-auth-failed",
            "message": "GitHub proxy requires explicit credential support and is disabled in the local v2 proxy"
        }
    }))
    .into_response()
}

pub async fn attachment_post(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    let decoded = amp_compat::decode_request_body(&headers, body);
    let origin = request_origin(&headers);
    amp_compat::attachment_post(&state.amp_store, origin, decoded).await
}

pub async fn attachment_get(State(state): State<AppState>, Path(id): Path<String>) -> Response {
    amp_compat::attachment_get(&state.amp_store, id).await
}

pub async fn threads_find(
    State(state): State<AppState>,
    Query(query): Query<ThreadsFindQuery>,
) -> Response {
    amp_compat::threads_find(&state.amp_store, query).await
}

pub async fn thread_markdown(
    State(state): State<AppState>,
    Path(file_name): Path<String>,
    Query(query): Query<ThreadsMarkdownQuery>,
) -> Response {
    amp_compat::thread_markdown(&state.amp_store, file_name, query).await
}

pub async fn news_rss() -> Response {
    let body = r#"<?xml version="1.0" encoding="UTF-8"?><rss version="2.0"><channel><title>Local Amp Proxy</title></channel></rss>"#;
    let mut response = Response::new(axum::body::Body::from(body));
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/rss+xml; charset=utf-8"),
    );
    response
}

fn request_origin(headers: &HeaderMap) -> String {
    let host = headers
        .get(header::HOST)
        .and_then(|value| value.to_str().ok())
        .unwrap_or("127.0.0.1:18743");
    format!("http://{host}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::to_bytes;
    use tempfile::TempDir;

    fn state() -> AppState {
        let temp = TempDir::new().unwrap();
        let path = temp.keep();
        AppState::for_tests(path.join("codex"), path.join("auth"))
    }

    #[tokio::test]
    async fn startup_methods_return_amp_ok_envelope() {
        let state = state();
        let response = amp_compat::dispatch_internal(
            &state.amp_store,
            "getUserInfo",
            Bytes::from_static(br#"{"method":"getUserInfo","params":{}}"#),
        )
        .await;
        assert_eq!(response.status(), StatusCode::OK);

        let response = amp_compat::dispatch_internal(
            &state.amp_store,
            "loadPlugins",
            Bytes::from_static(br#"{"method":"loadPlugins","params":{}}"#),
        )
        .await;
        assert_eq!(response.status(), StatusCode::OK);

        let response =
            amp_compat::dispatch_internal(&state.amp_store, "getUserFreeTierStatus", Bytes::new())
                .await;
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn telemetry_returns_204() {
        let response = telemetry(Bytes::new()).await;
        assert_eq!(response.status(), StatusCode::NO_CONTENT);
    }

    #[tokio::test]
    async fn news_rss_returns_feed() {
        let response = news_rss().await;
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        assert!(String::from_utf8_lossy(&bytes).contains("<rss"));
    }
}
