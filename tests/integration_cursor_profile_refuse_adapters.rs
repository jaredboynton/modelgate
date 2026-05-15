use serde_json::Value;
use unified_model_proxy_v2::adapter::cursor_events::ResponseContext;
use unified_model_proxy_v2::adapter::{cursor_chat, cursor_messages, cursor_responses};
use unified_model_proxy_v2::cursor_agent::CursorAgentEvent;

#[test]
fn responses_refuse_emits_response_failed_with_error_envelope() {
    let mut ctx = ResponseContext::new("composer-2", "resp_test");
    let _ = cursor_responses::emit_event(
        &CursorAgentEvent::TextDelta {
            delta: String::new(),
            content_index: 0,
        },
        &mut ctx,
    );
    assert!(ctx.started);

    let frames = cursor_responses::emit_event(
        &CursorAgentEvent::ProviderError {
            code: "unsupported_exec_kind".to_string(),
            message: "RecordScreen exec is not exposed as a public tool".to_string(),
            cursor_request_id: Some("req_xyz".to_string()),
        },
        &mut ctx,
    );

    let names: Vec<&str> = frames.iter().map(|f| f.event.as_str()).collect();
    assert!(
        names.contains(&"response.failed"),
        "expected response.failed in {names:?}",
    );

    let failed = frames
        .iter()
        .find(|f| f.event == "response.failed")
        .expect("response.failed frame present");
    let response = failed
        .data
        .get("response")
        .expect("response.failed.data.response present");
    assert_eq!(
        response.get("status").and_then(Value::as_str),
        Some("failed"),
    );
    let error = response.get("error").expect("error envelope present");
    assert_eq!(
        error.get("type").and_then(Value::as_str),
        Some("unsupported_exec_kind"),
    );
    assert_eq!(
        error.get("message").and_then(Value::as_str),
        Some("RecordScreen exec is not exposed as a public tool"),
    );
    assert!(ctx.failed);
}

#[test]
fn chat_refuse_emits_error_chunk_with_empty_choices() {
    let mut ctx = cursor_chat::ChatContext::new("composer-2");
    let chunks = cursor_chat::emit_event(
        &CursorAgentEvent::ProviderError {
            code: "unsupported_exec_kind".to_string(),
            message: "RecordScreen exec is not exposed as a public tool".to_string(),
            cursor_request_id: Some("req_xyz".to_string()),
        },
        &mut ctx,
    );

    let error_chunk = chunks
        .iter()
        .find(|chunk| chunk.get("error").is_some())
        .expect("error chunk present");

    assert_eq!(
        error_chunk.get("object").and_then(Value::as_str),
        Some("chat.completion.chunk"),
    );

    let error = error_chunk
        .get("error")
        .expect("error chunk has error object");
    assert_eq!(
        error.get("type").and_then(Value::as_str),
        Some("unsupported_exec_kind"),
    );
    assert_eq!(
        error.get("message").and_then(Value::as_str),
        Some("RecordScreen exec is not exposed as a public tool"),
    );

    let choices = error_chunk
        .get("choices")
        .and_then(Value::as_array)
        .expect("choices array present");
    assert!(choices.is_empty(), "choices must be empty, got {choices:?}");
    assert!(ctx.failed);

    let serialized = chunks
        .iter()
        .map(|c| c.to_string())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        !serialized.contains("[DONE]"),
        "adapter must not emit [DONE]; route layer owns that. got: {serialized}",
    );
}

#[test]
fn messages_refuse_emits_native_error_event() {
    let mut ctx = cursor_messages::MessagesContext::new("composer-2");
    let frames = cursor_messages::emit_event(
        &CursorAgentEvent::ProviderError {
            code: "unsupported_exec_kind".to_string(),
            message: "RecordScreen exec is not exposed as a public tool".to_string(),
            cursor_request_id: Some("req_xyz".to_string()),
        },
        &mut ctx,
    );

    let error_frame = frames
        .iter()
        .find(|f| f.event == "error")
        .expect("event: error frame present");

    assert_eq!(
        error_frame.data.get("type").and_then(Value::as_str),
        Some("error"),
    );
    let error = error_frame
        .data
        .get("error")
        .expect("error envelope present");
    assert_eq!(
        error.get("type").and_then(Value::as_str),
        Some("unsupported_exec_kind"),
    );
    assert_eq!(
        error.get("message").and_then(Value::as_str),
        Some("RecordScreen exec is not exposed as a public tool"),
    );

    let wire = error_frame.to_wire();
    assert!(wire.starts_with("event: error\n"), "wire: {wire}");
    assert!(wire.contains("\"type\":\"error\""), "wire: {wire}");
    assert!(ctx.failed);
}
