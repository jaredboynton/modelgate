//! Primitives and helpers for custom protobuf wire format serialization and deserialization.

/// Encode a 64-bit unsigned varint per the protobuf wire format.
pub fn encode_varint(mut value: u64) -> Vec<u8> {
    let mut bytes = Vec::new();
    while value > 127 {
        bytes.push(((value & 0x7f) as u8) | 0x80);
        value >>= 7;
    }
    bytes.push(value as u8);
    bytes
}

/// Decode a varint at `offset`, returning `(value, bytes_consumed)`.
///
/// Returns `None` on truncated or oversized input. The 10-byte cap matches
/// the upper bound for a 64-bit varint and prevents the parser from running
/// off the end of an attacker-supplied chunk.
pub fn decode_varint(data: &[u8], offset: usize) -> Option<(u64, usize)> {
    let mut value = 0u64;
    let mut shift = 0u32;
    for (index, byte) in data.get(offset..)?.iter().enumerate() {
        value |= u64::from(byte & 0x7f) << shift;
        if byte & 0x80 == 0 {
            return Some((value, index + 1));
        }
        shift += 7;
        if shift >= 64 {
            return None;
        }
    }
    None
}

/// Encode a length-delimited string field (wire type 2).
///
/// Short-circuits on empty input per proto3 default semantics.
pub fn encode_string_field(field_number: u32, value: &str) -> Vec<u8> {
    if value.is_empty() {
        return Vec::new();
    }
    encode_string_field_always(field_number, value)
}

pub fn encode_string_field_always(field_number: u32, value: &str) -> Vec<u8> {
    let mut out = encode_varint(((field_number << 3) | 2) as u64);
    out.extend(encode_varint(value.len() as u64));
    out.extend_from_slice(value.as_bytes());
    out
}

/// Encode a length-delimited message/bytes field (wire type 2).
///
/// Always written, even when the body is empty: callers explicitly use this
/// to send a present-but-empty sub-message (e.g. empty `ConversationStateStructure`).
pub fn encode_message_field(field_number: u32, data: &[u8]) -> Vec<u8> {
    let mut out = encode_varint(((field_number << 3) | 2) as u64);
    out.extend(encode_varint(data.len() as u64));
    out.extend_from_slice(data);
    out
}

/// Encode an int32 / uint32 / bool field (wire type 0).
///
/// Short-circuits on zero per proto3 default semantics.
pub fn encode_int32_field(field_number: u32, value: u32) -> Vec<u8> {
    if value == 0 {
        return Vec::new();
    }
    let mut out = encode_varint((field_number << 3) as u64);
    out.extend(encode_varint(value as u64));
    out
}

/// Encode an int64 / uint64 field (wire type 0).
///
/// Does NOT short-circuit on zero. kv/exec response builders need field 1
/// to be present even when id == 0.
pub fn encode_int64_field(field_number: u32, value: u64) -> Vec<u8> {
    let mut out = encode_varint((field_number << 3) as u64);
    out.extend(encode_varint(value));
    out
}

/// Encode a bool field (wire type 0). Mirrors proto3 default semantics —
/// `false` short-circuits to an empty byte slice.
pub fn encode_bool_field(field_number: u32, value: bool) -> Vec<u8> {
    if !value {
        return Vec::new();
    }
    let mut out = encode_varint((field_number << 3) as u64);
    out.push(1);
    out
}

pub fn encode_varint_field_always(field_number: u32, value: u64) -> Vec<u8> {
    let mut out = encode_varint((field_number << 3) as u64);
    out.extend(encode_varint(value));
    out
}

pub fn encode_double_field_always(field_number: u32, value: f64) -> Vec<u8> {
    let mut out = encode_varint(((field_number << 3) | 1) as u64);
    out.extend_from_slice(&value.to_le_bytes());
    out
}

pub fn encode_bool_field_always(field_number: u32, value: bool) -> Vec<u8> {
    encode_varint_field_always(field_number, u64::from(value))
}

/// Encode a `repeated string` proto3 field as unpacked length-delimited
/// entries (one tag/length pair per element).
pub fn encode_repeated_string_field(field_number: u32, values: &[String]) -> Vec<u8> {
    let mut out = Vec::new();
    for v in values {
        if v.is_empty() {
            // proto3 default; emit nothing rather than a zero-length string.
            continue;
        }
        out.extend_from_slice(&encode_string_field(field_number, v));
    }
    out
}

/// Encode a `repeated message` field as unpacked length-delimited entries.
pub fn encode_repeated_message_field(field_number: u32, items: &[Vec<u8>]) -> Vec<u8> {
    let mut out = Vec::new();
    for item in items {
        out.extend_from_slice(&encode_message_field(field_number, item));
    }
    out
}

/// Encode a `repeated uint64` as packed field
pub fn encode_packed_varint_field(field_number: u32, values: &[u64]) -> Vec<u8> {
    if values.is_empty() {
        return Vec::new();
    }
    let mut payload = Vec::new();
    for &val in values {
        payload.extend(encode_varint(val));
    }
    let mut out = encode_varint(((field_number << 3) | 2) as u64);
    out.extend(encode_varint(payload.len() as u64));
    out.extend(payload);
    out
}

/// Concatenate a list of byte chunks into a single buffer. Sum-allocate so we
/// never pay for incremental Vec growth.
pub fn concat_bytes(chunks: &[Vec<u8>]) -> Vec<u8> {
    let total: usize = chunks.iter().map(Vec::len).sum();
    let mut out = Vec::with_capacity(total);
    for chunk in chunks {
        out.extend_from_slice(chunk);
    }
    out
}

/// Decoded protobuf field. Wire-type 0 values are re-encoded as varint bytes
/// inside `value` so consumers stay uniform; call `decode_varint` again to
/// recover the integer.
#[derive(Debug, Clone)]
pub struct ProtoField {
    pub number: u32,
    pub wire_type: u8,
    pub value: Vec<u8>,
}

/// Walk a length-delimited body and return all decoded fields. Truncated or
/// malformed tails stop parsing early instead of panicking; partial frames
/// are filtered upstream by the Connect framing layer.
pub fn parse_proto_fields(data: &[u8]) -> Vec<ProtoField> {
    let mut fields = Vec::new();
    let mut offset = 0usize;
    while offset < data.len() {
        let Some((tag, tag_len)) = decode_varint(data, offset) else {
            break;
        };
        offset += tag_len;
        let number = (tag >> 3) as u32;
        let wire_type = (tag & 7) as u8;
        match wire_type {
            0 => {
                let Some((value, len)) = decode_varint(data, offset) else {
                    break;
                };
                offset += len;
                fields.push(ProtoField {
                    number,
                    wire_type,
                    value: encode_varint(value),
                });
            }
            2 => {
                let Some((len, len_len)) = decode_varint(data, offset) else {
                    break;
                };
                offset += len_len;
                let end = offset.saturating_add(len as usize);
                if end > data.len() {
                    break;
                }
                fields.push(ProtoField {
                    number,
                    wire_type,
                    value: data[offset..end].to_vec(),
                });
                offset = end;
            }
            1 => {
                let end = offset.saturating_add(8);
                if end > data.len() {
                    break;
                }
                fields.push(ProtoField {
                    number,
                    wire_type,
                    value: data[offset..end].to_vec(),
                });
                offset = end;
            }
            5 => {
                let end = offset.saturating_add(4);
                if end > data.len() {
                    break;
                }
                fields.push(ProtoField {
                    number,
                    wire_type,
                    value: data[offset..end].to_vec(),
                });
                offset = end;
            }
            _ => break,
        }
    }
    fields
}
