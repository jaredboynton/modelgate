//! Wiring test: the Cursor run engine dispatches public tool events
//! through the per-profile renderer.
//!
//! These cases assert that `public_tool_events_for_exec` honors the
//! `ClientProfile` argument: refusing profiles emit a single
//! `ProviderError`, while `GenericOpenAi` keeps the legacy 3-event
//! tool-call sequence.

use unified_model_proxy_v2::cursor_agent::CursorAgentEvent;
use unified_model_proxy_v2::upstream::cursor::client_profile::ClientProfile;
use unified_model_proxy_v2::upstream::cursor::proto::{encode_string_field, ExecKind, ExecRequest};
use unified_model_proxy_v2::upstream::cursor::run::{
    pending_tool_call_for_exec, public_tool_events_for_exec,
};

fn record_screen_exec() -> ExecRequest {
    ExecRequest {
        id: 1,
        exec_id: "exec-record".to_string(),
        kind: ExecKind::RecordScreen,
        args: Vec::new(),
    }
}

fn read_exec() -> ExecRequest {
    ExecRequest {
        id: 2,
        exec_id: "exec-read".to_string(),
        kind: ExecKind::Read,
        args: encode_string_field(1, "src/cursor_agent.rs"),
    }
}

fn assert_single_provider_error(events: &[CursorAgentEvent], exec_id: &str) {
    assert_eq!(
        events.len(),
        1,
        "refuse profile must collapse to one event, got {events:?}"
    );
    match &events[0] {
        CursorAgentEvent::ProviderError {
            code,
            cursor_request_id,
            ..
        } => {
            assert_eq!(code, "unsupported_exec_kind");
            assert_eq!(cursor_request_id.as_deref(), Some(exec_id));
        }
        other => panic!("expected ProviderError, got {other:?}"),
    }
}

#[test]
fn record_screen_refused_for_claude_code() {
    let exec = record_screen_exec();
    let events = public_tool_events_for_exec(ClientProfile::ClaudeCode, &exec);
    assert_single_provider_error(&events, "exec-record");
}

#[test]
fn record_screen_refused_for_codex_cli() {
    let exec = record_screen_exec();
    let events = public_tool_events_for_exec(ClientProfile::CodexCli, &exec);
    assert_single_provider_error(&events, "exec-record");
}

#[test]
fn record_screen_refused_for_droid() {
    let exec = record_screen_exec();
    let events = public_tool_events_for_exec(ClientProfile::Droid, &exec);
    assert_single_provider_error(&events, "exec-record");
}

#[test]
fn read_emits_three_events_for_generic_openai() {
    let exec = read_exec();
    let events = public_tool_events_for_exec(ClientProfile::GenericOpenAi, &exec);
    assert_eq!(
        events.len(),
        3,
        "generic_openai must emit Started/Delta/Done"
    );

    match &events[0] {
        CursorAgentEvent::ToolCallStarted { call_id, name, .. } => {
            assert_eq!(call_id, "exec-read");
            assert_eq!(name, "read");
        }
        other => panic!("expected ToolCallStarted, got {other:?}"),
    }
    match &events[1] {
        CursorAgentEvent::ToolCallArgumentsDelta { call_id, .. } => {
            assert_eq!(call_id, "exec-read");
        }
        other => panic!("expected ToolCallArgumentsDelta, got {other:?}"),
    }
    match &events[2] {
        CursorAgentEvent::ToolCallDone { call_id, arguments } => {
            assert_eq!(call_id, "exec-read");
            assert_eq!(arguments["path"], "src/cursor_agent.rs");
        }
        other => panic!("expected ToolCallDone, got {other:?}"),
    }
}

#[test]
fn pending_tool_call_emits_for_generic_openai_read() {
    let exec = read_exec();
    let pending = pending_tool_call_for_exec(ClientProfile::GenericOpenAi, &exec)
        .expect("emit profile must yield a pending tool call");
    assert_eq!(pending.id, "exec-read");
    assert_eq!(pending.name, "read");
    assert_eq!(pending.arguments["path"], "src/cursor_agent.rs");
}

#[test]
fn pending_tool_call_none_for_refusing_profiles() {
    let exec = record_screen_exec();
    for profile in [
        ClientProfile::ClaudeCode,
        ClientProfile::CodexCli,
        ClientProfile::Droid,
    ] {
        assert!(
            pending_tool_call_for_exec(profile, &exec).is_none(),
            "profile {profile:?} must refuse RecordScreen and yield no pending call"
        );
    }
}
