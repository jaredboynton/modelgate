use serde_json::json;
use unified_model_proxy_v2::adapter::{cursor_chat, cursor_messages, cursor_responses};
use unified_model_proxy_v2::cursor_agent::CursorMessage;

#[test]
fn responses_adapter_extracts_function_call_output_into_tool_results() {
    let body = json!({
    "model": "composer-2-fast",
    "input": [
    { "type": "message", "role": "user", "content": "hi" },
    {
    "type": "function_call_output",
    "call_id": "call_xyz",
    "output": "{\"result\":42}"
    }
    ],
    "previous_response_id": "resp_prior"
    });
    let request = cursor_responses::build_request(&body).expect("build_request succeeds");
    assert_eq!(request.tool_results.len(), 1);
    let result = &request.tool_results[0];
    assert_eq!(result.call_id, "call_xyz");
    assert!(
        result.output.is_string() || result.output.is_object(),
        "output is either the verbatim JSON string or a parsed object",
    );
    assert!(result.error.is_none());
}

#[test]
fn responses_adapter_accepts_parallel_tool_calls_compat_field() {
    for value in [json!(true), json!(false), json!(null)] {
        let body = json!({
        "model": "composer-2-fast",
        "input": "hi",
        "parallel_tool_calls": value
        });

        cursor_responses::build_request(&body).expect("parallel_tool_calls compat field accepted");
    }

    let body = json!({
    "model": "composer-2-fast",
    "input": "hi",
    "parallel_tool_calls": "false"
    });
    let err = cursor_responses::build_request(&body).expect_err("string field rejected");
    let msg = format!("{err:?}");
    assert!(
        msg.contains("parallel_tool_calls"),
        "error mentions parallel_tool_calls: {msg}",
    );
}

#[test]
fn responses_adapter_maps_canonical_function_call_output_blocks_to_cursor_tool_result() {
    let body = json!({
    "model": "composer-2-fast",
    "input": [
    { "type": "message", "role": "user", "content": "hi" },
    {
    "type": "function_call_output",
    "call_id": "call_openai",
    "output": [
    { "type": "output_text", "text": "first " },
    { "type": "text", "text": "second" }
    ]
    }
    ],
    "previous_response_id": "resp_prior"
    });

    let request = cursor_responses::build_request(&body).expect("build_request succeeds");

    assert_eq!(request.tool_results.len(), 1);
    assert_eq!(request.tool_results[0].call_id, "call_openai");
    assert_eq!(request.tool_results[0].output, json!("first second"));
    assert!(request.tool_results[0].error.is_none());
}

#[test]
fn responses_adapter_rejects_orphan_tool_result_without_previous_response_id() {
    let body = json!({
    "model": "composer-2-fast",
    "input": [
    { "type": "function_call_output", "call_id": "call_x", "output": "result" }
    ]
    });
    let err = cursor_responses::build_request(&body).expect_err("must reject orphan tool_result");
    let msg = format!("{err:?}");
    assert!(
        msg.contains("previous_response_id"),
        "error mentions previous_response_id: {msg}",
    );
}

#[test]
fn responses_adapter_accepts_replayed_tool_result_with_prior_function_call() {
    let body = json!({
    "model": "composer-2-fast",
    "store": false,
    "input": [
    { "type": "message", "role": "user", "content": "search the repo" },
    {
    "type": "function_call",
    "call_id": "call_lookup",
    "name": "Grep",
    "arguments": "{\"pattern\":\"needle\"}"
    },
    {
    "type": "function_call_output",
    "call_id": "call_lookup",
    "output": "needle found"
    }
    ]
    });
    let request = cursor_responses::build_request(&body).expect("replayed tool result accepted");
    assert_eq!(request.tool_results.len(), 1);
    assert_eq!(request.tool_results[0].call_id, "call_lookup");
}

#[test]
fn responses_adapter_rejects_replayed_tool_result_with_unmatched_call_id() {
    let body = json!({
    "model": "composer-2-fast",
    "input": [
    {
    "type": "function_call",
    "call_id": "call_lookup",
    "name": "Grep",
    "arguments": "{}"
    },
    {
    "type": "function_call_output",
    "call_id": "call_other",
    "output": "result"
    }
    ]
    });
    let err = cursor_responses::build_request(&body)
        .expect_err("unmatched replayed tool result rejected");
    let msg = format!("{err:?}");
    assert!(
        msg.contains("previous_response_id"),
        "error mentions previous_response_id: {msg}",
    );
}

#[test]
fn responses_adapter_extracts_custom_tool_call_output() {
    let body = json!({
    "model": "composer-2-fast",
    "input": [
    { "type": "message", "role": "user", "content": "hi" },
    {
    "type": "custom_tool_call_output",
    "call_id": "call_custom",
    "output": "freeform text"
    }
    ],
    "previous_response_id": "resp_prior"
    });
    let request = cursor_responses::build_request(&body).expect("build_request succeeds");
    assert_eq!(request.tool_results.len(), 1);
    assert_eq!(request.tool_results[0].call_id, "call_custom");
}

#[test]
fn chat_adapter_extracts_role_tool_messages_into_tool_results() {
    let body = json!({
    "model": "composer-2-fast",
    "messages": [
    { "role": "user", "content": "hi" },
    {
    "role": "assistant",
    "content": null,
    "tool_calls": [{
    "id": "call_a",
    "type": "function",
    "function": { "name": "shell", "arguments": "{}" }
    }]
    },
    { "role": "tool", "tool_call_id": "call_a", "content": "result text" }
    ]
    });
    let request = cursor_chat::build_request(&body).expect("build_request succeeds");
    assert_eq!(request.tool_results.len(), 1, "tool result extracted");
    assert_eq!(request.tool_results[0].call_id, "call_a");
    let user_count = request
        .messages
        .iter()
        .filter(|m| matches!(m, CursorMessage::User { .. }))
        .count();
    assert!(user_count >= 1, "user message preserved");
}

#[test]
fn chat_adapter_drops_echoed_assistant_tool_calls_when_tool_results_present() {
    let body = json!({
    "model": "composer-2-fast",
    "messages": [
    { "role": "user", "content": "hi" },
    {
    "role": "assistant",
    "content": null,
    "tool_calls": [{
    "id": "call_a",
    "type": "function",
    "function": { "name": "shell", "arguments": "{}" }
    }]
    },
    { "role": "tool", "tool_call_id": "call_a", "content": "result" }
    ]
    });
    let request = cursor_chat::build_request(&body).expect("build_request succeeds");
    let assistant_with_calls = request.messages.iter().any(|m| {
        matches!(
        m,
        CursorMessage::Assistant { tool_calls, .. } if !tool_calls.is_empty()
        )
    });
    assert!(
 !assistant_with_calls,
 "assistant tool_calls must be dropped when tool_results present (avoids duplicate tool_use replay)",
 );
}

#[test]
fn chat_adapter_extracts_error_from_json_tool_content() {
    let body = json!({
    "model": "composer-2-fast",
    "messages": [
    { "role": "user", "content": "hi" },
    {
    "role": "assistant",
    "content": null,
    "tool_calls": [{
    "id": "call_a",
    "type": "function",
    "function": { "name": "shell", "arguments": "{}" }
    }]
    },
    {
    "role": "tool",
    "tool_call_id": "call_a",
    "content": "{\"error\":\"command failed\",\"exit_code\":1}"
    }
    ]
    });
    let request = cursor_chat::build_request(&body).expect("build_request succeeds");
    assert_eq!(request.tool_results.len(), 1);
    assert_eq!(
        request.tool_results[0].error.as_deref(),
        Some("command failed"),
        "JSON error key surfaces on CursorToolResult::error",
    );
}

#[test]
fn anthropic_messages_adapter_extracts_tool_result_blocks() {
    let body = json!({
    "model": "composer-2-fast",
    "max_tokens": 256,
    "messages": [
    { "role": "user", "content": [{ "type": "text", "text": "hi" }] },
    {
    "role": "assistant",
    "content": [{
    "type": "tool_use",
    "id": "toolu_b",
    "name": "shell",
    "input": {}
    }]
    },
    {
    "role": "user",
    "content": [{
    "type": "tool_result",
    "tool_use_id": "toolu_b",
    "content": "ok"
    }]
    }
    ]
    });
    let request = cursor_messages::build_request(&body).expect("build_request succeeds");
    assert_eq!(request.tool_results.len(), 1);
    assert_eq!(request.tool_results[0].call_id, "toolu_b");
    assert!(
        request.tool_results[0].error.is_none(),
        "no is_error flag => no error populated",
    );
}

#[test]
fn anthropic_messages_adapter_extracts_is_error_into_tool_result_error() {
    let body = json!({
    "model": "composer-2-fast",
    "max_tokens": 256,
    "messages": [
    { "role": "user", "content": [{ "type": "text", "text": "hi" }] },
    {
    "role": "assistant",
    "content": [{
    "type": "tool_use",
    "id": "toolu_b",
    "name": "shell",
    "input": {}
    }]
    },
    {
    "role": "user",
    "content": [{
    "type": "tool_result",
    "tool_use_id": "toolu_b",
    "content": "command failed: exit 1",
    "is_error": true
    }]
    }
    ]
    });
    let request = cursor_messages::build_request(&body).expect("build_request succeeds");
    assert_eq!(request.tool_results.len(), 1);
    assert!(
        request.tool_results[0].error.is_some(),
        "is_error: true must populate the error field",
    );
    assert_eq!(
        request.tool_results[0].error.as_deref(),
        Some("command failed: exit 1"),
    );
}

#[test]
fn anthropic_messages_adapter_rejects_orphan_tool_result_without_prior_tool_use() {
    let body = json!({
    "model": "composer-2-fast",
    "max_tokens": 256,
    "messages": [
    { "role": "user", "content": [{ "type": "text", "text": "hi" }] },
    {
    "role": "user",
    "content": [{
    "type": "tool_result",
    "tool_use_id": "toolu_orphan",
    "content": "result"
    }]
    }
    ]
    });
    let err =
        cursor_messages::build_request(&body).expect_err("orphan tool_result must be rejected");
    let msg = format!("{err:?}");
    assert!(
        msg.contains("previous_response_id") || msg.contains("tool"),
        "error mentions tool/continuation context: {msg}",
    );
}

#[test]
fn anthropic_messages_adapter_extracts_text_content_block_from_tool_result() {
    let body = json!({
    "model": "composer-2-fast",
    "max_tokens": 256,
    "messages": [
    { "role": "user", "content": [{ "type": "text", "text": "hi" }] },
    {
    "role": "assistant",
    "content": [{
    "type": "tool_use",
    "id": "toolu_b",
    "name": "shell",
    "input": {}
    }]
    },
    {
    "role": "user",
    "content": [{
    "type": "tool_result",
    "tool_use_id": "toolu_b",
    "content": [
    { "type": "text", "text": "first " },
    { "type": "text", "text": "second" }
    ]
    }]
    }
    ]
    });
    let request = cursor_messages::build_request(&body).expect("build_request succeeds");
    assert_eq!(request.tool_results.len(), 1);
    let output_str = request.tool_results[0]
        .output
        .as_str()
        .expect("output preserved as string");
    assert_eq!(output_str, "first second");
}
