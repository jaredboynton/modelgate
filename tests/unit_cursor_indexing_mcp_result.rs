//! Regression tests for the MCP exec-result wire encoder used by the
//! legacy `cursor_codebase_search` MCP result encoder.
//!
//! These are structural tests that pin the wire shape produced by
//! `upstream::cursor::indexing::encode_cursor_codebase_search_mcp_result`.
//! They do not claim Cursor accepts the bytes — that's Phase 0 live
//! validation's job. The point is regression coverage: the encoder is
//! retained for wire compatibility; any drift in the field tags or varint
//! encoding will silently break callers that still need to send an MCP result
//! frame.

use unified_model_proxy_v2::upstream::cursor::indexing::encode_cursor_codebase_search_mcp_result;
use unified_model_proxy_v2::upstream::cursor::proto::{
    agent_client_message, exec_message, parse_proto_fields,
};

#[test]
fn mcp_result_encoder_wraps_body_as_text_content() {
    let body = "search results: file.rs:42 fn matches()";
    let encoded = encode_cursor_codebase_search_mcp_result(42, "exec_abc", body);

    // Top-level: `AgentClientMessage.exec_client_message` (field 2,
    // wire type 2). Every encoded payload must carry exactly this
    // wrapper at the outermost level so the Cursor server can demux it.
    let outer = parse_proto_fields(&encoded);
    assert!(!outer.is_empty(), "encoded payload must not be empty");
    let exec_client_field = outer
        .iter()
        .find(|f| f.number == agent_client_message::EXEC_CLIENT_MESSAGE)
        .expect("encoded payload wraps ExecClientMessage at agent_client_message field 2");
    assert_eq!(exec_client_field.wire_type, 2);

    // The body string must survive the encode end-to-end as the
    // McpTextContent.text bytes. Search the raw payload because each
    // length-delimited wrapper adds its own header.
    assert!(
        encoded.windows(body.len()).any(|w| w == body.as_bytes()),
        "body string preserved in encoded payload",
    );
}

#[test]
fn mcp_result_encoder_includes_cursor_exec_id_string_field() {
    let exec_id = "test-exec-id-42";
    let encoded = encode_cursor_codebase_search_mcp_result(7, exec_id, "ok");
    assert!(
        encoded
            .windows(exec_id.len())
            .any(|w| w == exec_id.as_bytes()),
        "cursor exec_id string preserved in encoded payload",
    );

    // Drill into the ExecClientMessage to confirm the EXEC_ID tag carries
    // the string at field 15 (the canonical exec_id position).
    let outer = parse_proto_fields(&encoded);
    let exec_client = outer
        .iter()
        .find(|f| f.number == agent_client_message::EXEC_CLIENT_MESSAGE)
        .expect("ExecClientMessage wrapper present");
    let inner = parse_proto_fields(&exec_client.value);
    let exec_id_field = inner
        .iter()
        .find(|f| f.number == exec_message::EXEC_ID)
        .expect("ExecClientMessage carries exec_id field 15");
    assert_eq!(exec_id_field.wire_type, 2);
    assert_eq!(
        std::str::from_utf8(&exec_id_field.value).unwrap_or(""),
        exec_id,
    );
}

#[test]
fn mcp_result_encoder_emits_int_id_varint_field() {
    // The numeric `id` field is encoded as a proto varint. id=255 is
    // 0xff 0x01 in varint form. Any drift in the varint helpers (or in
    // the wire-type chosen for ID) would corrupt this byte sequence.
    let encoded = encode_cursor_codebase_search_mcp_result(255, "exec", "body");
    assert!(
        encoded.windows(2).any(|w| w == [0xff, 0x01]),
        "varint encoding of id=255 (0xff 0x01) present in payload",
    );

    // And the ID field is reachable inside the ExecClientMessage at
    // field 1, wire type 0.
    let outer = parse_proto_fields(&encoded);
    let exec_client = outer
        .iter()
        .find(|f| f.number == agent_client_message::EXEC_CLIENT_MESSAGE)
        .expect("ExecClientMessage wrapper present");
    let inner = parse_proto_fields(&exec_client.value);
    let id_field = inner
        .iter()
        .find(|f| f.number == exec_message::ID)
        .expect("ExecClientMessage carries id field 1");
    assert_eq!(id_field.wire_type, 0);
}

#[test]
fn mcp_result_encoder_uses_mcp_args_field_for_result_payload() {
    // Cursor reuses the MCP_ARGS field number (11) on the request and
    // response sides of the oneof. The result sits under that field as
    // a serialized `McpResult` body. Lane K coordination: if the wire
    // schema ever splits MCP into separate request/response oneof tags,
    // this test should be split into two: one for the request side
    // (already covered by `decode_mcp_args` in proto.rs) and one for
    // the result encoder we exercise here.
    let encoded = encode_cursor_codebase_search_mcp_result(1, "exec", "body");
    let outer = parse_proto_fields(&encoded);
    let exec_client = outer
        .iter()
        .find(|f| f.number == agent_client_message::EXEC_CLIENT_MESSAGE)
        .expect("ExecClientMessage wrapper present");
    let inner = parse_proto_fields(&exec_client.value);
    let mcp_field = inner
        .iter()
        .find(|f| f.number == exec_message::MCP_ARGS)
        .expect("result body lives under exec_message::MCP_ARGS (field 11)");
    assert_eq!(mcp_field.wire_type, 2);
    assert!(
        !mcp_field.value.is_empty(),
        "MCP result body must not be empty for non-empty body input",
    );
}

#[test]
fn mcp_result_encoder_with_empty_body_still_produces_valid_envelope() {
    // Empty body must not collapse the entire payload to nothing. The
    // ExecClientMessage envelope still ships, the exec_id is preserved,
    // and the McpResult body is present (even if its inner text is the
    // proto3 default).
    let encoded = encode_cursor_codebase_search_mcp_result(0, "exec-empty", "");
    let outer = parse_proto_fields(&encoded);
    assert!(
        outer
            .iter()
            .any(|f| f.number == agent_client_message::EXEC_CLIENT_MESSAGE),
        "ExecClientMessage envelope present even with empty body",
    );
    assert!(
        encoded
            .windows("exec-empty".len())
            .any(|w| w == b"exec-empty"),
        "exec_id survives encoding regardless of body length",
    );
}
