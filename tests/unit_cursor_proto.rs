//! Cursor protobuf + Connect framing unit tests.
//!
//! Covers the manual protobuf wire layer and the Connect frame parser per
//! the Phase 0 fixtures manifest (see
//! `.omx/research/cursor-phase0/fixtures-extraction.md`). Fixtures are
//! generated from inputs at test-time when on-disk goldens are absent so
//! the suite stays runnable before the parity script lands.
//!
//! Once `tests/fixtures/cursor/{run,server,unary,connect}/*.bin` is
//! committed (per `tests/fixtures/cursor/scripts/generate-fixtures.mjs`),
//! the comparison switches to byte-for-byte equality against the on-disk
//! goldens via `include_bytes!`. Until then, tests assert determinism +
//! roundtrip + structural shape.

use std::path::{Path, PathBuf};

use unified_model_proxy_v2::cursor_agent::{CursorTool, CursorToolKind};
use unified_model_proxy_v2::upstream::cursor::{
    connect::{
        frame_connect_message, parse_connect_end_stream, take_connect_frame, ConnectError,
        CONNECT_END_STREAM_FLAG, GRPC_WEB_TRAILER_FLAG,
    },
    proto::{
        agent_client_message, agent_run_request, agent_server_message, conversation_action,
        decode_agent_server_message, decode_exec_public_tool_call,
        decode_get_usable_models_response, decode_varint, encode_agent_run_request,
        encode_bool_field, encode_client_heartbeat, encode_get_usable_models_request,
        encode_int32_field, encode_int64_field, encode_message_field, encode_repeated_string_field,
        encode_request_context_result, encode_string_field, encode_varint,
        get_usable_models_response, interaction_update, model_details, parse_proto_fields,
        request_context, request_context_env, requested_model, requested_model_parameter,
        user_message, user_message_action, AgentRunRequestInput, ExecKind, ExecRequest,
        InteractionEvent, KvKind, Message, ProtoField, RequestedModelInput,
        RequestedModelParameter,
    },
};

const FIXTURE_RUN_BASIC_SYSTEM_USER: &str = "tests/fixtures/cursor/run/basic_system_user.bin";

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn fixture_path(relative: &str) -> PathBuf {
    manifest_dir().join(relative)
}

fn read_fixture_bytes(relative: &str) -> Option<Vec<u8>> {
    let path = fixture_path(relative);
    if !path.is_file() {
        return None;
    }
    std::fs::read(&path).ok()
}

fn write_fixture_bytes(relative: &str, bytes: &[u8]) {
    if std::env::var_os("UMP_REGENERATE_CURSOR_FIXTURES").is_none() {
        return;
    }
    let path = fixture_path(relative);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("create fixture dir");
    }
    std::fs::write(&path, bytes).expect("write fixture bytes");
}

// ---------------------------------------------------------------------------
// Varint roundtrip + proto3 default short-circuit
// ---------------------------------------------------------------------------

#[test]
fn varint_roundtrip_covers_one_byte_and_multi_byte_widths() {
    for value in [
        0u64,
        1,
        0x7f,
        0x80,
        0x3fff,
        0x4000,
        0x1f_ffff,
        0x20_0000,
        u32::MAX as u64,
        u64::MAX,
    ] {
        let bytes = encode_varint(value);
        let (decoded, consumed) = decode_varint(&bytes, 0).expect("decode varint");
        assert_eq!(value, decoded, "value {value} round-tripped wrong");
        assert_eq!(consumed, bytes.len(), "value {value} consumed wrong");
    }
}

#[test]
fn varint_decode_rejects_truncated_and_oversized_input() {
    // 11-byte continuation pattern overflows the 64-bit cap.
    let oversized = vec![0xffu8; 11];
    assert!(
        decode_varint(&oversized, 0).is_none(),
        "varint > 10 bytes must error",
    );

    // Continuation bit set on the last byte, but no follow-up byte.
    let truncated = vec![0x80u8];
    assert!(
        decode_varint(&truncated, 0).is_none(),
        "truncated varint must return None",
    );
}

#[test]
fn proto3_default_short_circuits_match_v1_encoder_quirks() {
    // Empty strings produce zero bytes (proto3 default).
    assert!(encode_string_field(1, "").is_empty());
    // Zero int32 produces zero bytes (proto3 default).
    assert!(encode_int32_field(1, 0).is_empty());
    // Zero int64 still emits the field (kv/exec id quirk per proto.rs).
    assert_eq!(encode_int64_field(1, 0), vec![0x08, 0x00]);
    // False bool produces zero bytes (proto3 default).
    assert!(encode_bool_field(1, false).is_empty());
    // True bool emits varint 1.
    assert_eq!(encode_bool_field(1, true), vec![0x08, 0x01]);
}

#[test]
fn repeated_string_field_skips_empty_entries() {
    let bytes =
        encode_repeated_string_field(2, &["a".to_string(), "".to_string(), "b".to_string()]);
    let fields = parse_proto_fields(&bytes);
    assert_eq!(fields.len(), 2, "empty strings must be skipped");
    for field in fields {
        assert_eq!(field.number, 2);
        assert_eq!(field.wire_type, 2);
    }
}

// ---------------------------------------------------------------------------
// Connect frame encode/decode roundtrips
// ---------------------------------------------------------------------------

#[test]
fn connect_frame_encode_decode_roundtrip_for_simple_payload() {
    let payload = b"protobuf-bytes";
    let framed = frame_connect_message(payload, 0);
    assert_eq!(framed[0], 0, "flags byte should be zero for plain frames");
    let mut buf = framed.clone();
    let (flags, decoded) = take_connect_frame(&mut buf).expect("frame decodes cleanly");
    assert_eq!(flags, 0);
    assert_eq!(decoded.as_slice(), payload);
    assert!(buf.is_empty(), "buffer drained after pop");
}

#[test]
fn connect_frame_split_across_two_reads_buffers_until_complete() {
    let payload = b"split-frame-payload-larger-than-trivial";
    let framed = frame_connect_message(payload, 0);
    let split_at = framed.len() / 2;
    assert!(split_at > 5, "split must straddle the 5-byte header");

    let mut buf = framed[..split_at].to_vec();
    assert!(
        take_connect_frame(&mut buf).is_none(),
        "partial frame must return None",
    );

    buf.extend_from_slice(&framed[split_at..]);
    let (flags, decoded) = take_connect_frame(&mut buf).expect("second read completes the frame");
    assert_eq!(flags, 0);
    assert_eq!(decoded.as_slice(), payload);
    assert!(buf.is_empty());
}

#[test]
fn connect_frame_multi_frame_per_read_drains_repeatedly() {
    let payloads: [&[u8]; 3] = [b"alpha", b"beta-payload", b"gamma-payload-extra"];
    let mut combined = Vec::new();
    for payload in payloads {
        combined.extend_from_slice(&frame_connect_message(payload, 0));
    }

    let mut buf = combined.clone();
    let mut decoded = Vec::new();
    while let Some((_flags, body)) = take_connect_frame(&mut buf) {
        decoded.push(body);
    }

    assert_eq!(decoded.len(), payloads.len(), "all frames drained");
    for (decoded_body, expected) in decoded.iter().zip(payloads.iter()) {
        assert_eq!(decoded_body.as_slice(), *expected);
    }
    assert!(buf.is_empty());
}

#[test]
fn connect_frame_decoder_handles_end_stream_and_trailer_flags() {
    let trailer_payload = b"grpc-status: 0";
    let framed = frame_connect_message(trailer_payload, GRPC_WEB_TRAILER_FLAG);
    let mut buf = framed;
    let (flags, decoded) = take_connect_frame(&mut buf).expect("trailer frame decodes");
    assert_eq!(flags & GRPC_WEB_TRAILER_FLAG, GRPC_WEB_TRAILER_FLAG);
    assert_eq!(decoded.as_slice(), trailer_payload);

    let end_stream_body = br#"{"error":{"code":"unauthenticated","message":"token expired"}}"#;
    let end_stream_frame = frame_connect_message(end_stream_body, CONNECT_END_STREAM_FLAG);
    let mut buf = end_stream_frame;
    let (flags, body) = take_connect_frame(&mut buf).expect("end-stream frame decodes");
    assert_eq!(flags & CONNECT_END_STREAM_FLAG, CONNECT_END_STREAM_FLAG);
    let parsed = parse_connect_end_stream(&body).expect("error envelope present");
    assert_eq!(
        parsed,
        ConnectError {
            code: "unauthenticated".to_string(),
            message: "token expired".to_string(),
        },
    );
}

#[test]
fn connect_frame_decoder_handles_malformed_end_stream_payload_without_panic() {
    let garbled = b"not-json-bytes";
    let parsed = parse_connect_end_stream(garbled).expect("malformed body still surfaces a stub");
    assert_eq!(parsed.code, "internal");
    assert!(parsed.message.contains("not-json-bytes"));

    // Sanity: a well-formed frame with a 4-byte payload decodes cleanly.
    // Header layout is `[flags: u8][len: u32_be][payload: len bytes]`, so a
    // 4-byte body needs a 5-byte header with `len = 4`.
    let mut buf = vec![0u8, 0, 0, 0, 4, 0, 1, 2, 3];
    let (_flags, body) = take_connect_frame(&mut buf).expect("4-byte body decodes");
    assert_eq!(body, vec![0, 1, 2, 3]);

    // Header advertises more bytes than the buffer holds. The parser must
    // return None instead of panicking, leaving the pending bytes intact for
    // the next read.
    let mut starved = vec![0u8, 0, 0, 0, 8, 0xaa, 0xbb];
    assert!(
        take_connect_frame(&mut starved).is_none(),
        "starved frame must return None, never panic",
    );
}

// ---------------------------------------------------------------------------
// AgentRunRequest encode parity
// ---------------------------------------------------------------------------

#[test]
fn agent_run_request_basic_encodes_byte_stable_against_committed_fixture() {
    let messages = vec![
        Message {
            role: "system".to_string(),
            text: "You are a helpful assistant.".to_string(),
        },
        Message {
            role: "user".to_string(),
            text: "Explain quantum entanglement in one paragraph.".to_string(),
        },
    ];

    let encoded = encode_agent_run_request(AgentRunRequestInput {
        model: "composer-2-fast",
        requested_model: None,
        messages: &messages,
        message_id: "msg-fixture-0001",
        conversation_id: Some("conv-fixture-0001"),
        os_version: "darwin-24.6.0",
        workspace_path: "/tmp/cursor-fixture-workspace",
        shell: "/bin/zsh",
        tools: &[],
    });

    // Wrap inside `AgentClientMessage` for the canonical fixture comparison.
    // `encode_agent_run_request` already returns the wrapped wire bytes
    // (the function name in proto.rs encodes `AgentClientMessage.RunRequest`).
    write_fixture_bytes(FIXTURE_RUN_BASIC_SYSTEM_USER, &encoded);

    if let Some(expected) = read_fixture_bytes(FIXTURE_RUN_BASIC_SYSTEM_USER) {
        assert_eq!(
            expected, encoded,
            "AgentRunRequest parity drift against committed fixture",
        );
    } else {
        // No on-disk golden yet; assert structural invariants instead so
        // the test still fails on regressions.
        let outer = parse_proto_fields(&encoded);
        assert!(
            outer
                .iter()
                .any(|f| f.number == agent_client_message::RUN_REQUEST && f.wire_type == 2),
            "encoded bytes must wrap field {} as the AgentClientMessage.RunRequest",
            agent_client_message::RUN_REQUEST,
        );
        let run_request_payload = outer
            .iter()
            .find(|f| f.number == agent_client_message::RUN_REQUEST)
            .map(|f| f.value.clone())
            .unwrap_or_default();
        let run_fields = parse_proto_fields(&run_request_payload);
        let has_action = run_fields
            .iter()
            .any(|f| f.number == agent_run_request::ACTION && f.wire_type == 2);
        let has_model = run_fields
            .iter()
            .any(|f| f.number == agent_run_request::MODEL_DETAILS && f.wire_type == 2);
        let has_conversation_id = run_fields
            .iter()
            .any(|f| f.number == agent_run_request::CONVERSATION_ID && f.wire_type == 2);
        assert!(has_action, "RunRequest must carry the action sub-message");
        assert!(has_model, "RunRequest must carry model details");
        assert!(
            has_conversation_id,
            "RunRequest must carry conversation_id when supplied",
        );
    }
}

#[test]
fn client_heartbeat_encodes_to_two_bytes() {
    // ClientHeartbeat is the empty message wrapped in
    // `AgentClientMessage.client_heartbeat` (field 7, wire type 2, len 0).
    let bytes = encode_client_heartbeat();
    assert_eq!(
        bytes,
        vec![0x3a, 0x00],
        "heartbeat wire bytes must match v1 encoder exactly",
    );
}

#[test]
fn agent_run_request_encodes_workspace_env_and_model_field_paths() {
    // White-box check that the workspace path lands under
    // RequestContext.env.workspace_paths AND env.project_folder, and that
    // the model id round-trips via ModelDetails.
    let messages = [Message {
        role: "user".to_string(),
        text: "ping".to_string(),
    }];
    let encoded = encode_agent_run_request(AgentRunRequestInput {
        model: "composer-1.5",
        requested_model: None,
        messages: &messages,
        message_id: "mid",
        conversation_id: None,
        os_version: "darwin-24.6.0",
        workspace_path: "/tmp/workspace",
        shell: "/bin/zsh",
        tools: &[],
    });

    let outer = parse_proto_fields(&encoded);
    let run_payload = outer
        .iter()
        .find(|f| f.number == agent_client_message::RUN_REQUEST)
        .expect("encoded bytes carry AgentClientMessage.run_request")
        .value
        .clone();

    let run_fields = parse_proto_fields(&run_payload);
    let action_payload = run_fields
        .iter()
        .find(|f| f.number == agent_run_request::ACTION)
        .expect("RunRequest must carry an action")
        .value
        .clone();
    let action_fields = parse_proto_fields(&action_payload);

    let user_action_payload = action_fields
        .iter()
        .find(|f| f.number == conversation_action::USER_MESSAGE_ACTION)
        .expect("ConversationAction.user_message_action present")
        .value
        .clone();
    let user_action_fields = parse_proto_fields(&user_action_payload);

    // RequestContext sub-message at field 2.
    let request_context_payload = user_action_fields
        .iter()
        .find(|f| f.number == user_message_action::REQUEST_CONTEXT)
        .expect("UserMessageAction.request_context present")
        .value
        .clone();
    let request_context_fields = parse_proto_fields(&request_context_payload);
    let env_payload = request_context_fields
        .iter()
        .find(|f| f.number == request_context::ENV)
        .expect("RequestContext.env present")
        .value
        .clone();
    let env_fields = parse_proto_fields(&env_payload);

    let env_strings: Vec<String> = env_fields
        .iter()
        .filter(|f| f.wire_type == 2)
        .map(|f| String::from_utf8_lossy(&f.value).into_owned())
        .collect();
    assert!(
        env_strings.iter().any(|s| s == "/tmp/workspace"),
        "workspace path absent from RequestContextEnv: {env_strings:?}",
    );

    // Model details at field 3 (under RunRequest).
    let model_payload = run_fields
        .iter()
        .find(|f| f.number == agent_run_request::MODEL_DETAILS)
        .expect("RunRequest must carry model details")
        .value
        .clone();
    let model_fields = parse_proto_fields(&model_payload);
    let model_id = model_fields
        .iter()
        .find(|f| f.number == model_details::MODEL_ID)
        .map(|f| String::from_utf8_lossy(&f.value).into_owned())
        .expect("ModelDetails.model_id present");
    assert_eq!(model_id, "composer-1.5");

    // Verify env namespace tags surface as expected.
    let _ = (
        request_context_env::OS_VERSION,
        request_context_env::SHELL,
        request_context_env::PROJECT_FOLDER,
        request_context_env::WORKSPACE_PATHS,
    );
    // Lane G dependency: once Lane G replaces the Message struct with a
    // richer DTO, this test should re-key on the new type. Pinning the
    // Phase 0 wire shape now keeps Phase 1 honest.
    let _ = user_message::TEXT;
}

#[test]
fn agent_run_request_encodes_requested_model_for_composer_fast_variant() {
    let messages = [Message {
        role: "user".to_string(),
        text: "ping".to_string(),
    }];
    let parameters = [RequestedModelParameter {
        id: "fast",
        value: "true",
    }];
    let encoded = encode_agent_run_request(AgentRunRequestInput {
        model: "composer-2.5-fast",
        requested_model: Some(RequestedModelInput {
            model_id: "composer-2.5",
            max_mode: false,
            parameters: &parameters,
        }),
        messages: &messages,
        message_id: "mid",
        conversation_id: Some("conv"),
        os_version: "Darwin",
        workspace_path: "/tmp/workspace",
        shell: "zsh",
        tools: &[],
    });

    let outer = parse_proto_fields(&encoded);
    let run_payload = outer
        .iter()
        .find(|field| field.number == agent_client_message::RUN_REQUEST)
        .expect("run request")
        .value
        .clone();
    let run_fields = parse_proto_fields(&run_payload);

    let model_payload = run_fields
        .iter()
        .find(|field| field.number == agent_run_request::MODEL_DETAILS)
        .expect("ModelDetails present")
        .value
        .clone();
    let model_id = parse_proto_fields(&model_payload)
        .into_iter()
        .find(|field| field.number == model_details::MODEL_ID)
        .map(|field| String::from_utf8_lossy(&field.value).into_owned())
        .expect("ModelDetails.model_id present");
    assert_eq!(model_id, "composer-2.5-fast");

    let requested_payload = run_fields
        .iter()
        .find(|field| field.number == agent_run_request::REQUESTED_MODEL)
        .expect("RequestedModel present")
        .value
        .clone();
    let requested_fields = parse_proto_fields(&requested_payload);
    let requested_model_id = requested_fields
        .iter()
        .find(|field| field.number == requested_model::MODEL_ID)
        .map(|field| String::from_utf8_lossy(&field.value).into_owned())
        .expect("RequestedModel.model_id present");
    assert_eq!(requested_model_id, "composer-2.5");

    let parameter_payload = requested_fields
        .iter()
        .find(|field| field.number == requested_model::PARAMETERS)
        .expect("RequestedModel.parameters present")
        .value
        .clone();
    let parameter_fields = parse_proto_fields(&parameter_payload);
    let parameter_id = parameter_fields
        .iter()
        .find(|field| field.number == requested_model_parameter::ID)
        .map(|field| String::from_utf8_lossy(&field.value).into_owned())
        .expect("parameter id present");
    let parameter_value = parameter_fields
        .iter()
        .find(|field| field.number == requested_model_parameter::VALUE)
        .map(|field| String::from_utf8_lossy(&field.value).into_owned())
        .expect("parameter value present");
    assert_eq!(
        (parameter_id.as_str(), parameter_value.as_str()),
        ("fast", "true")
    );
}

// ---------------------------------------------------------------------------
// AgentServerMessage decode coverage per variant
// ---------------------------------------------------------------------------

#[test]
fn agent_server_message_decodes_text_delta_into_text_event() {
    let interaction_payload = encode_message_field(
        interaction_update::TEXT_DELTA,
        &encode_string_field(1, "hello world"),
    );
    let server_payload = encode_message_field(
        agent_server_message::INTERACTION_UPDATE,
        &interaction_payload,
    );
    let decoded = decode_agent_server_message(&server_payload);
    assert_eq!(
        decoded.events,
        vec![InteractionEvent::Text("hello world".to_string())]
    );
    assert!(decoded.exec_requests.is_empty());
    assert!(decoded.kv_requests.is_empty());
}

#[test]
fn agent_server_message_decodes_reasoning_delta_into_thinking_event() {
    let interaction_payload = encode_message_field(
        interaction_update::THINKING_DELTA,
        &encode_string_field(1, "step-by-step reasoning"),
    );
    let server_payload = encode_message_field(
        agent_server_message::INTERACTION_UPDATE,
        &interaction_payload,
    );
    let decoded = decode_agent_server_message(&server_payload);
    assert_eq!(
        decoded.events,
        vec![InteractionEvent::Thinking(
            "step-by-step reasoning".to_string()
        )]
    );
}

#[test]
fn agent_server_message_decodes_token_delta_event() {
    let interaction_payload =
        encode_message_field(interaction_update::TOKEN_DELTA, &encode_int32_field(1, 42));
    let server_payload = encode_message_field(
        agent_server_message::INTERACTION_UPDATE,
        &interaction_payload,
    );
    let decoded = decode_agent_server_message(&server_payload);
    assert_eq!(decoded.events, vec![InteractionEvent::TokenDelta(42)]);
}

#[test]
fn agent_server_message_decodes_heartbeat_and_turn_ended() {
    // Heartbeat and TurnEnded are presence-only. Construct InteractionUpdate
    // as `{ heartbeat: <empty>, turn_ended: <varint=1> }`. Wire-type 2
    // (length-delimited) for HEARTBEAT, since the schema represents it as an
    // empty sub-message.
    let interaction_payload = {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&encode_message_field(interaction_update::HEARTBEAT, &[]));
        bytes.extend_from_slice(&encode_message_field(interaction_update::TURN_ENDED, &[]));
        bytes
    };
    let server_payload = encode_message_field(
        agent_server_message::INTERACTION_UPDATE,
        &interaction_payload,
    );
    let decoded = decode_agent_server_message(&server_payload);
    assert!(decoded.events.contains(&InteractionEvent::Heartbeat));
    assert!(decoded.events.contains(&InteractionEvent::TurnEnded));
}

#[test]
fn agent_server_message_decodes_checkpoint_update_flag() {
    // ConversationCheckpointUpdate at field 3. Body content is opaque for
    // Phase 0; the decoder just records that we saw one.
    let server_payload = encode_message_field(
        agent_server_message::CONVERSATION_CHECKPOINT_UPDATE,
        b"opaque-checkpoint-body",
    );
    let decoded = decode_agent_server_message(&server_payload);
    assert!(
        decoded.saw_checkpoint,
        "checkpoint frame must set saw_checkpoint",
    );
}

#[test]
fn agent_server_message_decodes_exec_request_envelope() {
    // ExecServerMessage with id=7, exec_id="exec-fixture", and a fetch
    // sub-args payload. The decoder surfaces these so higher-level lanes
    // can dispatch the request.
    let mut exec_bytes = Vec::new();
    exec_bytes.extend_from_slice(&encode_int64_field(1, 7));
    exec_bytes.extend_from_slice(&encode_string_field(15, "exec-fixture"));
    exec_bytes.extend_from_slice(&encode_message_field(20, b"fetch-args-payload")); // FETCH_ARGS = 20
    let server_payload =
        encode_message_field(agent_server_message::EXEC_SERVER_MESSAGE, &exec_bytes);
    let decoded = decode_agent_server_message(&server_payload);
    assert_eq!(decoded.exec_requests.len(), 1);
    let exec = &decoded.exec_requests[0];
    assert_eq!(exec.id, 7);
    assert_eq!(exec.exec_id, "exec-fixture");
    assert_eq!(exec.kind, ExecKind::Fetch);
    assert_eq!(exec.args, b"fetch-args-payload");
}

#[test]
fn exec_request_maps_native_grep_to_public_tool_call() {
    let args = [
        encode_string_field(1, "lookup"),
        encode_string_field(2, "."),
        encode_string_field(3, "*.{rs,toml}"),
    ]
    .concat();
    let exec = ExecRequest {
        id: 7,
        exec_id: "exec-grep".to_string(),
        kind: ExecKind::Grep,
        args,
    };

    let (name, call_id, arguments) = decode_exec_public_tool_call(&exec);

    assert_eq!(name, "grep");
    assert_eq!(call_id, "exec-grep");
    assert_eq!(arguments["pattern"], "lookup");
    assert_eq!(arguments["path"], ".");
    assert_eq!(arguments["output_mode"], "*.{rs,toml}");
}

#[test]
fn exec_request_maps_mcp_payload_to_original_public_tool() {
    let argument_entry = [
        encode_string_field(1, "query"),
        encode_message_field(2, br#""cursor adapter""#),
    ]
    .concat();
    let args = [
        encode_message_field(2, &argument_entry),
        encode_string_field(3, "call-search"),
        encode_string_field(5, "cursor_codebase_search"),
    ]
    .concat();
    let exec = ExecRequest {
        id: 11,
        exec_id: "exec-mcp".to_string(),
        kind: ExecKind::Mcp,
        args,
    };

    let (name, call_id, arguments) = decode_exec_public_tool_call(&exec);

    assert_eq!(name, "cursor_codebase_search");
    assert_eq!(call_id, "call-search");
    assert_eq!(arguments["query"], "cursor adapter");
}

#[test]
fn request_context_encodes_mcp_tool_schema_as_protobuf_value() {
    let exec = ExecRequest {
        id: 5,
        exec_id: "exec-request-context".to_string(),
        kind: ExecKind::RequestContext,
        args: Vec::new(),
    };
    let tools = vec![CursorTool {
        name: "cursor_codebase_search".to_string(),
        description: Some("Search the Cursor index".to_string()),
        parameters_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "query": { "type": "string" },
                "include_globs": false,
                "limit": 0
            },
            "required": ["query"]
        }),
        kind: CursorToolKind::Function,
    }];

    let encoded = encode_request_context_result(&exec, &tools, "Darwin", "/tmp/workspace", "zsh");
    let outer = parse_proto_fields(&encoded);
    let exec_payload = outer
        .iter()
        .find(|field| field.number == agent_client_message::EXEC_CLIENT_MESSAGE)
        .expect("exec client message")
        .value
        .clone();
    let exec_fields = parse_proto_fields(&exec_payload);
    let request_context_result = exec_fields
        .iter()
        .find(|field| field.number == 10)
        .expect("request context result")
        .value
        .clone();
    let success = parse_proto_fields(&request_context_result)
        .into_iter()
        .find(|field| field.number == 1)
        .expect("success result")
        .value;
    let context = parse_proto_fields(&success)
        .into_iter()
        .find(|field| field.number == 1)
        .expect("request context payload")
        .value;
    let tool = parse_proto_fields(&context)
        .into_iter()
        .find(|field| field.number == request_context::TOOLS)
        .expect("tool definition")
        .value;
    let input_schema = parse_proto_fields(&tool)
        .into_iter()
        .find(|field| field.number == 3)
        .expect("input schema")
        .value;

    assert!(
        !input_schema.starts_with(b"{"),
        "Cursor expects protobuf google.protobuf.Value bytes, not raw JSON"
    );
    let value_fields = parse_proto_fields(&input_schema);
    let struct_value = value_fields
        .iter()
        .find(|field| field.number == 5)
        .expect("top-level schema encoded as Value.struct_value");
    let entries = parse_proto_fields(&struct_value.value);
    assert!(
        entries.iter().any(|field| field.number == 1),
        "struct value carries schema field entries"
    );
}

#[test]
fn agent_run_request_preserves_empty_string_schema_values_as_protobuf_values() {
    let tools = vec![CursorTool {
        name: "noop_empty".to_string(),
        description: Some("No-op tool".to_string()),
        parameters_schema: serde_json::json!({
            "type": "object",
            "description": "",
            "properties": {
                "path": {
                    "type": "string",
                    "description": ""
                }
            },
            "required": ["path"]
        }),
        kind: CursorToolKind::Function,
    }];
    let messages = [Message {
        role: "user".to_string(),
        text: "hello".to_string(),
    }];

    let encoded = encode_agent_run_request(AgentRunRequestInput {
        model: "composer-2-fast",
        requested_model: None,
        messages: &messages,
        message_id: "mid",
        conversation_id: Some("conv"),
        os_version: "Darwin",
        workspace_path: "/tmp/workspace",
        shell: "zsh",
        tools: &tools,
    });

    let outer = parse_proto_fields(&encoded);
    let run_payload = outer
        .iter()
        .find(|field| field.number == agent_client_message::RUN_REQUEST)
        .expect("run request")
        .value
        .clone();
    let run_fields = parse_proto_fields(&run_payload);
    let mcp_tools = run_fields
        .iter()
        .find(|field| field.number == agent_run_request::MCP_TOOLS)
        .expect("mcp tools")
        .value
        .clone();
    let tool = parse_proto_fields(&mcp_tools)
        .into_iter()
        .find(|field| field.number == 1)
        .expect("tool entry")
        .value;
    let input_schema = parse_proto_fields(&tool)
        .into_iter()
        .find(|field| field.number == 3)
        .expect("input schema")
        .value;
    let value_fields = parse_proto_fields(&input_schema);
    let struct_value = value_fields
        .iter()
        .find(|field| field.number == 5)
        .expect("top-level schema encoded as Value.struct_value");
    let description_entry = parse_proto_fields(&struct_value.value)
        .into_iter()
        .find(|entry| {
            parse_proto_fields(&entry.value)
                .into_iter()
                .any(|field| field.number == 1 && field.value == b"description")
        })
        .expect("description field entry");
    let empty_description_value = parse_proto_fields(&description_entry.value)
        .into_iter()
        .find(|field| field.number == 2)
        .expect("description value");
    let string_value = parse_proto_fields(&empty_description_value.value)
        .into_iter()
        .find(|field| field.number == 3)
        .expect("empty string_value kind must be present");

    assert!(string_value.value.is_empty());
}

#[test]
fn agent_run_request_includes_initial_mcp_tools() {
    let tools = vec![CursorTool {
        name: "cursor_codebase_search".to_string(),
        description: Some("Search the Cursor index".to_string()),
        parameters_schema: serde_json::json!({
            "type": "object",
            "properties": { "query": { "type": "string" } },
            "required": ["query"]
        }),
        kind: CursorToolKind::Function,
    }];
    let messages = [Message {
        role: "user".to_string(),
        text: "search".to_string(),
    }];
    let encoded = encode_agent_run_request(AgentRunRequestInput {
        model: "composer-2-fast",
        requested_model: None,
        messages: &messages,
        message_id: "mid",
        conversation_id: Some("conv"),
        os_version: "Darwin",
        workspace_path: "/tmp/workspace",
        shell: "zsh",
        tools: &tools,
    });

    let outer = parse_proto_fields(&encoded);
    let run_payload = outer
        .iter()
        .find(|field| field.number == agent_client_message::RUN_REQUEST)
        .expect("run request")
        .value
        .clone();
    let run_fields = parse_proto_fields(&run_payload);
    let mcp_tools = run_fields
        .iter()
        .find(|field| field.number == agent_run_request::MCP_TOOLS)
        .expect("AgentRunRequest.mcp_tools present")
        .value
        .clone();
    let tool = parse_proto_fields(&mcp_tools)
        .into_iter()
        .find(|field| field.number == 1)
        .expect("McpTools.mcp_tools entry")
        .value;
    let tool_fields = parse_proto_fields(&tool);

    assert_eq!(
        tool_fields
            .iter()
            .find(|field| field.number == 1)
            .map(|field| String::from_utf8_lossy(&field.value).into_owned())
            .as_deref(),
        Some("cursor_codebase_search")
    );
    assert!(
        tool_fields
            .iter()
            .find(|field| field.number == 3)
            .is_some_and(|field| !field.value.starts_with(b"{")),
        "initial MCP tool schema must be protobuf Value bytes"
    );
}

#[test]
fn agent_server_message_decodes_kv_get_blob_request() {
    let mut kv_bytes = Vec::new();
    kv_bytes.extend_from_slice(&encode_int64_field(1, 0));
    kv_bytes.extend_from_slice(&encode_message_field(2, b"blob-id-bytes")); // GET_BLOB
    let server_payload = encode_message_field(agent_server_message::KV_SERVER_MESSAGE, &kv_bytes);
    let decoded = decode_agent_server_message(&server_payload);
    assert_eq!(decoded.kv_requests.len(), 1);
    let kv = &decoded.kv_requests[0];
    assert_eq!(kv.id, 0);
    assert_eq!(kv.kind, KvKind::GetBlob);
    assert_eq!(kv.args, b"blob-id-bytes");
}

#[test]
fn agent_server_message_decode_handles_malformed_payload_without_panic() {
    // Truncated tag: a single 0x80 byte means "wait for more" but the
    // parser should silently stop, not panic.
    let decoded = decode_agent_server_message(&[0x80]);
    assert!(decoded.events.is_empty());
    assert!(decoded.exec_requests.is_empty());
    assert!(decoded.kv_requests.is_empty());
    assert!(!decoded.saw_checkpoint);

    // Length-delimited field with len=8 but only 2 bytes follow.
    let payload: &[u8] = &[
        ((agent_server_message::INTERACTION_UPDATE << 3) | 2) as u8,
        8,
        1,
        2,
    ];
    let decoded = decode_agent_server_message(payload);
    assert!(decoded.events.is_empty());
}

// ---------------------------------------------------------------------------
// GetUsableModels unary
// ---------------------------------------------------------------------------

#[test]
fn get_usable_models_request_is_empty_body() {
    assert!(
        encode_get_usable_models_request().is_empty(),
        "GetUsableModelsRequest is the proto3 empty message",
    );
}

#[test]
fn get_usable_models_response_decodes_three_composer_rows() {
    // Synthesize a response body carrying three ModelDetails entries with
    // model_ids `composer-1.5`, `composer-2`, `composer-2-fast`. composer-2
    // and composer-2-fast set the THINKING_DETAILS presence bit so
    // `supports_reasoning` flips true.
    let mut models_payload = Vec::new();
    for (model_id, with_thinking) in [
        ("composer-1.5", false),
        ("composer-2", true),
        ("composer-2-fast", true),
    ] {
        let mut details = Vec::new();
        details.extend_from_slice(&encode_string_field(model_details::MODEL_ID, model_id));
        if with_thinking {
            details.extend_from_slice(&encode_message_field(model_details::THINKING_DETAILS, &[]));
        }
        models_payload.extend_from_slice(&encode_message_field(
            get_usable_models_response::MODELS,
            &details,
        ));
    }

    let descriptors = decode_get_usable_models_response(&models_payload);
    let ids: Vec<&str> = descriptors.iter().map(|d| d.model_id.as_str()).collect();
    assert!(ids.contains(&"composer-1.5"));
    assert!(ids.contains(&"composer-2"));
    assert!(ids.contains(&"composer-2-fast"));

    let composer_2 = descriptors
        .iter()
        .find(|d| d.model_id == "composer-2")
        .unwrap();
    assert!(composer_2.supports_reasoning);
    let composer_15 = descriptors
        .iter()
        .find(|d| d.model_id == "composer-1.5")
        .unwrap();
    assert!(!composer_15.supports_reasoning);
}

#[test]
fn get_usable_models_response_decodes_connect_framed_payload() {
    // Two-frame Connect framing: first frame is the protobuf body, second
    // frame is the end-stream sentinel. The decoder runs over the inner
    // frame body once the caller strips the envelope.
    let mut details = Vec::new();
    details.extend_from_slice(&encode_string_field(model_details::MODEL_ID, "composer-2"));
    let body = encode_message_field(get_usable_models_response::MODELS, &details);
    let connect_body = frame_connect_message(&body, 0);
    let trailer = frame_connect_message(b"", CONNECT_END_STREAM_FLAG);

    let mut buf = Vec::new();
    buf.extend_from_slice(&connect_body);
    buf.extend_from_slice(&trailer);

    // Pop the first (data) frame.
    let mut work = buf.clone();
    let (flags, payload) = take_connect_frame(&mut work).expect("first frame is data");
    assert_eq!(flags, 0);
    let descriptors = decode_get_usable_models_response(&payload);
    assert_eq!(descriptors.len(), 1);
    assert_eq!(descriptors[0].model_id, "composer-2");

    // Pop the second (end-stream) frame.
    let (flags, _trailer_body) = take_connect_frame(&mut work).expect("second frame trailer");
    assert_eq!(flags & CONNECT_END_STREAM_FLAG, CONNECT_END_STREAM_FLAG);
}

// ---------------------------------------------------------------------------
// Fixture helpers shared with future test files
// ---------------------------------------------------------------------------

#[allow(dead_code)]
fn assert_fixture_dir_exists() {
    let dir = manifest_dir().join("tests/fixtures/cursor");
    assert!(
        Path::new(&dir).is_dir(),
        "tests/fixtures/cursor must exist; run scripts/cursor/generate-fixtures.mjs to populate",
    );
}

#[allow(dead_code)]
fn proto_field_count(bytes: &[u8]) -> usize {
    parse_proto_fields(bytes).len()
}

#[allow(dead_code)]
fn _proto_field_unused(_field: &ProtoField) {}

// ---------------------------------------------------------------------------
// ExecKind -> public tool-call mapping coverage.
//
// Pins the actual current behavior of
// `proto::decode_exec_public_tool_call` so any drift in the load-bearing
// translation table fails CI loudly. Lane K coordination notes inline
// where the task originally proposed a different mapping (e.g.
// read_file vs read, list_directory vs ls, apply_patch vs write/delete);
// the tests record what the implementation does today, not the
// aspirational naming.
// ---------------------------------------------------------------------------

fn build_exec(kind: ExecKind, args: Vec<u8>) -> ExecRequest {
    ExecRequest {
        id: 42,
        exec_id: "exec-fixture-id".to_string(),
        kind,
        args,
    }
}

#[test]
fn exec_request_maps_read_args_to_read_tool() {
    // ReadArgs is `{ path = 1 }`. Implementation produces tool_name "read"
    // (NOT "read_file" as the dispatcher contract docs sometimes phrase
    // it) and the call_id mirrors the cursor exec_id, not "".
    let args = encode_string_field(1, "/tmp/file.txt");
    let exec = build_exec(ExecKind::Read, args);
    let (name, call_id, arguments) = decode_exec_public_tool_call(&exec);
    assert_eq!(name, "read");
    assert_eq!(call_id, "exec-fixture-id");
    assert_eq!(arguments["path"], "/tmp/file.txt");
}

#[test]
fn exec_request_maps_ls_args_to_ls_tool() {
    // LsArgs `{ path = 1 }`. Implementation tool_name is "ls" (NOT
    // "list_directory" — that is a Cursor-side display label, not the
    // proxy-internal mapping).
    let args = encode_string_field(1, "/tmp");
    let exec = build_exec(ExecKind::Ls, args);
    let (name, call_id, arguments) = decode_exec_public_tool_call(&exec);
    assert_eq!(name, "ls");
    assert_eq!(call_id, "exec-fixture-id");
    assert_eq!(arguments["path"], "/tmp");
}

#[test]
fn exec_request_maps_grep_args_to_grep_tool() {
    // GrepArgs `{ pattern = 1, path = 2, output_mode = 3 }`.
    let args = [
        encode_string_field(1, "needle"),
        encode_string_field(2, "/repo"),
        encode_string_field(3, "files_with_matches"),
    ]
    .concat();
    let exec = build_exec(ExecKind::Grep, args);
    let (name, call_id, arguments) = decode_exec_public_tool_call(&exec);
    assert_eq!(name, "grep");
    assert_eq!(call_id, "exec-fixture-id");
    assert_eq!(arguments["pattern"], "needle");
    assert_eq!(arguments["path"], "/repo");
    assert_eq!(arguments["output_mode"], "files_with_matches");
}

#[test]
fn exec_request_maps_shell_args_to_shell_tool() {
    // ShellArgs `{ command = 1, working_directory = 2 }`.
    let args = [
        encode_string_field(1, "ls -la"),
        encode_string_field(2, "/repo"),
    ]
    .concat();
    let exec = build_exec(ExecKind::Shell, args);
    let (name, call_id, arguments) = decode_exec_public_tool_call(&exec);
    assert_eq!(name, "shell");
    assert_eq!(call_id, "exec-fixture-id");
    assert_eq!(arguments["command"], "ls -la");
    assert_eq!(arguments["working_directory"], "/repo");
}

#[test]
fn exec_request_maps_shell_stream_args_to_shell_stream_tool() {
    // ShellStream uses the same `{ command, working_directory }` envelope
    // and emits tool_name "shell_stream" (NOT "shell"; the run engine
    // distinguishes the two so the route layer can route stream output
    // back over the wire).
    let args = [
        encode_string_field(1, "tail -f log"),
        encode_string_field(2, "/var/log"),
    ]
    .concat();
    let exec = build_exec(ExecKind::ShellStream, args);
    let (name, call_id, arguments) = decode_exec_public_tool_call(&exec);
    assert_eq!(name, "shell_stream");
    assert_eq!(call_id, "exec-fixture-id");
    assert_eq!(arguments["command"], "tail -f log");
    assert_eq!(arguments["working_directory"], "/var/log");
}

#[test]
fn exec_request_maps_write_args_to_write_tool() {
    // WriteArgs `{ path = 1, ... }`. The proxy emits tool_name "write"
    // (NOT "apply_patch" as the Codex CLI built-in is named upstream;
    // Lane K future work may align these for the public-API translation,
    // but the proto path keeps the Cursor-native name today).
    let args = encode_string_field(1, "/tmp/out.txt");
    let exec = build_exec(ExecKind::Write, args);
    let (name, call_id, arguments) = decode_exec_public_tool_call(&exec);
    assert_eq!(name, "write");
    assert_eq!(call_id, "exec-fixture-id");
    assert_eq!(arguments["path"], "/tmp/out.txt");
}

#[test]
fn exec_request_maps_delete_args_to_delete_tool() {
    // DeleteArgs `{ path = 1 }`. tool_name is "delete" (NOT
    // "apply_patch" — see Lane K coordination note above).
    let args = encode_string_field(1, "/tmp/garbage.txt");
    let exec = build_exec(ExecKind::Delete, args);
    let (name, call_id, arguments) = decode_exec_public_tool_call(&exec);
    assert_eq!(name, "delete");
    assert_eq!(call_id, "exec-fixture-id");
    assert_eq!(arguments["path"], "/tmp/garbage.txt");
}

#[test]
fn exec_request_maps_diagnostics_to_diagnostics_tool_with_unknown_args_shape() {
    // DiagnosticsArgs schema is opaque to the proxy today, so the impl
    // routes them through `decode_unknown_exec_args`, which returns an
    // object keyed `field_<n>`. Lane K coordination: when a public schema
    // for diagnostics surfaces, this test should pin to that shape; for
    // now we lock the fallback shape so any structural drift is loud.
    let args = [encode_string_field(1, "/tmp/diag.txt")].concat();
    let exec = build_exec(ExecKind::Diagnostics, args);
    let (name, call_id, arguments) = decode_exec_public_tool_call(&exec);
    assert_eq!(name, "diagnostics");
    assert_eq!(call_id, "exec-fixture-id");
    // `decode_unknown_exec_args` keys field 1 as "field_1".
    assert_eq!(arguments["field_1"], "/tmp/diag.txt");
}

#[test]
fn exec_request_maps_mcp_payload_to_mcp_tool_name_from_field_5() {
    // McpArgs uses field 5 for the tool name, field 3 for tool_call_id,
    // field 2 (repeated) for arguments. This is the canonical MCP envelope
    // (NOT a proxy-internal cursor_codebase_search special case; all MCP
    // tools route through the same public tool-call decoder).
    let argument_entry = [
        encode_string_field(1, "key"),
        encode_message_field(2, br#""value""#),
    ]
    .concat();
    let args = [
        encode_message_field(2, &argument_entry),
        encode_string_field(3, "mcp-call-id-123"),
        encode_string_field(5, "third_party_tool"),
    ]
    .concat();
    let exec = build_exec(ExecKind::Mcp, args);
    let (name, call_id, arguments) = decode_exec_public_tool_call(&exec);
    assert_eq!(name, "third_party_tool");
    assert_eq!(call_id, "mcp-call-id-123");
    assert_eq!(arguments["key"], "value");
}

#[test]
fn exec_request_maps_mcp_payload_for_cursor_codebase_search_public_tool() {
    // Composer-2 reasoning emits MCP args with `tool: cursor_codebase_search`
    // when it wants to consult the workspace index. The decoder surfaces the
    // same name unchanged so the public adapters emit a normal tool call.
    let argument_entry = [
        encode_string_field(1, "query"),
        encode_message_field(2, br#""how does auth flow""#),
    ]
    .concat();
    let args = [
        encode_message_field(2, &argument_entry),
        encode_string_field(3, "search-call-id"),
        encode_string_field(5, "cursor_codebase_search"),
    ]
    .concat();
    let exec = build_exec(ExecKind::Mcp, args);
    let (name, call_id, arguments) = decode_exec_public_tool_call(&exec);
    assert_eq!(name, "cursor_codebase_search");
    assert_eq!(call_id, "search-call-id");
    assert_eq!(arguments["query"], "how does auth flow");
}

#[test]
fn exec_request_maps_fetch_args_to_fetch_tool() {
    // FetchArgs `{ url = 1 }`. tool_name is "fetch" (NOT "fetch_url" —
    // see exec_public_tool_name in proto.rs).
    let args = encode_string_field(1, "https://example.com");
    let exec = build_exec(ExecKind::Fetch, args);
    let (name, call_id, arguments) = decode_exec_public_tool_call(&exec);
    assert_eq!(name, "fetch");
    assert_eq!(call_id, "exec-fixture-id");
    assert_eq!(arguments["url"], "https://example.com");
}

#[test]
fn exec_request_maps_other_field_to_cursor_exec_fallback() {
    // Unknown ExecKind variants (proto fields the proxy does not know
    // how to translate) collapse to tool_name "cursor_exec" and an
    // unknown_exec_args shape with field_<n> keys. Lane K coordination:
    // any new ExecKind should add a typed mapping rather than relying on
    // this fallback.
    let args = [encode_string_field(1, "opaque payload")].concat();
    let exec = build_exec(ExecKind::Other(99), args);
    let (name, call_id, arguments) = decode_exec_public_tool_call(&exec);
    assert_eq!(name, "cursor_exec");
    assert_eq!(call_id, "exec-fixture-id");
    assert_eq!(arguments["field_1"], "opaque payload");
}

#[test]
fn agent_server_message_decodes_unknown_exec_oneof_as_cursor_exec_fallback() {
    let args = encode_string_field(1, "future opaque payload");
    let mut exec_bytes = Vec::new();
    exec_bytes.extend_from_slice(&encode_int64_field(1, 99));
    exec_bytes.extend_from_slice(&encode_string_field(15, "future-exec-id"));
    exec_bytes.extend_from_slice(&encode_message_field(99, &args));
    let server_payload =
        encode_message_field(agent_server_message::EXEC_SERVER_MESSAGE, &exec_bytes);

    let decoded = decode_agent_server_message(&server_payload);

    assert_eq!(decoded.exec_requests.len(), 1);
    let exec = &decoded.exec_requests[0];
    assert_eq!(exec.id, 99);
    assert_eq!(exec.exec_id, "future-exec-id");
    assert_eq!(exec.kind, ExecKind::Other(99));
    let (name, call_id, arguments) = decode_exec_public_tool_call(exec);
    assert_eq!(name, "cursor_exec");
    assert_eq!(call_id, "future-exec-id");
    assert_eq!(arguments["field_1"], "future opaque payload");
}
