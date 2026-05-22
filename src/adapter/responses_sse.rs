use bytes::{Buf, Bytes, BytesMut};
use serde_json::Value;

use crate::{AppError, AppResult};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ResponsesSseFrame {
    pub event: Option<String>,
    pub data: Value,
}

#[derive(Debug, Default)]
pub struct ResponsesSseParser {
    buffer: BytesMut,
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
            if let Some(frame) = self.parse_event(&self.buffer[..end.event_len])? {
                if is_responses_terminal_event(&frame.data) {
                    self.saw_completed = true;
                }
                frames.push(frame);
            }
            self.buffer.advance(end.consumed);
        }

        Ok(frames)
    }

    pub fn finish(&mut self) -> AppResult<Vec<ResponsesSseFrame>> {
        let mut frames = Vec::new();
        if !self.buffer.iter().all(|byte| byte.is_ascii_whitespace()) {
            let remaining = self.buffer.split().freeze();
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
        let mut event_name = None;
        let mut data = DataField::default();

        for raw_line in event.split(|byte| *byte == b'\n') {
            let line = raw_line.strip_suffix(b"\r").unwrap_or(raw_line);
            if line.is_empty() || line.starts_with(b":") {
                continue;
            }
            let Some(colon) = line.iter().position(|byte| *byte == b':') else {
                continue;
            };
            let field = &line[..colon];
            let raw_value = &line[colon + 1..];
            let value = raw_value.strip_prefix(b" ").unwrap_or(raw_value);
            match field {
                b"event" => {
                    let value = std::str::from_utf8(value).map_err(|error| {
                        AppError::Upstream(format!("invalid Responses SSE UTF-8: {error}"))
                    })?;
                    event_name = Some(value.to_string());
                }
                b"data" => data.push(value),
                _ => {}
            }
        }

        let Some(data) = data.into_bytes() else {
            return Ok(None);
        };

        if data.as_ref() == b"[DONE]" {
            if self.saw_completed {
                return Ok(None);
            }
            return Err(AppError::Upstream(
                "Responses SSE sent [DONE] before response.completed".into(),
            ));
        }

        std::str::from_utf8(data.as_ref())
            .map_err(|error| AppError::Upstream(format!("invalid Responses SSE UTF-8: {error}")))?;
        let data = serde_json::from_slice(data.as_ref())?;
        Ok(Some(ResponsesSseFrame {
            event: event_name,
            data,
        }))
    }
}

#[derive(Default)]
enum DataField<'a> {
    #[default]
    Empty,
    Borrowed(&'a [u8]),
    Owned(Vec<u8>),
}

impl<'a> DataField<'a> {
    fn push(&mut self, value: &'a [u8]) {
        match self {
            Self::Empty => *self = Self::Borrowed(value),
            Self::Borrowed(first) => {
                let mut data = Vec::with_capacity(first.len() + 1 + value.len());
                data.extend_from_slice(first);
                data.push(b'\n');
                data.extend_from_slice(value);
                *self = Self::Owned(data);
            }
            Self::Owned(data) => {
                data.push(b'\n');
                data.extend_from_slice(value);
            }
        }
    }

    fn into_bytes(self) -> Option<std::borrow::Cow<'a, [u8]>> {
        match self {
            Self::Empty => None,
            Self::Borrowed(data) => Some(std::borrow::Cow::Borrowed(data)),
            Self::Owned(data) => Some(std::borrow::Cow::Owned(data)),
        }
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
