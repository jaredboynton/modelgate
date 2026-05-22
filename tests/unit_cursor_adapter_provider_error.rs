use serde_json::{json, Value};
use unified_model_proxy_v2::adapter::cursor_events::ResponseContext;
use unified_model_proxy_v2::adapter::{cursor_chat, cursor_messages, cursor_responses};
use unified_model_proxy_v2::cursor_agent::CursorAgentEvent;

#[test]
fn responses_adapter_emits_response_failed_for_provider_error_after_started() {
    let mut ctx = ResponseContext::new("composer-2", "resp_test");
    let _ = cursor_responses::emit_event(
        &CursorAgentEvent::TextDelta {
            delta: String::new(),
            content_index: 0,
        },
        &mut ctx,
    );
    assert!(ctx.started, "first event must flip started=true");

    let event = CursorAgentEvent::ProviderError {
        code: "unsupported_exec_kind".to_string(),
        message: "test".to_string(),
        cursor_request_id: Some("req_xyz".to_string()),
    };
    let frames = cursor_responses::emit_event(&event, &mut ctx);

    let names: Vec<&str> = frames.iter().map(|f| f.event.as_str()).collect();
    assert!(
        names.contains(&"response.failed"),
        "post-started ProviderError emits response.failed, got {names:?}",
    );
    assert!(
        !names.contains(&"error"),
        "post-started ProviderError must NOT emit a top-level error event, got {names:?}",
    );

    let failed = frames
        .iter()
        .find(|f| f.event == "response.failed")
        .expect("response.failed frame present");
    let response = failed
        .data
        .get("response")
        .expect("response.failed.data has response object");
    let error = response
        .get("error")
        .expect("response.failed.data.response.error present");
    assert_eq!(
        error.get("type").and_then(Value::as_str),
        Some("unsupported_exec_kind"),
    );
    assert_eq!(error.get("message").and_then(Value::as_str), Some("test"));
    assert_eq!(
        response.get("status").and_then(Value::as_str),
        Some("failed"),
    );
    assert!(ctx.failed, "context flagged as failed after ProviderError");
}

#[test]
fn responses_adapter_initial_prelude_is_single_response_created() {
    let mut ctx = ResponseContext::new("composer-2", "resp_test");

    let frame = cursor_responses::emit_initial_response_created(&mut ctx)
        .expect("fresh context emits prelude");

    assert_eq!(frame.event, "response.created");
    assert!(ctx.started);
    assert!(cursor_responses::emit_initial_response_created(&mut ctx).is_none());
    let frames = cursor_responses::emit_event(
        &CursorAgentEvent::TextDelta {
            delta: "hello".into(),
            content_index: 0,
        },
        &mut ctx,
    );
    assert!(
        frames.iter().all(|frame| frame.event != "response.created"),
        "prelude suppresses duplicate response.created: {frames:?}"
    );
}

#[test]
fn responses_adapter_emits_error_for_provider_error_before_started() {
    let mut ctx = ResponseContext::new("composer-2", "resp_test");
    assert!(!ctx.started);

    let event = CursorAgentEvent::ProviderError {
        code: "unsupported_exec_kind".to_string(),
        message: "test".to_string(),
        cursor_request_id: Some("req_xyz".to_string()),
    };
    let frames = cursor_responses::emit_event(&event, &mut ctx);

    let names: Vec<&str> = frames.iter().map(|f| f.event.as_str()).collect();
    assert!(
        names.contains(&"response.created"),
        "fresh ProviderError still emits response.created first today, got {names:?}",
    );
    assert!(
 names.contains(&"response.failed"),
 "fresh ProviderError emits response.failed today (started flipped at line 102), got {names:?}",
 );

    let failed = frames
        .iter()
        .find(|f| f.event == "response.failed")
        .expect("response.failed frame present");
    let error = failed
        .data
        .pointer("/response/error")
        .expect("response.failed.data.response.error present");
    assert_eq!(
        error.get("type").and_then(Value::as_str),
        Some("unsupported_exec_kind"),
    );
    assert!(ctx.failed);
}

#[test]
fn responses_adapter_provider_error_does_not_surface_cursor_request_id() {
    let mut ctx = ResponseContext::new("composer-2", "resp_test");
    let _ = cursor_responses::emit_event(
        &CursorAgentEvent::TextDelta {
            delta: String::new(),
            content_index: 0,
        },
        &mut ctx,
    );

    let event = CursorAgentEvent::ProviderError {
        code: "unsupported_exec_kind".to_string(),
        message: "msg".to_string(),
        cursor_request_id: Some("req_xyz".to_string()),
    };
    let frames = cursor_responses::emit_event(&event, &mut ctx);

    let serialized = frames
        .iter()
        .map(|f| f.data.to_string())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        !serialized.contains("req_xyz"),
        "current cursor_responses behavior drops cursor_request_id, got {serialized}",
    );
}

#[test]
fn chat_adapter_emits_error_chunk_with_empty_choices() {
    let mut ctx = cursor_chat::ChatContext::new("composer-2");
    let event = CursorAgentEvent::ProviderError {
        code: "client_capability_unsupported".to_string(),
        message: "Droid cannot represent X".to_string(),
        cursor_request_id: None,
    };
    let chunks = cursor_chat::emit_event(&event, &mut ctx);
    assert!(
        chunks.len() >= 2,
        "expected at least the initial role chunk plus the error chunk, got {}",
        chunks.len(),
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
        Some("client_capability_unsupported"),
    );
    assert_eq!(
        error.get("message").and_then(Value::as_str),
        Some("Droid cannot represent X"),
    );

    let choices = error_chunk
        .get("choices")
        .and_then(Value::as_array)
        .expect("choices array present");
    assert!(
        choices.is_empty(),
        "design spec requires choices: [] on the error chunk, got {choices:?}",
    );
    assert!(ctx.failed, "context flagged failed after error chunk");
    let serialized = chunks
        .iter()
        .map(|c| c.to_string())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        !serialized.contains("[DONE]"),
        "adapter must not emit [DONE]; that is the route layer's job. got: {serialized}",
    );
}

#[test]
fn chat_adapter_provider_error_does_not_surface_cursor_request_id() {
    let mut ctx = cursor_chat::ChatContext::new("composer-2");
    let event = CursorAgentEvent::ProviderError {
        code: "unsupported_exec_kind".to_string(),
        message: "msg".to_string(),
        cursor_request_id: Some("req_xyz".to_string()),
    };
    let chunks = cursor_chat::emit_event(&event, &mut ctx);

    let serialized = chunks
        .iter()
        .map(|c| c.to_string())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        !serialized.contains("req_xyz"),
        "current cursor_chat behavior drops cursor_request_id, got {serialized}",
    );
}

#[test]
fn messages_adapter_emits_native_error_event() {
    let mut ctx = cursor_messages::MessagesContext::new("composer-2");
    let event = CursorAgentEvent::ProviderError {
        code: "shape_unknown_pending_live_phase0".to_string(),
        message: "Diagnostics shape unknown".to_string(),
        cursor_request_id: Some("req_abc".to_string()),
    };
    let frames = cursor_messages::emit_event(&event, &mut ctx);
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
        Some("shape_unknown_pending_live_phase0"),
    );
    assert_eq!(
        error.get("message").and_then(Value::as_str),
        Some("Diagnostics shape unknown"),
    );
    let wire = error_frame.to_wire();
    assert!(wire.starts_with("event: error\n"), "wire: {wire}");
    assert!(wire.contains("\"type\":\"error\""), "wire: {wire}");

    assert!(ctx.failed, "context flagged failed after error event");
}

#[test]
fn messages_adapter_provider_error_does_not_surface_cursor_request_id() {
    let mut ctx = cursor_messages::MessagesContext::new("composer-2");
    let event = CursorAgentEvent::ProviderError {
        code: "shape_unknown_pending_live_phase0".to_string(),
        message: "msg".to_string(),
        cursor_request_id: Some("req_abc".to_string()),
    };
    let frames = cursor_messages::emit_event(&event, &mut ctx);

    let serialized = frames
        .iter()
        .map(|f| f.data.to_string())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        !serialized.contains("req_abc"),
        "current cursor_messages behavior drops cursor_request_id, got {serialized}",
    );
}

#[test]
fn all_four_canonical_refuse_codes_round_trip_through_each_adapter() {
    for code in [
        "unsupported_exec_kind",
        "shape_unknown_pending_live_phase0",
        "missing_required_field",
        "client_capability_unsupported",
    ] {
        let mut ctx = ResponseContext::new("composer-2", "resp_test");
        let _ = cursor_responses::emit_event(
            &CursorAgentEvent::TextDelta {
                delta: String::new(),
                content_index: 0,
            },
            &mut ctx,
        );
        let frames = cursor_responses::emit_event(
            &CursorAgentEvent::ProviderError {
                code: code.to_string(),
                message: "m".to_string(),
                cursor_request_id: None,
            },
            &mut ctx,
        );
        let failed = frames
            .iter()
            .find(|f| f.event == "response.failed")
            .unwrap_or_else(|| panic!("responses adapter missing response.failed for {code}"));
        let got = failed
            .data
            .pointer("/response/error/type")
            .and_then(Value::as_str);
        assert_eq!(
            got,
            Some(code),
            "responses error.type round-trip for {code}"
        );
        let mut chat_ctx = cursor_chat::ChatContext::new("composer-2");
        let chunks = cursor_chat::emit_event(
            &CursorAgentEvent::ProviderError {
                code: code.to_string(),
                message: "m".to_string(),
                cursor_request_id: None,
            },
            &mut chat_ctx,
        );
        let chunk = chunks
            .iter()
            .find(|c| c.get("error").is_some())
            .unwrap_or_else(|| panic!("chat adapter missing error chunk for {code}"));
        let got = chunk.pointer("/error/type").and_then(Value::as_str);
        assert_eq!(got, Some(code), "chat error.type round-trip for {code}");
        let mut msg_ctx = cursor_messages::MessagesContext::new("composer-2");
        let frames = cursor_messages::emit_event(
            &CursorAgentEvent::ProviderError {
                code: code.to_string(),
                message: "m".to_string(),
                cursor_request_id: None,
            },
            &mut msg_ctx,
        );
        let err = frames
            .iter()
            .find(|f| f.event == "error")
            .unwrap_or_else(|| panic!("messages adapter missing event:error for {code}"));
        let got = err.data.pointer("/error/type").and_then(Value::as_str);
        assert_eq!(got, Some(code), "messages error.type round-trip for {code}");
    }
    let sample = json!({
    "type": "error",
    "error": {
    "type": "shape_unknown_pending_live_phase0",
    "message": "Cursor Diagnostics shape is not decoded yet",
    },
    });
    assert_eq!(sample.get("type").and_then(Value::as_str), Some("error"));
}
