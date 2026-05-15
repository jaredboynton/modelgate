//! Shared protobuf field readers used by the per-profile renderers.
//!
//! `proto::decode_string_field` and `proto::decode_u64_field` are private to
//! the proto module; the profile renderers all need the same readers, so the
//! shared copy lives here next to its callers.

use crate::upstream::cursor::proto::{decode_varint, parse_proto_fields};

pub(super) fn read_string_field(data: &[u8], field_number: u32) -> Option<String> {
    parse_proto_fields(data)
        .into_iter()
        .find(|field| field.number == field_number && field.wire_type == 2)
        .map(|field| String::from_utf8_lossy(&field.value).into_owned())
}

pub(super) fn read_u64_field(data: &[u8], field_number: u32) -> Option<u64> {
    let field = parse_proto_fields(data)
        .into_iter()
        .find(|field| field.number == field_number && field.wire_type == 0)?;
    decode_varint(&field.value, 0).map(|(value, _)| value)
}
