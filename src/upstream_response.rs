use std::{convert::Infallible, io};

use axum::{
    body::{to_bytes, Body},
    http::{header, HeaderMap, HeaderName, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
};
use bytes::Bytes;
use futures::{stream, Stream, TryStreamExt};
use specter::{Body as SpecterBody, Headers as SpecterHeaders, Response as SpecterResponse};

use crate::AppError;

const PRESERVED_HEADERS: &[&str] = &[
    "content-type",
    "cache-control",
    "x-accel-buffering",
    "x-request-id",
    "request-id",
    "anthropic-request-id",
    "openai-request-id",
    "x-amzn-requestid",
];

const HOP_BY_HOP_HEADERS: &[&str] = &[
    "connection",
    "keep-alive",
    "proxy-authenticate",
    "proxy-authorization",
    "te",
    "trailer",
    "transfer-encoding",
    "upgrade",
];

pub const MAX_UPSTREAM_BODY_BYTES: usize = 16 * 1024 * 1024;
pub const MAX_UPSTREAM_ERROR_BODY_BYTES: usize = 1024 * 1024;

pub struct UpstreamResponse {
    pub status: StatusCode,
    pub headers: HeaderMap,
    pub body: Body,
    pub provider: &'static str,
    pub upstream_status: Option<StatusCode>,
    pub latency_ms: Option<u128>,
}

#[derive(Clone, Debug)]
pub struct UpstreamResponseMetadata {
    pub provider: &'static str,
    pub upstream_status: Option<StatusCode>,
    pub latency_ms: Option<u128>,
}

impl UpstreamResponse {
    pub fn bytes(
        provider: &'static str,
        status: StatusCode,
        headers: HeaderMap,
        body: Bytes,
    ) -> Self {
        Self {
            status,
            headers: sanitize_headers(&headers),
            body: Body::from(body),
            provider,
            upstream_status: Some(status),
            latency_ms: None,
        }
    }

    pub fn json(
        provider: &'static str,
        value: serde_json::Value,
    ) -> Result<Self, serde_json::Error> {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        );
        Ok(Self::bytes(
            provider,
            StatusCode::OK,
            headers,
            Bytes::from(serde_json::to_vec(&value)?),
        ))
    }

    pub fn stream<S, E>(
        provider: &'static str,
        status: StatusCode,
        headers: HeaderMap,
        stream: S,
    ) -> Self
    where
        S: Stream<Item = Result<Bytes, E>> + Send + 'static,
        E: Into<Box<dyn std::error::Error + Send + Sync>> + 'static,
    {
        let stream = stream.map_err(io::Error::other);
        Self {
            status,
            headers: sanitize_headers(&headers),
            body: Body::from_stream(stream),
            provider,
            upstream_status: Some(status),
            latency_ms: None,
        }
    }

    pub fn from_specter(provider: &'static str, response: SpecterResponse) -> Self {
        observe_specter_response(provider, &response);
        let status = response.status();
        Self {
            status,
            headers: sanitize_specter_headers(response.headers()),
            body: specter_body_to_axum_body(response.into_body()),
            provider,
            upstream_status: Some(status),
            latency_ms: None,
        }
    }

    pub fn with_latency_ms(mut self, latency_ms: u128) -> Self {
        self.latency_ms = Some(latency_ms);
        self
    }
}

pub fn sse_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("text/event-stream"),
    );
    headers.insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("no-cache, no-transform"),
    );
    headers.insert(
        HeaderName::from_static("x-accel-buffering"),
        HeaderValue::from_static("no"),
    );
    headers
}

pub fn observe_specter_response(provider: &'static str, response: &SpecterResponse) {
    tracing::debug!(
        provider,
        status = %response.status(),
        version = response.http_version(),
        "upstream HTTP response"
    );
}

pub async fn collect_upstream_body(body: Body) -> Result<Bytes, AppError> {
    collect_limited_body(body, MAX_UPSTREAM_BODY_BYTES, "upstream response").await
}

pub async fn collect_upstream_error_body(body: Body) -> Result<Bytes, AppError> {
    collect_limited_body(
        body,
        MAX_UPSTREAM_ERROR_BODY_BYTES,
        "upstream error response",
    )
    .await
}

pub async fn collect_specter_body(
    mut body: SpecterBody,
    context: &'static str,
) -> Result<Bytes, AppError> {
    body.collect_to_bytes()
        .await
        .map_err(|error| AppError::Upstream(format!("{context}: {error}")))
}

pub fn specter_body_stream(
    body: SpecterBody,
    context: &'static str,
) -> futures::stream::BoxStream<'static, Result<Bytes, AppError>> {
    Box::pin(stream::unfold(body, move |mut body| async move {
        body.chunk().await.map(|chunk| {
            let item = chunk.map_err(|error| AppError::Upstream(format!("{context}: {error}")));
            (item, body)
        })
    }))
}

fn specter_body_to_axum_body(body: SpecterBody) -> Body {
    let stream = specter_body_stream(body, "upstream response").map_err(io::Error::other);
    Body::from_stream(stream)
}

async fn collect_limited_body(body: Body, limit: usize, context: &str) -> Result<Bytes, AppError> {
    to_bytes(body, limit).await.map_err(|error| {
        AppError::Upstream(format!(
            "{context} body exceeded {limit} bytes or could not be read: {error}"
        ))
    })
}

pub fn sanitize_specter_headers(headers: &SpecterHeaders) -> HeaderMap {
    let mut sanitized = HeaderMap::new();
    for (name, value) in headers.iter() {
        let Ok(name) = HeaderName::from_bytes(name.as_bytes()) else {
            continue;
        };
        let Ok(value) = HeaderValue::from_str(value) else {
            continue;
        };
        if should_preserve_header(&name) {
            sanitized.insert(name, value);
        }
    }
    sanitized
}

impl IntoResponse for UpstreamResponse {
    fn into_response(self) -> Response {
        let metadata = UpstreamResponseMetadata {
            provider: self.provider,
            upstream_status: self.upstream_status,
            latency_ms: self.latency_ms,
        };
        let mut response = (self.status, self.body).into_response();
        *response.headers_mut() = self.headers;
        response.extensions_mut().insert(metadata);
        response
    }
}

impl From<UpstreamResponse> for Result<Response, Infallible> {
    fn from(value: UpstreamResponse) -> Self {
        Ok(value.into_response())
    }
}

impl From<AppError> for UpstreamResponse {
    fn from(error: AppError) -> Self {
        error.into_response().into()
    }
}

impl From<Response> for UpstreamResponse {
    fn from(response: Response) -> Self {
        let status = response.status();
        let headers = sanitize_headers(response.headers());
        Self {
            status,
            headers,
            body: response.into_body(),
            provider: "local",
            upstream_status: None,
            latency_ms: None,
        }
    }
}

pub fn sanitize_headers(headers: &HeaderMap) -> HeaderMap {
    let mut sanitized = HeaderMap::new();
    for (name, value) in headers {
        if should_preserve_header(name) {
            sanitized.insert(name.clone(), value.clone());
        }
    }
    sanitized
}

fn should_preserve_header(name: &HeaderName) -> bool {
    let lower = name.as_str().to_ascii_lowercase();
    if HOP_BY_HOP_HEADERS.contains(&lower.as_str()) {
        return false;
    }
    if matches!(
        lower.as_str(),
        "authorization" | "cookie" | "set-cookie" | "x-api-key" | "x-goog-api-key"
    ) {
        return false;
    }
    PRESERVED_HEADERS.contains(&lower.as_str())
}
