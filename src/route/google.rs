use axum::{
    body::Bytes,
    extract::State,
    http::{HeaderMap, Method, Uri},
};
use futures::StreamExt;

use crate::{
    adapter::google_generate_content::{
        format_generate_content_response_for_caller, parse_google_generate_content_route,
        GoogleGenerateContentCaller, GoogleGenerateContentSseTranslator,
    },
    upstream,
    upstream_response::{collect_upstream_body, sse_headers},
    AppError, AppResult, AppState, UpstreamResponse,
};

pub async fn google(
    State(state): State<AppState>,
    method: Method,
    uri: Uri,
    headers: HeaderMap,
    body: Bytes,
) -> AppResult<UpstreamResponse> {
    let path = uri
        .path_and_query()
        .map_or(uri.path(), |value| value.as_str());
    upstream::google::forward_google(&state, method, path, headers, body).await
}

pub async fn generate_content(
    State(state): State<AppState>,
    method: Method,
    uri: Uri,
    headers: HeaderMap,
    body: Bytes,
) -> AppResult<UpstreamResponse> {
    if method != Method::POST {
        return Err(AppError::BadRequest(
            "Google generateContent routes require POST".into(),
        ));
    }
    let path = uri
        .path_and_query()
        .map_or(uri.path(), |value| value.as_str());
    let route = parse_google_generate_content_route(path)?;

    let upstream_response = if route.stream() {
        upstream::google::forward_stream_generate_content_direct_response(
            &state,
            &route.model,
            headers,
            body,
        )
        .await?
    } else {
        upstream::google::forward_generate_content_direct_response(
            &state,
            &route.model,
            headers,
            body,
        )
        .await?
    };

    generate_content_response_for_caller(upstream_response, route.caller, route.stream()).await
}

async fn generate_content_response_for_caller(
    response: UpstreamResponse,
    caller: GoogleGenerateContentCaller,
    stream_response: bool,
) -> AppResult<UpstreamResponse> {
    if !response.status.is_success() {
        return Ok(response);
    }

    let provider = response.provider;
    if stream_response {
        let mut translator = GoogleGenerateContentSseTranslator::new(caller);
        let stream = response
            .body
            .into_data_stream()
            .map(move |chunk| match chunk {
                Ok(bytes) => translator.push_bytes(&bytes).map(Bytes::from),
                Err(error) => Err(AppError::Upstream(format!(
                    "Google generateContent SSE stream failed: {error}"
                ))),
            })
            .filter_map(|chunk| async move {
                match chunk {
                    Ok(bytes) if bytes.is_empty() => None,
                    other => Some(other),
                }
            });
        return Ok(UpstreamResponse::stream(
            provider,
            response.status,
            sse_headers(),
            stream,
        ));
    }

    let body = collect_upstream_body(response.body).await?;
    let google: serde_json::Value = serde_json::from_slice(&body)?;
    let shaped = format_generate_content_response_for_caller(google, caller)?;
    UpstreamResponse::json(provider, shaped).map_err(AppError::Json)
}
