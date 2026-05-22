use std::{convert::Infallible, io};

use axum::{
    body::Body,
    http::{header, HeaderMap, HeaderName, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
};
use bytes::Bytes;
use futures::{Stream, TryStreamExt};

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

    pub fn from_reqwest(provider: &'static str, response: reqwest::Response) -> Self {
        observe_reqwest_response(provider, &response);
        let status = response.status();
        let headers = sanitize_headers(response.headers());
        let stream = response.bytes_stream().map_err(io::Error::other);
        Self {
            status,
            headers,
            body: Body::from_stream(stream),
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

pub fn observe_reqwest_response(provider: &'static str, response: &reqwest::Response) {
    tracing::debug!(
        provider,
        status = %response.status(),
        version = ?response.version(),
        "upstream HTTP response"
    );
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
