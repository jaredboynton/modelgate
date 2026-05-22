use bytes::Bytes;
use serde_json::Value;

use crate::{AppError, AppResult};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ResponsesSseFrame {
    pub event: Option<String>,
    pub data: Value,
}

#[derive(Debug, Default)]
pub struct ResponsesSseParser {
    buffer: Vec<u8>,
    saw_completed: bool,
}

impl ResponsesSseParser {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push_bytes(&mut self, bytes: Bytes) -> AppResult<Vec<ResponsesSseFrame>> {
        self.buffer.extend_from_slice(&bytes);
        let mut frames = Vec::new();

        while let Some(end) = next_event_end(&self.buffer) {
            let event = self.buffer.drain(..end.consumed).collect::<Vec<_>>();
            let event = &event[..end.event_len];
            if let Some(frame) = self.parse_event(event)? {
                if is_responses_terminal_event(&frame.data) {
                    self.saw_completed = true;
                }
                frames.push(frame);
            }
        }

        Ok(frames)
    }

    pub fn finish(&mut self) -> AppResult<Vec<ResponsesSseFrame>> {
        let mut frames = Vec::new();
        if !self.buffer.iter().all(|byte| byte.is_ascii_whitespace()) {
            let remaining = std::mem::take(&mut self.buffer);
            if let Some(frame) = self.parse_event(&remaining)? {
                if is_responses_terminal_event(&frame.data) {
                    self.saw_completed = true;
                }
                frames.push(frame);
            }
        }
        if self.saw_completed {
            Ok(frames)
        } else {
            Err(AppError::Upstream(
                "Responses SSE ended before terminal response event".into(),
            ))
        }
    }

    fn parse_event(&self, event: &[u8]) -> AppResult<Option<ResponsesSseFrame>> {
        let event = std::str::from_utf8(event)
            .map_err(|error| AppError::Upstream(format!("invalid Responses SSE UTF-8: {error}")))?;
        let mut event_name = None;
        let mut data = String::new();

        for raw_line in event.lines() {
            let line = raw_line.strip_suffix('\r').unwrap_or(raw_line);
            if line.is_empty() || line.starts_with(':') {
                continue;
            }
            let Some((field, raw_value)) = line.split_once(':') else {
                continue;
            };
            let value = raw_value.strip_prefix(' ').unwrap_or(raw_value);
            match field {
                "event" => event_name = Some(value.to_string()),
                "data" => {
                    if !data.is_empty() {
                        data.push('\n');
                    }
                    data.push_str(value);
                }
                _ => {}
            }
        }

        if data.is_empty() {
            return Ok(None);
        }

        if data == "[DONE]" {
            if self.saw_completed {
                return Ok(None);
            }
            return Err(AppError::Upstream(
                "Responses SSE sent [DONE] before response.completed".into(),
            ));
        }

        let data = serde_json::from_str(&data)?;
        Ok(Some(ResponsesSseFrame {
            event: event_name,
            data,
        }))
    }
}

fn is_responses_terminal_event(value: &Value) -> bool {
    matches!(
        value.get("type").and_then(Value::as_str),
        Some("response.completed" | "response.failed" | "response.incomplete")
    )
}

#[derive(Debug, Clone, Copy)]
struct EventEnd {
    event_len: usize,
    consumed: usize,
}

fn next_event_end(bytes: &[u8]) -> Option<EventEnd> {
    for index in 0..bytes.len() {
        match bytes.get(index..) {
            Some(rest) if rest.starts_with(b"\n\n") => {
                return Some(EventEnd {
                    event_len: index,
                    consumed: index + 2,
                });
            }
            Some(rest) if rest.starts_with(b"\r\n\r\n") => {
                return Some(EventEnd {
                    event_len: index,
                    consumed: index + 4,
                });
            }
            Some(rest) if rest.starts_with(b"\n\r\n") => {
                return Some(EventEnd {
                    event_len: index,
                    consumed: index + 3,
                });
            }
            _ => {}
        }
    }
    None
}
