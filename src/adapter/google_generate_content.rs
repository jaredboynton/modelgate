use crate::error::{AppError, AppResult};
use serde_json::{json, Map, Value};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum GoogleGenerateContentCaller {
    Gemini,
    Vertex,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum GoogleGenerateContentAction {
    GenerateContent,
    StreamGenerateContent,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct GoogleGenerateContentRoute {
    pub caller: GoogleGenerateContentCaller,
    pub action: GoogleGenerateContentAction,
    pub api_version: String,
    pub model: String,
    pub project: Option<String>,
    pub location: Option<String>,
}

impl GoogleGenerateContentRoute {
    pub fn stream(&self) -> bool {
        self.action == GoogleGenerateContentAction::StreamGenerateContent
    }
}

pub struct GoogleGenerateContentSseTranslator {
    caller: GoogleGenerateContentCaller,
    buffer: String,
}

impl GoogleGenerateContentSseTranslator {
    pub fn new(caller: GoogleGenerateContentCaller) -> Self {
        Self {
            caller,
            buffer: String::new(),
        }
    }

    pub fn push_bytes(&mut self, bytes: &[u8]) -> AppResult<Vec<u8>> {
        let text = std::str::from_utf8(bytes)
            .map_err(|err| AppError::Upstream(format!("invalid Google SSE UTF-8: {err}")))?;
        self.buffer.push_str(text);

        let mut output = String::new();
        while let Some(frame_end) = find_sse_frame_end(&self.buffer) {
            let frame: String = self.buffer.drain(..frame_end).collect();
            drain_frame_separator(&mut self.buffer);
            output.push_str(&format_generate_content_sse_frame(&frame, self.caller)?);
        }

        Ok(output.into_bytes())
    }
}

pub fn parse_google_generate_content_route(
    path_and_query: &str,
) -> AppResult<GoogleGenerateContentRoute> {
    let path = path_and_query
        .split_once('?')
        .map(|(path, _)| path)
        .unwrap_or(path_and_query);
    let segments: Vec<&str> = path.split('/').collect();

    if let Some(route) = parse_gemini_route(&segments)? {
        return Ok(route);
    }
    if let Some(route) = parse_vertex_route(&segments)? {
        return Ok(route);
    }

    Err(AppError::BadRequest(format!(
        "unsupported Google generateContent route: {path}"
    )))
}

pub fn format_generate_content_response_for_caller(
    value: Value,
    caller: GoogleGenerateContentCaller,
) -> AppResult<Value> {
    match caller {
        GoogleGenerateContentCaller::Gemini => Ok(filter_response_fields(
            value,
            &[
                "candidates",
                "promptFeedback",
                "usageMetadata",
                "modelVersion",
                "responseId",
                "modelStatus",
            ],
        )),
        GoogleGenerateContentCaller::Vertex => Ok(filter_response_fields(
            value,
            &[
                "candidates",
                "promptFeedback",
                "usageMetadata",
                "modelVersion",
                "responseId",
                "createTime",
            ],
        )),
    }
}

pub fn google_generate_content_sse_to_text(
    input: &str,
    caller: GoogleGenerateContentCaller,
) -> AppResult<String> {
    let mut output = String::new();
    for frame in input.split("\n\n") {
        output.push_str(&format_generate_content_sse_frame(frame, caller)?);
    }
    Ok(output)
}

fn parse_gemini_route(segments: &[&str]) -> AppResult<Option<GoogleGenerateContentRoute>> {
    if segments.len() != 4 || !segments[0].is_empty() || segments[2] != "models" {
        return Ok(None);
    }
    let api_version = segments[1];
    if api_version != "v1" && api_version != "v1beta" {
        return Ok(None);
    }

    let (model, action) = parse_model_action(segments[3])?;
    Ok(Some(GoogleGenerateContentRoute {
        caller: GoogleGenerateContentCaller::Gemini,
        action,
        api_version: api_version.to_string(),
        model,
        project: None,
        location: None,
    }))
}

fn parse_vertex_route(segments: &[&str]) -> AppResult<Option<GoogleGenerateContentRoute>> {
    if segments.len() != 10
        || !segments[0].is_empty()
        || segments[2] != "projects"
        || segments[4] != "locations"
        || segments[6] != "publishers"
        || segments[8] != "models"
    {
        return Ok(None);
    }
    let api_version = segments[1];
    if api_version != "v1" && api_version != "v1beta1" {
        return Ok(None);
    }

    let project = decode_and_validate_path_segment(segments[3], "project")?;
    let location = decode_and_validate_path_segment(segments[5], "location")?;
    let publisher = decode_and_validate_path_segment(segments[7], "publisher")?;
    if publisher != "google" {
        return Err(AppError::BadRequest(format!(
            "unsupported Vertex publisher: {publisher}"
        )));
    }
    let (model, action) = parse_model_action(segments[9])?;

    Ok(Some(GoogleGenerateContentRoute {
        caller: GoogleGenerateContentCaller::Vertex,
        action,
        api_version: api_version.to_string(),
        model,
        project: Some(project),
        location: Some(location),
    }))
}

fn parse_model_action(raw: &str) -> AppResult<(String, GoogleGenerateContentAction)> {
    let (raw_model, raw_action) = raw.rsplit_once(':').ok_or_else(|| {
        AppError::BadRequest("Google route is missing generateContent action".to_string())
    })?;
    let model = decode_and_validate_path_segment(raw_model, "model")?;
    let action = match raw_action {
        "generateContent" => GoogleGenerateContentAction::GenerateContent,
        "streamGenerateContent" => GoogleGenerateContentAction::StreamGenerateContent,
        other => {
            return Err(AppError::BadRequest(format!(
                "unsupported Google generateContent action: {other}"
            )));
        }
    };
    Ok((model, action))
}

fn decode_and_validate_path_segment(raw: &str, label: &str) -> AppResult<String> {
    let decoded = percent_decode_path_segment(raw)?;
    if decoded.is_empty()
        || decoded.contains('/')
        || decoded.contains("..")
        || decoded.contains('?')
        || decoded.contains('&')
        || decoded.contains('\0')
    {
        return Err(AppError::BadRequest(format!(
            "invalid Google {label} path segment"
        )));
    }
    Ok(decoded)
}

fn percent_decode_path_segment(raw: &str) -> AppResult<String> {
    let bytes = raw.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' {
            if index + 2 >= bytes.len() {
                return Err(AppError::BadRequest(
                    "invalid percent escape in Google path segment".to_string(),
                ));
            }
            let high = hex_value(bytes[index + 1]).ok_or_else(|| {
                AppError::BadRequest("invalid percent escape in Google path segment".to_string())
            })?;
            let low = hex_value(bytes[index + 2]).ok_or_else(|| {
                AppError::BadRequest("invalid percent escape in Google path segment".to_string())
            })?;
            decoded.push((high << 4) | low);
            index += 3;
        } else {
            decoded.push(bytes[index]);
            index += 1;
        }
    }

    String::from_utf8(decoded)
        .map_err(|err| AppError::BadRequest(format!("invalid UTF-8 in Google path segment: {err}")))
}

fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

fn filter_response_fields(value: Value, keys: &[&str]) -> Value {
    let Value::Object(source) = value else {
        return value;
    };

    let mut output = Map::new();
    for key in keys {
        if let Some(value) = source.get(*key) {
            output.insert((*key).to_string(), value.clone());
        }
    }
    Value::Object(output)
}

fn format_generate_content_sse_frame(
    frame: &str,
    caller: GoogleGenerateContentCaller,
) -> AppResult<String> {
    let data = collect_sse_data(frame);
    if data.is_empty() {
        return Ok(String::new());
    }
    if data == "[DONE]" {
        return Ok("data: [DONE]\n\n".to_string());
    }

    let value: Value = serde_json::from_str(&data).map_err(|err| {
        AppError::Upstream(format!("invalid Google generateContent SSE JSON: {err}"))
    })?;
    let shaped = format_generate_content_response_for_caller(value, caller)?;
    Ok(format!("data: {}\n\n", compact_json(&shaped)))
}

fn collect_sse_data(frame: &str) -> String {
    let mut data_lines = Vec::new();
    for line in frame.lines() {
        let line = line.trim_end_matches('\r');
        if let Some(data) = line.strip_prefix("data:") {
            data_lines.push(data.trim_start());
        }
    }
    data_lines.join("\n")
}

fn compact_json(value: &Value) -> String {
    serde_json::to_string(value).unwrap_or_else(|_| json!({}).to_string())
}

fn find_sse_frame_end(buffer: &str) -> Option<usize> {
    buffer.find("\n\n").or_else(|| buffer.find("\r\n\r\n"))
}

fn drain_frame_separator(buffer: &mut String) {
    if buffer.starts_with("\r\n\r\n") {
        buffer.drain(..4);
    } else if buffer.starts_with("\n\n") {
        buffer.drain(..2);
    }
}
