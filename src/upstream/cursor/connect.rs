//! Connect / gRPC-web framing helpers.
//!
//! Frame layout: `[flags: u8][len: u32_be][payload: len bytes]`.
//!
//! The encoder always writes the flags byte verbatim; today every outbound
//! frame uses `0x00`, but the parameter is kept open so future support for
//! the Connect compression bit (`0x01`) does not need a wire-shape change.
//! On the receive side the relevant flag bits the upstream module honors are
//! `0x02` (Connect end-stream) and `0x80` (gRPC-web trailer block).

use bytes::{Bytes, BytesMut};

/// Connect end-stream flag. Frame body is a JSON trailer per the Connect
/// streaming spec.
pub const CONNECT_END_STREAM_FLAG: u8 = 0b0000_0010;

/// gRPC-web trailer flag. Cursor still emits these for status/message
/// metadata even when running over Connect.
pub const GRPC_WEB_TRAILER_FLAG: u8 = 0b1000_0000;

/// Encode a single Connect frame: `[flags][be_u32 len][payload]`.
pub fn frame_connect_message(payload: &[u8], flags: u8) -> Vec<u8> {
    let mut out = Vec::with_capacity(payload.len() + 5);
    out.push(flags);
    let len = payload.len() as u32;
    out.extend_from_slice(&len.to_be_bytes());
    out.extend_from_slice(payload);
    out
}

/// Pop a single Connect frame off the front of `buf`. Returns `None` when
/// the buffer does not yet hold a complete header + payload, leaving the
/// pending bytes intact for the next read.
pub fn take_connect_frame(buf: &mut Vec<u8>) -> Option<(u8, Vec<u8>)> {
    let mut bytes = BytesMut::from(&buf[..]);
    let original_len = bytes.len();
    let (flags, payload) = take_connect_frame_bytes(&mut bytes)?;
    let consumed = original_len - bytes.len();
    buf.drain(..consumed);
    Some((flags, payload.to_vec()))
}

/// Pop a single Connect frame off the front of a `BytesMut` without shifting
/// the remaining buffer. Hot streaming paths should prefer this over the
/// `Vec<u8>` compatibility wrapper above.
pub fn take_connect_frame_bytes(buf: &mut BytesMut) -> Option<(u8, Bytes)> {
    if buf.len() < 5 {
        return None;
    }
    let len = u32::from_be_bytes([buf[1], buf[2], buf[3], buf[4]]) as usize;
    if buf.len() < 5 + len {
        return None;
    }
    let flags = buf[0];
    let _header = buf.split_to(5);
    let payload = buf.split_to(len).freeze();
    Some((flags, payload))
}

/// Decoded Connect end-stream error envelope.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConnectError {
    /// Connect error code string (e.g. `unauthenticated`, `resource_exhausted`).
    pub code: String,
    /// Human-readable error message supplied by the upstream.
    pub message: String,
}

/// Parse a Connect end-stream payload as JSON. Returns `Some(ConnectError)`
/// when the body carries an `error` object, `None` when the close was clean
/// (`{}` or any payload without an `error` field).
///
/// Malformed JSON is mapped to `Some(ConnectError { code = "internal", message
/// = "<utf8 fragment>" })` so callers can surface the raw bytes instead of
/// silently dropping a non-empty terminal frame.
pub fn parse_connect_end_stream(payload: &[u8]) -> Option<ConnectError> {
    if payload.is_empty() {
        return None;
    }
    match serde_json::from_slice::<serde_json::Value>(payload) {
        Ok(value) => {
            let error = value.get("error")?;
            let code = error
                .get("code")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string();
            let message = error
                .get("message")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown error")
                .to_string();
            Some(ConnectError { code, message })
        }
        Err(_) => Some(ConnectError {
            code: "internal".to_string(),
            message: String::from_utf8_lossy(payload).into_owned(),
        }),
    }
}
