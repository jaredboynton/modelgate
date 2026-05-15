use axum::{
    body::Bytes,
    extract::State,
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{Html, IntoResponse, Response},
    Json,
};
use serde_json::{json, Value};

use crate::{config_graph, error::openai_error_body, AppError, AppState};

const CONFIG_CSP: &str = "default-src 'none'; base-uri 'none'; frame-ancestors 'none'; form-action 'none'; connect-src 'self'; img-src 'self' data:; style-src 'self'; script-src 'self'; object-src 'none'";
const CONFIG_HTML: &str = include_str!("../assets/config/page.html");
const CONFIG_CSS: &str = include_str!("../assets/config/config.css");
const CONFIG_JS: &str = include_str!("../assets/config/config.js");

pub async fn config_page() -> Response {
    html_response(CONFIG_HTML)
}

pub async fn config_css() -> Response {
    static_response("text/css; charset=utf-8", CONFIG_CSS)
}

pub async fn config_js() -> Response {
    static_response("application/javascript; charset=utf-8", CONFIG_JS)
}

pub async fn get_config(State(state): State<AppState>) -> Response {
    json_result_response(state.routing_config.read_json())
}

pub async fn put_config(State(state): State<AppState>, body: Bytes) -> Response {
    let config: Value = match serde_json::from_slice(&body) {
        Ok(config) => config,
        Err(error) => return invalid_routing_config_response(error.to_string()),
    };

    if let Err(error) = state.routing_config.write_json(&config) {
        return routing_config_error_response(error);
    }

    json_response(
        StatusCode::OK,
        json!({
            "ok": true,
            "config": config,
        }),
    )
}

pub async fn get_config_graph(State(state): State<AppState>) -> Response {
    match state.routing_config.read_json() {
        Ok(config) => graph_response(config),
        Err(error) => routing_config_error_response(error),
    }
}

pub async fn post_config_graph(body: Bytes) -> Response {
    let draft: Value = match serde_json::from_slice(&body) {
        Ok(draft) => draft,
        Err(_) => return sanitized_invalid_routing_config_response(),
    };
    sanitized_graph_response(draft)
}

fn html_response(html: &'static str) -> Response {
    let mut response = Html(html).into_response();
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("text/html; charset=utf-8"),
    );
    response.headers_mut().insert(
        header::CONTENT_SECURITY_POLICY,
        HeaderValue::from_static(CONFIG_CSP),
    );
    add_admin_headers(response.headers_mut());
    response
}

fn static_response(content_type: &'static str, body: &'static str) -> Response {
    let mut response = body.into_response();
    response
        .headers_mut()
        .insert(header::CONTENT_TYPE, HeaderValue::from_static(content_type));
    add_admin_headers(response.headers_mut());
    response
}

fn json_result_response(result: Result<Value, AppError>) -> Response {
    match result {
        Ok(value) => json_response(StatusCode::OK, value),
        Err(error) => routing_config_error_response(error),
    }
}

fn json_response(status: StatusCode, body: Value) -> Response {
    let mut response = (status, Json(body)).into_response();
    add_admin_headers(response.headers_mut());
    response
}

fn routing_config_error_response(error: AppError) -> Response {
    match error {
        AppError::BadRequest(message) => invalid_routing_config_response(message),
        AppError::Json(error) => invalid_routing_config_response(error.to_string()),
        other => {
            let status = other.status();
            let body = openai_error_body(other.to_string(), other.error_type(), None, other.code());
            json_response(status, body)
        }
    }
}

fn invalid_routing_config_response(message: String) -> Response {
    json_response(
        StatusCode::BAD_REQUEST,
        openai_error_body(
            format!("invalid routing config: {message}"),
            "invalid_request_error",
            None,
            Some("invalid_routing_config"),
        ),
    )
}

fn sanitized_invalid_routing_config_response() -> Response {
    json_response(
        StatusCode::BAD_REQUEST,
        openai_error_body(
            "invalid routing config",
            "invalid_request_error",
            None,
            Some("invalid_routing_config"),
        ),
    )
}

fn add_admin_headers(headers: &mut HeaderMap) {
    headers.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    headers.insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );
}

fn graph_response(raw_hot_config: Value) -> Response {
    match config_graph::build_config_graph(raw_hot_config)
        .and_then(|graph| serde_json::to_value(graph).map_err(AppError::Json))
    {
        Ok(graph) => json_response(StatusCode::OK, graph),
        Err(error) => routing_config_error_response(error),
    }
}

fn sanitized_graph_response(raw_hot_config: Value) -> Response {
    match config_graph::build_config_graph(raw_hot_config)
        .and_then(|graph| serde_json::to_value(graph).map_err(AppError::Json))
    {
        Ok(graph) => json_response(StatusCode::OK, graph),
        Err(_) => sanitized_invalid_routing_config_response(),
    }
}
