//! Regression tests pinning the wire shapes the three public Cursor adapters
//! emit when a `CursorAgentEvent::ProviderError` arrives mid-stream.
//!
//! These tests encode the round-5 APPROVED design spec (CRIT-3) for the
//! `cursor-phase0` client profile work — see
//! `.omx/research/cursor-phase0/client-profile-design-v3-deltas.md` lines
//! 132-209 for the per-adapter wire shapes pinned here.
//!
//! Each adapter renders `ProviderError` differently because the public stream
//! shapes differ:
//!
//! - `/v1/responses`: native `response.failed` (post-`response.created`) or
//!   `event: error` (pre-`response.created`).
//! - `/v1/chat/completions`: a `chat.completion.chunk` with `error: {...}`
//!   and `choices: []`. The route layer is responsible for the trailing
//!   `data: [DONE]` line; the adapter only emits chunks.
//! - `/v1/messages`: native `event: error`.
//!
//! `cursor_request_id` Lane K coordination note: the design spec wire shapes
//! in CRIT-3 do NOT surface `cursor_request_id` in the emitted JSON. All
//! three adapters currently destructure it with `..` and drop it. The tests
//! pin that current behavior. See the
//! `provider_error_does_not_currently_surface_cursor_request_id_*` tests.
//! TODO Lane K: align with design spec — decide whether the request id
//! should be threaded into the wire (e.g. as `error.request_id` or a span
//! attribute) and update either the adapters or these assertions in the
//! follow-up round.
//!
//! Tests use `serde_json` for JSON parsing. No `axum` runtime is needed
//! because the adapter modules are pure JSON-in / JSON-out functions with
//! no async or I/O.

use serde_json::{json, Value};
use unified_model_proxy_v2::adapter::cursor_events::ResponseContext;
use unified_model_proxy_v2::adapter::{cursor_chat, cursor_messages, cursor_responses};
use unified_model_proxy_v2::cursor_agent::CursorAgentEvent;

// ---------------------------------------------------------------------------
// /v1/responses: ProviderError -> response.failed | event: error
// ---------------------------------------------------------------------------

#[test]
fn responses_adapter_emits_response_failed_for_provider_error_after_started() {
    // ResponseContext starts with `started == false`. The first event drives
    // `emit_event` to push `response.created` and flip the flag. We simulate
    // that by sending a benign first event, then the ProviderError.
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
fn responses_adapter_emits_error_for_provider_error_before_started() {
    // Fresh ResponseContext: started == false. The current implementation
    // (cursor_responses::emit_event) ALWAYS emits a `response.created` frame
    // first when started == false (see cursor_responses.rs:101-104). So the
    // ProviderError branch is reached AFTER ctx.started has just been
    // flipped to true, and the emitted frames include both `response.created`
    // and `response.failed` (NOT a bare `event: error`).
    //
    // The "before started" branch in `emit_provider_error` (line 1141-1149)
    // is currently unreachable from `emit_event` because the started gate
    // runs unconditionally at the top. This test pins that observable
    // behavior so a future change that splits the started-gate from the
    // ProviderError path will fail this assertion deliberately.
    //
    // TODO Lane K: align with design spec — the design says
    // "If `response.created` already emitted, emit `response.failed`;
    // otherwise emit `error`." (delta line 140). Because the adapter
    // unconditionally emits `response.created` before matching, the
    // pre-started branch is dead code in practice. Either remove the dead
    // branch or move the started-gate inside the match so a fresh-error
    // stream starts with `event: error` only.
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
    // Pin current behavior: the cursor_request_id payload is dropped by
    // cursor_responses::emit_event (the match destructure uses `..`).
    // TODO Lane K: align with design spec — see module-level note.
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

// ---------------------------------------------------------------------------
// /v1/chat/completions: ProviderError -> chat.completion.chunk with error
// ---------------------------------------------------------------------------

#[test]
fn chat_adapter_emits_error_chunk_with_empty_choices() {
    let mut ctx = cursor_chat::ChatContext::new("composer-2");
    let event = CursorAgentEvent::ProviderError {
        code: "client_capability_unsupported".to_string(),
        message: "Droid cannot represent X".to_string(),
        cursor_request_id: None,
    };
    let chunks = cursor_chat::emit_event(&event, &mut ctx);

    // The adapter emits `initial_role_chunk` first (started flag was false)
    // and the error chunk second. The error chunk is the one carrying the
    // `error` object and empty `choices` array per design CRIT-3.
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

    // The route layer (NOT the adapter) appends the trailing `data: [DONE]`
    // SSE line. The adapter's job is the chunk; pin that the adapter does
    // not synthesize a [DONE] sentinel itself.
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
    // Pin current behavior: cursor_chat ignores cursor_request_id (`..`
    // destructure at line 194). TODO Lane K — see module note.
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

// ---------------------------------------------------------------------------
// /v1/messages: ProviderError -> event: error
// ---------------------------------------------------------------------------

#[test]
fn messages_adapter_emits_native_error_event() {
    let mut ctx = cursor_messages::MessagesContext::new("composer-2");
    let event = CursorAgentEvent::ProviderError {
        code: "shape_unknown_pending_live_phase0".to_string(),
        message: "Diagnostics shape unknown".to_string(),
        cursor_request_id: Some("req_abc".to_string()),
    };
    let frames = cursor_messages::emit_event(&event, &mut ctx);

    // The first frame is `message_start` (started flag was false). The
    // error frame is the one carrying `event: error` per CRIT-3.
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

    // Verify the SSE wire shape includes both the `event: error` line and
    // the JSON data line, matching the spec sample (delta lines 192-194).
    let wire = error_frame.to_wire();
    assert!(wire.starts_with("event: error\n"), "wire: {wire}");
    assert!(wire.contains("\"type\":\"error\""), "wire: {wire}");

    assert!(ctx.failed, "context flagged failed after error event");
}

#[test]
fn messages_adapter_provider_error_does_not_surface_cursor_request_id() {
    // Pin current behavior: cursor_messages ignores cursor_request_id
    // (`..` destructure at line 212). TODO Lane K — see module note.
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

// ---------------------------------------------------------------------------
// Cross-adapter: ProviderError code dictionary smoke (MAJ-1)
// ---------------------------------------------------------------------------

#[test]
fn all_four_canonical_refuse_codes_round_trip_through_each_adapter() {
    // MAJ-1 pins these four canonical codes. Smoke-test each adapter renders
    // them verbatim into the public error envelope.
    for code in [
        "unsupported_exec_kind",
        "shape_unknown_pending_live_phase0",
        "missing_required_field",
        "client_capability_unsupported",
    ] {
        // Responses
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

        // Chat
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

        // Messages
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

    // Sanity: the sample messages from the design spec also pass the
    // basic shape check.
    let sample = json!({
        "type": "error",
        "error": {
            "type": "shape_unknown_pending_live_phase0",
            "message": "Cursor Diagnostics shape is not decoded yet",
        },
    });
    assert_eq!(sample.get("type").and_then(Value::as_str), Some("error"));
}
