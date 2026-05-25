use std::collections::VecDeque;

use bytes::Bytes;
use futures::{stream, StreamExt};
use serde_json::Value;
use specter::HttpVersion;
use uuid::Uuid;

use crate::{
    auth,
    upstream_response::{collect_specter_body, specter_body_stream},
    AppError, AppResult, AppState,
};

pub const WINDSURF_PROVIDER: &str = "windsurf";
const GET_CHAT_MESSAGE_PATH: &str = "/exa.api_server_pb.ApiServerService/GetChatMessage";
const DEFAULT_WINDSURF_VERSION: &str = "1.13.104";

pub async fn ensure_credentials(state: &AppState) -> AppResult<()> {
    auth::windsurf::api_key(state).map(|_| ())
}

pub async fn collect_chat_text(
    state: &AppState,
    request: &Value,
    upstream_model: &str,
) -> AppResult<String> {
    let response = send_chat_request(state, request, upstream_model).await?;
    let bytes = response
        .bytes()
        .map_err(|error| AppError::Upstream(format!("Windsurf stream failed: {error}")))?;
    parse_complete_response(&bytes)
}

pub async fn stream_chat_text(
    state: &AppState,
    request: &Value,
    upstream_model: &str,
) -> AppResult<futures::stream::BoxStream<'static, AppResult<String>>> {
    match send_chat_request_streaming(state, request, upstream_model).await? {
        WindsurfStreamResponse::Buffered(response) => {
            let status = response.status();
            let body = response
                .bytes()
                .map_err(|error| AppError::Upstream(format!("Windsurf stream failed: {error}")))?;
            if !status.is_success() {
                return Err(AppError::Upstream(format!(
                    "Windsurf returned {status}: {}",
                    String::from_utf8_lossy(&body)
                )));
            }
            let text = parse_complete_response(&body)?;
            Ok(stream::once(async move { Ok(text) }).boxed())
        }
        WindsurfStreamResponse::Streaming(response) => {
            let status = response.status();
            let body = response.into_body();
            if !status.is_success() {
                let body = collect_specter_body(body, "Windsurf stream failed").await?;
                return Err(AppError::Upstream(format!(
                    "Windsurf returned {status}: {}",
                    String::from_utf8_lossy(&body)
                )));
            }
            let source = specter_body_stream(body, "Windsurf stream failed");
            let state = StreamState {
                source,
                buffer: Vec::new(),
                pending: VecDeque::new(),
                finished: false,
            };
            Ok(stream::unfold(state, |mut state| async move {
                if let Some(item) = state.pending.pop_front() {
                    return Some((item, state));
                }
                if state.finished {
                    return None;
                }

                loop {
                    match state.source.next().await {
                        Some(Ok(bytes)) => {
                            state.buffer.extend_from_slice(&bytes);
                            match drain_text_chunks(&mut state.buffer) {
                                Ok(chunks) => {
                                    if chunks.is_empty() {
                                        continue;
                                    }
                                    state.pending = chunks.into_iter().map(Ok).collect();
                                    let item = state.pending.pop_front().expect("pending chunk");
                                    return Some((item, state));
                                }
                                Err(error) => {
                                    state.finished = true;
                                    return Some((Err(error), state));
                                }
                            }
                        }
                        Some(Err(error)) => {
                            state.finished = true;
                            return Some((
                                Err(AppError::Upstream(format!(
                                    "Windsurf stream failed: {error}"
                                ))),
                                state,
                            ));
                        }
                        None => {
                            state.finished = true;
                            if state.buffer.iter().any(|byte| !byte.is_ascii_whitespace()) {
                                return Some((
                                    Err(AppError::Upstream(
                                        "Windsurf response ended with an incomplete Connect frame"
                                            .into(),
                                    )),
                                    state,
                                ));
                            }
                            return None;
                        }
                    }
                }
            })
            .boxed())
        }
    }
}

async fn send_chat_request(
    state: &AppState,
    request: &Value,
    upstream_model: &str,
) -> AppResult<specter::Response> {
    let api_key = auth::windsurf::api_key(state)?;
    let payload = build_get_chat_message_request(
        request,
        &api_key,
        DEFAULT_WINDSURF_VERSION,
        upstream_model,
    )?;
    let body = connect_envelope(&payload);
    let url = format!(
        "{}{}",
        state.runtime.windsurf_cloud_base_url.trim_end_matches('/'),
        GET_CHAT_MESSAGE_PATH
    );
    let client = state.specter.clone();
    let response = client
        .post(url)
        .header("content-type", "application/connect+proto")
        .header("connect-protocol-version", "1")
        .header("accept", "application/connect+proto")
        .body(body)
        .send()
        .await
        .map_err(|error| AppError::Upstream(format!("Windsurf request failed: {error}")))?;

    if !response.status().is_success() {
        let status = response.status();
        let text = response
            .text()
            .unwrap_or_else(|error| format!("failed to read Windsurf error body: {error}"));
        return Err(AppError::Upstream(format!(
            "Windsurf returned {status}: {text}"
        )));
    }

    Ok(response)
}

async fn send_chat_request_streaming(
    state: &AppState,
    request: &Value,
    upstream_model: &str,
) -> AppResult<WindsurfStreamResponse> {
    let api_key = auth::windsurf::api_key(state)?;
    let payload = build_get_chat_message_request(
        request,
        &api_key,
        DEFAULT_WINDSURF_VERSION,
        upstream_model,
    )?;
    let body = connect_envelope(&payload);
    let url = format!(
        "{}{}",
        state.runtime.windsurf_cloud_base_url.trim_end_matches('/'),
        GET_CHAT_MESSAGE_PATH
    );
    let client = state.specter.clone();
    match client
        .post(url)
        .header("content-type", "application/connect+proto")
        .header("connect-protocol-version", "1")
        .header("accept", "application/connect+proto")
        .body(body)
        .version(HttpVersion::Http2)
        .send_streaming()
        .await
    {
        Ok(response) => Ok(WindsurfStreamResponse::Streaming(response)),
        Err(error) if is_non_h2_streaming_error(&error) => {
            let response = send_chat_request(state, request, upstream_model).await?;
            Ok(WindsurfStreamResponse::Buffered(response))
        }
        Err(error) => Err(AppError::Upstream(format!(
            "Windsurf request failed: {error}"
        ))),
    }
}

pub fn build_get_chat_message_request(
    request: &Value,
    api_key: &str,
    version: &str,
    upstream_model: &str,
) -> AppResult<Vec<u8>> {
    let object = request
        .as_object()
        .ok_or_else(|| AppError::BadRequest("request body must be a JSON object".into()))?;
    let messages = object
        .get("messages")
        .and_then(Value::as_array)
        .ok_or_else(|| AppError::BadRequest("messages must be an array".into()))?;

    let mut out = Vec::with_capacity(512);
    let metadata = build_metadata(api_key, version);
    extend_message(&mut out, 1, &metadata);

    let mut prompt_count = 0;
    for message in messages {
        let prompt_text = text_content(message.get("content")).trim().to_string();
        let has_assistant_tool_calls = message.get("role").and_then(Value::as_str)
            == Some("assistant")
            && message
                .get("tool_calls")
                .is_some_and(|value| value.as_array().is_some_and(|items| !items.is_empty()));
        if prompt_text.is_empty() && !has_assistant_tool_calls {
            continue;
        }
        let prompt = build_chat_prompt(message);
        extend_message(&mut out, 3, &prompt);
        prompt_count += 1;
    }

    if prompt_count == 0 {
        return Err(AppError::BadRequest(
            "No prompt text found in request messages".into(),
        ));
    }

    extend_varint_field(&mut out, 7, 5);
    extend_string(&mut out, 21, upstream_model);
    Ok(out)
}

pub fn parse_complete_response(bytes: &[u8]) -> AppResult<String> {
    let mut buffer = bytes.to_vec();
    let chunks = drain_text_chunks(&mut buffer)?;
    if buffer.iter().any(|byte| !byte.is_ascii_whitespace()) {
        return Err(AppError::Upstream(
            "Windsurf response ended with an incomplete Connect frame".into(),
        ));
    }
    Ok(chunks.join(""))
}

pub fn connect_envelope(payload: &[u8]) -> Vec<u8> {
    let mut frame = Vec::with_capacity(payload.len() + 5);
    frame.push(0);
    frame.extend_from_slice(&(payload.len() as u32).to_be_bytes());
    frame.extend_from_slice(payload);
    frame
}

pub fn encode_string(field_num: u32, value: &str) -> Vec<u8> {
    let mut out = Vec::with_capacity(value.len() + 8);
    extend_string(&mut out, field_num, value);
    out
}

pub fn encode_message(field_num: u32, value: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(value.len() + 8);
    extend_message(&mut out, field_num, value);
    out
}

pub fn encode_varint_field(field_num: u32, value: u64) -> Vec<u8> {
    let mut out = Vec::with_capacity(16);
    extend_varint_field(&mut out, field_num, value);
    out
}

fn build_metadata(api_key: &str, version: &str) -> Vec<u8> {
    let mut out = Vec::with_capacity(api_key.len() + version.len() * 2 + 96);
    extend_string(&mut out, 3, api_key);
    extend_string(&mut out, 1, "windsurf");
    extend_string(&mut out, 7, version);
    extend_string(&mut out, 2, version);
    extend_string(&mut out, 12, "windsurf");
    extend_string(&mut out, 10, &Uuid::new_v4().to_string());
    extend_string(&mut out, 4, "en-US");
    extend_string(&mut out, 28, "windsurf");
    out
}

fn build_chat_prompt(message: &Value) -> Vec<u8> {
    let role = message
        .get("role")
        .and_then(Value::as_str)
        .unwrap_or("user");
    let mut prompt = text_content(message.get("content")).trim().to_string();
    if role == "assistant" {
        if let Some(tool_calls) = message.get("tool_calls") {
            if tool_calls.as_array().is_some_and(|items| !items.is_empty()) {
                let suffix = format!("ASSISTANT TOOL_CALLS: {tool_calls}");
                prompt = if prompt.is_empty() {
                    suffix
                } else {
                    format!("{prompt}\n{suffix}")
                };
            }
        }
    } else if role == "tool" {
        let call_id = message
            .get("tool_call_id")
            .and_then(Value::as_str)
            .map(|value| format!(" {value}"))
            .unwrap_or_default();
        prompt = format!("TOOL RESULT{call_id}: {prompt}");
    }

    let mut out = Vec::with_capacity(prompt.len() + 64);
    extend_string(&mut out, 1, &Uuid::new_v4().to_string());
    extend_varint_field(&mut out, 2, source_for_role(role));
    extend_string(&mut out, 3, &prompt);
    out
}

fn text_content(value: Option<&Value>) -> String {
    match value {
        Some(Value::String(text)) => text.clone(),
        Some(Value::Array(parts)) => {
            let mut out = String::new();
            for text in parts
                .iter()
                .filter(|part| part.get("type").and_then(Value::as_str) == Some("text"))
                .filter_map(|part| part.get("text").and_then(Value::as_str))
            {
                if !out.is_empty() {
                    out.push('\n');
                }
                out.push_str(text);
            }
            out
        }
        _ => String::new(),
    }
}

fn extend_string(out: &mut Vec<u8>, field_num: u32, value: &str) {
    let bytes = value.as_bytes();
    extend_key(out, field_num, 2);
    extend_varint(out, bytes.len() as u64);
    out.extend_from_slice(bytes);
}

fn extend_message(out: &mut Vec<u8>, field_num: u32, value: &[u8]) {
    extend_key(out, field_num, 2);
    extend_varint(out, value.len() as u64);
    out.extend_from_slice(value);
}

fn extend_varint_field(out: &mut Vec<u8>, field_num: u32, value: u64) {
    extend_key(out, field_num, 0);
    extend_varint(out, value);
}

fn extend_key(out: &mut Vec<u8>, field_num: u32, wire_type: u8) {
    extend_varint(out, ((field_num << 3) | u32::from(wire_type)) as u64);
}

fn extend_varint(out: &mut Vec<u8>, mut value: u64) {
    while value > 127 {
        out.push(((value & 0x7f) as u8) | 0x80);
        value >>= 7;
    }
    out.push(value as u8);
}

fn source_for_role(role: &str) -> u64 {
    match role {
        "system" => 5,
        "tool" => 4,
        _ => 1,
    }
}

fn drain_text_chunks(buffer: &mut Vec<u8>) -> AppResult<Vec<String>> {
    let mut chunks = Vec::new();
    let mut offset = 0;
    while offset + 5 <= buffer.len() {
        let flags = buffer[offset];
        let length = u32::from_be_bytes([
            buffer[offset + 1],
            buffer[offset + 2],
            buffer[offset + 3],
            buffer[offset + 4],
        ]) as usize;
        let frame_start = offset + 5;
        let frame_end = frame_start + length;
        if frame_end > buffer.len() {
            break;
        }
        let body = &buffer[frame_start..frame_end];
        chunks.extend(text_chunks_from_envelope(flags, body)?);
        offset = frame_end;
    }

    if offset > 0 {
        let remaining = buffer.split_off(offset);
        *buffer = remaining;
    }
    Ok(chunks)
}

fn text_chunks_from_envelope(flags: u8, body: &[u8]) -> AppResult<Vec<String>> {
    if flags == 2 {
        let trailer = String::from_utf8_lossy(body);
        if trailer.contains("\"error\"") {
            return Err(AppError::Upstream(format!(
                "Windsurf stream error: {trailer}"
            )));
        }
        return Ok(Vec::new());
    }

    let mut chunks = Vec::new();
    collect_text_fields(body, &mut chunks)?;
    Ok(chunks)
}

fn collect_text_fields(buffer: &[u8], chunks: &mut Vec<String>) -> AppResult<()> {
    let mut offset = 0;
    while offset < buffer.len() {
        let Some((key, next)) = read_varint(buffer, offset) else {
            break;
        };
        offset = next;
        let field_num = (key >> 3) as u32;
        let wire_type = (key & 7) as u8;
        match wire_type {
            0 => {
                let Some((_value, next)) = read_varint(buffer, offset) else {
                    break;
                };
                offset = next;
            }
            1 => {
                if offset + 8 > buffer.len() {
                    break;
                }
                offset += 8;
            }
            2 => {
                let Some((length, next)) = read_varint(buffer, offset) else {
                    break;
                };
                offset = next;
                let Some(end) = offset.checked_add(length as usize) else {
                    break;
                };
                if end > buffer.len() {
                    break;
                }
                if field_num == 3 {
                    let text = std::str::from_utf8(&buffer[offset..end]).map_err(|error| {
                        AppError::Upstream(format!("Windsurf text chunk was not UTF-8: {error}"))
                    })?;
                    chunks.push(text.to_owned());
                }
                offset = end;
            }
            5 => {
                if offset + 4 > buffer.len() {
                    break;
                }
                offset += 4;
            }
            _ => break,
        }
    }
    Ok(())
}

fn read_varint(buffer: &[u8], offset: usize) -> Option<(u64, usize)> {
    let mut value = 0u64;
    let mut shift = 0;
    let mut next = offset;
    while next < buffer.len() {
        let byte = buffer[next];
        value |= ((byte & 0x7f) as u64) << shift;
        next += 1;
        if byte < 0x80 {
            return Some((value, next));
        }
        shift += 7;
        if shift >= 64 {
            return None;
        }
    }
    None
}

struct StreamState {
    source: futures::stream::BoxStream<'static, AppResult<Bytes>>,
    buffer: Vec<u8>,
    pending: VecDeque<AppResult<String>>,
    finished: bool,
}

enum WindsurfStreamResponse {
    Buffered(specter::Response),
    Streaming(specter::Response),
}

fn is_non_h2_streaming_error(error: &specter::Error) -> bool {
    matches!(error, specter::Error::HttpProtocol(message) if message.contains("Expected h2 ALPN"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn connect_envelope_prefixes_flags_and_big_endian_length() {
        let frame = connect_envelope(b"abc");

        assert_eq!(&frame[..5], &[0, 0, 0, 0, 3]);
        assert_eq!(&frame[5..], b"abc");
    }

    #[test]
    fn get_chat_message_request_contains_prompts_metadata_and_model_uid() {
        let request = json!({
            "model": "swe-1.6",
            "messages": [
                { "role": "system", "content": "be terse" },
                { "role": "user", "content": "hello" },
                {
                    "role": "assistant",
                    "content": null,
                    "tool_calls": [{
                        "id": "call_1",
                        "type": "function",
                        "function": { "name": "lookup", "arguments": "{}" }
                    }]
                },
                { "role": "tool", "tool_call_id": "call_1", "content": "result" }
            ]
        });

        let body =
            build_get_chat_message_request(&request, "fake_windsurf_key", "1.13.104", "swe-1-6")
                .unwrap();
        let haystack = String::from_utf8_lossy(&body);

        assert!(haystack.contains("fake_windsurf_key"));
        assert!(haystack.contains("windsurf"));
        assert!(haystack.contains("be terse"));
        assert!(haystack.contains("hello"));
        assert!(haystack.contains("ASSISTANT TOOL_CALLS"));
        assert!(haystack.contains("TOOL RESULT call_1: result"));
        assert!(haystack.contains("swe-1-6"));
    }

    #[test]
    fn parses_connect_frames_into_text_chunks_and_trailer_errors() {
        let mut body = Vec::new();
        body.extend(encode_string(3, "hel"));
        let mut body2 = Vec::new();
        body2.extend(encode_string(3, "lo"));

        let mut response = Vec::new();
        response.extend(connect_envelope(&body));
        response.extend(connect_envelope(&body2));
        response.extend([2, 0, 0, 0, 2, b'o', b'k']);

        assert_eq!(parse_complete_response(&response).unwrap(), "hello");

        let trailer = br#"{"error":"bad"}"#;
        let mut error_frame = vec![2];
        error_frame.extend_from_slice(&(trailer.len() as u32).to_be_bytes());
        error_frame.extend_from_slice(trailer);
        let error = parse_complete_response(&error_frame).unwrap_err();
        assert!(error.to_string().contains("Windsurf stream error"));
    }

    #[test]
    fn incomplete_frame_is_rejected() {
        let error = parse_complete_response(&[0, 0, 0, 0, 10, b'a']).unwrap_err();

        assert!(error.to_string().contains("incomplete Connect frame"));
    }
}
