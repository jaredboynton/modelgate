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

pub fn extend_string(out: &mut Vec<u8>, field_num: u32, value: &str) {
    let bytes = value.as_bytes();
    extend_key(out, field_num, 2);
    extend_varint(out, bytes.len() as u64);
    out.extend_from_slice(bytes);
}

pub fn extend_message(out: &mut Vec<u8>, field_num: u32, value: &[u8]) {
    extend_key(out, field_num, 2);
    extend_varint(out, value.len() as u64);
    out.extend_from_slice(value);
}

pub fn extend_varint_field(out: &mut Vec<u8>, field_num: u32, value: u64) {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn connect_envelope_prefixes_flags_and_big_endian_length() {
        let frame = connect_envelope(b"abc");

        assert_eq!(&frame[..5], &[0, 0, 0, 0, 3]);
        assert_eq!(&frame[5..], b"abc");
    }

    #[test]
    fn encodes_length_delimited_and_varint_fields() {
        assert_eq!(encode_string(3, "hi"), vec![26, 2, b'h', b'i']);
        assert_eq!(encode_message(1, &[8, 5]), vec![10, 2, 8, 5]);
        assert_eq!(encode_varint_field(7, 150), vec![56, 150, 1]);
    }
}
