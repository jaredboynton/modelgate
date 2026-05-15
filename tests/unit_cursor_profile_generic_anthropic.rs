use unified_model_proxy_v2::upstream::cursor::profiles::{
    generic_anthropic, generic_openai, RenderedToolCall,
};
use unified_model_proxy_v2::upstream::cursor::proto::{
    encode_message_field, encode_string_field, ExecKind, ExecRequest,
};

fn build_exec(kind: ExecKind, args: Vec<u8>) -> ExecRequest {
    ExecRequest {
        id: 42,
        exec_id: "exec-fixture-id".to_string(),
        kind,
        args,
    }
}

#[test]
fn generic_anthropic_read_matches_generic_openai_lowercase_read() {
    let args = encode_string_field(1, "/tmp/file.txt");
    let exec = build_exec(ExecKind::Read, args);
    let anthropic = generic_anthropic::render(&exec);
    let openai = generic_openai::render(&exec);
    assert_eq!(anthropic, openai);
    if let RenderedToolCall::Emit {
        tool_name,
        tool_call_id,
        arguments,
    } = anthropic
    {
        assert_eq!(tool_name, "read");
        assert_eq!(tool_call_id, "exec-fixture-id");
        assert_eq!(arguments["path"], "/tmp/file.txt");
    } else {
        panic!("expected Emit");
    }
}

#[test]
fn generic_anthropic_shell_matches_generic_openai_lowercase_shell() {
    let args = [
        encode_string_field(1, "ls -la"),
        encode_string_field(2, "/repo"),
    ]
    .concat();
    let exec = build_exec(ExecKind::Shell, args);
    let anthropic = generic_anthropic::render(&exec);
    let openai = generic_openai::render(&exec);
    assert_eq!(anthropic, openai);
    if let RenderedToolCall::Emit {
        tool_name,
        tool_call_id,
        arguments,
    } = anthropic
    {
        assert_eq!(tool_name, "shell");
        assert_eq!(tool_call_id, "exec-fixture-id");
        assert_eq!(arguments["command"], "ls -la");
        assert_eq!(arguments["working_directory"], "/repo");
    } else {
        panic!("expected Emit");
    }
}

#[test]
fn generic_anthropic_grep_matches_generic_openai_lowercase_grep() {
    let args = [
        encode_string_field(1, "needle"),
        encode_string_field(2, "/repo"),
        encode_string_field(3, "files_with_matches"),
    ]
    .concat();
    let exec = build_exec(ExecKind::Grep, args);
    let anthropic = generic_anthropic::render(&exec);
    let openai = generic_openai::render(&exec);
    assert_eq!(anthropic, openai);
    if let RenderedToolCall::Emit {
        tool_name,
        tool_call_id,
        arguments,
    } = anthropic
    {
        assert_eq!(tool_name, "grep");
        assert_eq!(tool_call_id, "exec-fixture-id");
        assert_eq!(arguments["pattern"], "needle");
        assert_eq!(arguments["path"], "/repo");
        assert_eq!(arguments["output_mode"], "files_with_matches");
    } else {
        panic!("expected Emit");
    }
}

#[test]
fn generic_anthropic_fetch_matches_generic_openai_lowercase_fetch() {
    let args = encode_string_field(1, "https://example.com");
    let exec = build_exec(ExecKind::Fetch, args);
    let anthropic = generic_anthropic::render(&exec);
    let openai = generic_openai::render(&exec);
    assert_eq!(anthropic, openai);
    if let RenderedToolCall::Emit {
        tool_name,
        tool_call_id,
        arguments,
    } = anthropic
    {
        assert_eq!(tool_name, "fetch");
        assert_eq!(tool_call_id, "exec-fixture-id");
        assert_eq!(arguments["url"], "https://example.com");
    } else {
        panic!("expected Emit");
    }
}

#[test]
fn generic_anthropic_other_field_falls_back_to_cursor_exec_like_generic_openai() {
    let args = [encode_string_field(1, "opaque payload")].concat();
    let exec = build_exec(ExecKind::Other(99), args);
    let anthropic = generic_anthropic::render(&exec);
    let openai = generic_openai::render(&exec);
    assert_eq!(anthropic, openai);
    if let RenderedToolCall::Emit {
        tool_name,
        tool_call_id,
        arguments,
    } = anthropic
    {
        assert_eq!(tool_name, "cursor_exec");
        assert_eq!(tool_call_id, "exec-fixture-id");
        assert_eq!(arguments["field_1"], "opaque payload");
    } else {
        panic!("expected Emit");
    }
    let argument_entry = [
        encode_string_field(1, "key"),
        encode_message_field(2, br#""value""#),
    ]
    .concat();
    let mcp_args = [
        encode_message_field(2, &argument_entry),
        encode_string_field(3, "mcp-call-id-123"),
        encode_string_field(5, "third_party_tool"),
    ]
    .concat();
    let mcp_exec = build_exec(ExecKind::Mcp, mcp_args);
    assert_eq!(
        generic_anthropic::render(&mcp_exec),
        generic_openai::render(&mcp_exec),
    );
}
