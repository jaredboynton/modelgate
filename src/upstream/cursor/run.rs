//! Cursor agent run engine.
//!
//! Orchestrates a single Cursor `Run` RPC: resolves credentials, encodes
//! the AgentRunRequest, opens the streaming Connect bridge via the
//! transport layer, decodes server frames into neutral `CursorAgentEvent`s,
//! and persists continuation state in `CursorSessionStore`. Heartbeats are
//! owned by `RunStream` and need no driving from this layer.
//!
//! Per ralplan Section 4 plan items 6-14, this engine is the only place
//! Cursor wire types collide with `AppState` wiring, and even here only
//! through a thin `&AppState` borrow used to reach the session store and
//! credential helpers.
//!
//! Returns an `impl Stream<Item = CursorAgentEvent>`. Callers (the public
//! adapters) translate these neutral events into route-specific SSE.

use std::collections::HashMap;

use bytes::Bytes;
use futures::{stream, Stream};
use tokio::sync::mpsc;
use tracing::debug;
use uuid::Uuid;

use crate::{
    auth::cursor::resolve_cursor_credentials,
    cursor_agent::{
        CursorAgentEvent, CursorAgentRequest, CursorContentBlock, CursorFinishReason,
        CursorMessage, CursorToolKind,
    },
    upstream::cursor::{
        client_profile::ClientProfile,
        connect::frame_connect_message,
        profiles::{native_tools::tools_visible_to_cursor, render_tool_call, RenderedToolCall},
        proto::{
            decode_agent_server_message, decode_exec_public_tool_call, decode_get_blob_args,
            decode_set_blob_args, encode_agent_run_request, encode_get_blob_result,
            encode_request_context_result, encode_set_blob_result, AgentRunRequestInput,
            InteractionEvent, KvKind, Message, RequestedModelInput, RequestedModelParameter,
        },
        session::ConversationState,
        transport::open_streaming_run,
    },
    AppState,
};

const EVENT_CHANNEL_CAPACITY: usize = 64;

/// Spawns the Cursor run pipeline and returns a stream of neutral events.
///
/// The stream terminates with either `CursorAgentEvent::Done` (graceful) or
/// `CursorAgentEvent::ProviderError` (transport, decode, or end-stream
/// failure). Dropping the stream cancels the upstream task on the next
/// channel send.
pub async fn run(
    state: &AppState,
    request: CursorAgentRequest,
) -> impl Stream<Item = CursorAgentEvent> {
    let (tx, rx) = mpsc::channel(EVENT_CHANNEL_CAPACITY);

    let credentials = match resolve_cursor_credentials(state.auth_home.as_path()).await {
        Ok(creds) => creds,
        Err(err) => {
            let _ = tx
                .send(CursorAgentEvent::ProviderError {
                    code: "missing_credential".to_string(),
                    message: format!("cursor credential resolution failed: {err}"),
                    cursor_request_id: None,
                })
                .await;
            return receiver_into_stream(rx);
        }
    };

    let proto_messages = build_proto_messages(&request);
    let message_id = Uuid::new_v4().to_string();
    let conversation_id_out = request
        .continuation_key
        .as_ref()
        .map(|k| k.conversation_id.clone())
        .unwrap_or_else(|| format!("conv_{}", Uuid::new_v4().simple()));
    let workspace_path = request
        .workspace
        .as_ref()
        .map(|w| w.root.to_string_lossy().to_string())
        .unwrap_or_default();
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/zsh".into());
    let os_version = std::env::consts::OS.to_string();

    let requested_parameters =
        cursor_requested_model_parameters(&request.model, &request.upstream_model);
    let requested_model = RequestedModelInput {
        model_id: &request.upstream_model,
        max_mode: false,
        parameters: &requested_parameters,
    };
    let cursor_visible_tools =
        tools_visible_to_cursor(request.client_profile.into(), &request.tools);

    let body = encode_agent_run_request(AgentRunRequestInput {
        model: &request.model,
        requested_model: Some(requested_model),
        messages: &proto_messages,
        message_id: &message_id,
        conversation_id: Some(conversation_id_out.as_str()),
        os_version: &os_version,
        workspace_path: &workspace_path,
        shell: &shell,
        tools: &cursor_visible_tools,
    });

    let cursor_sessions = state.cursor_sessions.clone();
    let continuation_key = request.continuation_key.clone();

    tokio::spawn(async move {
        let response_id = format!("resp_{}", Uuid::new_v4().simple());
        let mut pending_tool_calls: Vec<crate::cursor_agent::CursorToolCall> =
            match continuation_key.as_ref() {
                Some(key) => cursor_sessions
                    .lookup_continuation(key)
                    .map(|state| state.pending_tool_calls)
                    .unwrap_or_default(),
                None => Vec::new(),
            };
        let mut finish_reason = CursorFinishReason::Stop;
        let content_index: u32 = 0;
        let mut blob_store: HashMap<String, Vec<u8>> = HashMap::new();

        let mut transport_stream = match open_streaming_run(&credentials.access_token, body).await {
            Ok(stream) => stream,
            Err(err) => {
                let _ = tx
                    .send(CursorAgentEvent::ProviderError {
                        code: "transport_open_failed".to_string(),
                        message: format!("open run stream: {err}"),
                        cursor_request_id: None,
                    })
                    .await;
                return;
            }
        };

        let cursor_request_id = transport_stream.request_id().to_string();

        while let Some(payload) = transport_stream.next_frame().await {
            let server_message = decode_agent_server_message(&payload);

            for event in server_message.events {
                let cursor_event = match event {
                    InteractionEvent::Text(delta) => Some(CursorAgentEvent::TextDelta {
                        delta,
                        content_index,
                    }),
                    InteractionEvent::Thinking(delta) => {
                        Some(CursorAgentEvent::ReasoningDelta { delta })
                    }
                    InteractionEvent::TokenDelta(tokens) => Some(CursorAgentEvent::UsageUpdate {
                        input_tokens: 0,
                        output_tokens: tokens.max(0) as u64,
                        reasoning_tokens: None,
                    }),
                    InteractionEvent::Heartbeat => None,
                    InteractionEvent::TurnEnded => {
                        finish_reason = match finish_reason {
                            CursorFinishReason::ToolCalls => CursorFinishReason::ToolCalls,
                            _ => CursorFinishReason::Stop,
                        };
                        None
                    }
                };

                if let Some(event) = cursor_event {
                    if tx.send(event).await.is_err() {
                        debug!("cursor run consumer dropped; cancelling stream");
                        let _ = transport_stream.close().await;
                        return;
                    }
                }
            }

            let pending_checkpoint_id = if server_message.saw_checkpoint {
                Some(
                    server_message
                        .checkpoint_update
                        .as_ref()
                        .map(|bytes| hex_lower(bytes))
                        .unwrap_or_else(|| "empty-checkpoint".to_string()),
                )
            } else {
                None
            };

            let mut emitted_public_tool_call = false;
            for exec in &server_message.exec_requests {
                debug!(
                    exec_id = %exec.exec_id,
                    kind = ?exec.kind,
                    "cursor exec request received",
                );
                match exec.kind {
                    crate::upstream::cursor::proto::ExecKind::RequestContext => {
                        let response = encode_request_context_result(
                            exec,
                            &cursor_visible_tools,
                            &os_version,
                            &workspace_path,
                            &shell,
                        );
                        if send_cursor_client_frame(&mut transport_stream, response)
                            .await
                            .is_err()
                        {
                            let _ = tx
                                .send(CursorAgentEvent::ProviderError {
                                    code: "exec_response_failed".to_string(),
                                    message: "cursor request-context response failed".to_string(),
                                    cursor_request_id: Some(cursor_request_id.clone()),
                                })
                                .await;
                            let _ = transport_stream.close().await;
                            return;
                        }
                    }
                    _ => {
                        if let Some(response) = maybe_cursor_codebase_search_result(
                            exec,
                            &request,
                            &credentials.access_token,
                        )
                        .await
                        {
                            if send_cursor_client_frame(&mut transport_stream, response)
                                .await
                                .is_err()
                            {
                                let _ = tx
                                    .send(CursorAgentEvent::ProviderError {
                                        code: "index_response_failed".to_string(),
                                        message: "cursor codebase search response failed"
                                            .to_string(),
                                        cursor_request_id: Some(cursor_request_id.clone()),
                                    })
                                    .await;
                                let _ = transport_stream.close().await;
                                return;
                            }
                            continue;
                        }
                        finish_reason = CursorFinishReason::ToolCalls;
                        emitted_public_tool_call = true;
                        if let Some(call) =
                            pending_tool_call_for_exec(request.client_profile.into(), exec)
                        {
                            pending_tool_calls.push(call);
                        }
                        for event in
                            public_tool_events_for_exec(request.client_profile.into(), exec)
                        {
                            if tx.send(event).await.is_err() {
                                let _ = transport_stream.close().await;
                                return;
                            }
                        }
                    }
                }
            }
            for kv in &server_message.kv_requests {
                if let Some(response) = handle_kv_request(kv, &mut blob_store) {
                    let frame = frame_connect_message(&response, 0);
                    if let Err(err) = transport_stream.send_frame(Bytes::from(frame), false).await {
                        let _ = tx
                            .send(CursorAgentEvent::ProviderError {
                                code: "kv_response_failed".to_string(),
                                message: format!("cursor kv response failed: {err}"),
                                cursor_request_id: Some(cursor_request_id.clone()),
                            })
                            .await;
                        let _ = transport_stream.close().await;
                        return;
                    }
                } else {
                    debug!(
                        kv_id = kv.id,
                        kind = ?kv.kind,
                        "cursor kv request ignored",
                    );
                }
            }
            if let Some(checkpoint_id) = pending_checkpoint_id {
                if let Some(key) = continuation_key.as_ref() {
                    let conv_state = ConversationState {
                        checkpoint: Some(checkpoint_id.clone()),
                        pending_tool_calls: pending_tool_calls.clone(),
                        last_access: std::time::Instant::now(),
                        route: key.route,
                        provider: key.provider,
                        upstream_model: key.upstream_model.clone(),
                        target_format: key.target_format,
                        client_profile: request.client_profile,
                        stable_field_hash: [0u8; 32],
                        response_id: response_id.clone(),
                        conversation_id: conversation_id_out.clone(),
                        blob_store: blob_store.clone(),
                    };
                    cursor_sessions.store_continuation(key, conv_state);
                }
                let _ = tx
                    .send(CursorAgentEvent::Checkpoint { checkpoint_id })
                    .await;
            }

            if emitted_public_tool_call {
                if let Some(key) = continuation_key.as_ref() {
                    let conv_state = ConversationState {
                        checkpoint: None,
                        pending_tool_calls: pending_tool_calls.clone(),
                        last_access: std::time::Instant::now(),
                        route: key.route,
                        provider: key.provider,
                        upstream_model: key.upstream_model.clone(),
                        target_format: key.target_format,
                        client_profile: request.client_profile,
                        stable_field_hash: [0u8; 32],
                        response_id: response_id.clone(),
                        conversation_id: conversation_id_out.clone(),
                        blob_store: blob_store.clone(),
                    };
                    cursor_sessions.store_continuation(key, conv_state);
                }
                let _ = transport_stream.close().await;
                let _ = tx
                    .send(CursorAgentEvent::Done {
                        finish_reason,
                        response_id: response_id.clone(),
                        conversation_id: conversation_id_out.clone(),
                    })
                    .await;
                return;
            }
        }

        if let Some(connect_err) = transport_stream.take_connect_error().await {
            let _ = tx
                .send(CursorAgentEvent::ProviderError {
                    code: connect_err.code,
                    message: connect_err.message,
                    cursor_request_id: Some(cursor_request_id),
                })
                .await;
            return;
        }

        if let Some(key) = continuation_key.as_ref() {
            let conv_state = ConversationState {
                checkpoint: None,
                pending_tool_calls: pending_tool_calls.clone(),
                last_access: std::time::Instant::now(),
                route: key.route,
                provider: key.provider,
                upstream_model: key.upstream_model.clone(),
                target_format: key.target_format,
                client_profile: request.client_profile,
                stable_field_hash: [0u8; 32],
                response_id: response_id.clone(),
                conversation_id: conversation_id_out.clone(),
                blob_store: blob_store.clone(),
            };
            cursor_sessions.store_continuation(key, conv_state);
        }

        let _ = tx
            .send(CursorAgentEvent::Done {
                finish_reason,
                response_id,
                conversation_id: conversation_id_out,
            })
            .await;
    });

    receiver_into_stream(rx)
}

fn cursor_requested_model_parameters(
    model: &str,
    upstream_model: &str,
) -> Vec<RequestedModelParameter<'static>> {
    if model == "composer-2.5" {
        return vec![RequestedModelParameter {
            id: "fast",
            value: "false",
        }];
    }
    if model.starts_with("composer-") && model.ends_with("-fast") && model != upstream_model {
        return vec![RequestedModelParameter {
            id: "fast",
            value: "true",
        }];
    }
    Vec::new()
}

async fn maybe_cursor_codebase_search_result(
    exec: &crate::upstream::cursor::proto::ExecRequest,
    request: &CursorAgentRequest,
    token: &str,
) -> Option<Vec<u8>> {
    let (tool_name, tool_call_id, arguments) = decode_exec_public_tool_call(exec);
    if tool_name != "cursor_codebase_search" {
        return None;
    }
    let workspace = request.workspace.as_ref()?;
    let query = arguments
        .get("query")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default();
    let hits = crate::upstream::cursor::indexing::search(
        token,
        &workspace.root,
        query,
        &workspace.allowlist,
    )
    .await;
    let body = crate::upstream::cursor::indexing::render_body(&hits);
    Some(
        crate::upstream::cursor::indexing::encode_cursor_codebase_search_mcp_result(
            exec.id,
            if tool_call_id.trim().is_empty() {
                exec.exec_id.as_str()
            } else {
                tool_call_id.as_str()
            },
            &body,
        ),
    )
}

fn handle_kv_request(
    kv: &crate::upstream::cursor::proto::KvRequest,
    blob_store: &mut HashMap<String, Vec<u8>>,
) -> Option<Vec<u8>> {
    match kv.kind {
        KvKind::GetBlob => {
            let blob_id = decode_get_blob_args(&kv.args)?;
            let key = hex_lower(&blob_id);
            let body = blob_store.get(&key).map(Vec::as_slice);
            Some(encode_get_blob_result(kv.id, body))
        }
        KvKind::SetBlob => {
            let (blob_id, blob_data) = decode_set_blob_args(&kv.args)?;
            blob_store.insert(hex_lower(&blob_id), blob_data);
            Some(encode_set_blob_result(kv.id))
        }
        KvKind::Other(_) => None,
    }
}

/// Render a Cursor exec request as one or more neutral
/// `CursorAgentEvent`s, dispatching through the per-profile renderer.
///
/// Emit decisions yield the standard 3-event tool-call sequence
/// (`ToolCallStarted` / `ToolCallArgumentsDelta` / `ToolCallDone`). Refuse
/// decisions yield a single `ProviderError` event with the refuse code as
/// the error code and the human-readable reason as the message; the exec
/// id surfaces via `cursor_request_id` so adapters can correlate the
/// refusal with the originating exec.
pub fn public_tool_events_for_exec(
    profile: ClientProfile,
    exec: &crate::upstream::cursor::proto::ExecRequest,
) -> Vec<CursorAgentEvent> {
    match render_tool_call(profile, exec) {
        RenderedToolCall::Emit {
            tool_name,
            arguments,
            tool_call_id,
        } => {
            let call_id = if tool_call_id.trim().is_empty() {
                exec.exec_id.clone()
            } else {
                tool_call_id
            };
            let arguments_string = arguments.to_string();
            vec![
                CursorAgentEvent::ToolCallStarted {
                    call_id: call_id.clone(),
                    name: tool_name,
                    kind: CursorToolKind::Function,
                    argument_index: 0,
                },
                CursorAgentEvent::ToolCallArgumentsDelta {
                    call_id: call_id.clone(),
                    delta: arguments_string,
                },
                CursorAgentEvent::ToolCallDone { call_id, arguments },
            ]
        }
        RenderedToolCall::Refuse {
            exec_id,
            reason,
            code,
        } => vec![CursorAgentEvent::ProviderError {
            code: code.to_string(),
            message: reason,
            cursor_request_id: Some(exec_id),
        }],
    }
}

/// Build the `CursorToolCall` that mirrors a public tool-call emission for
/// continuation-state tracking. Returns `None` when the profile refuses the
/// exec, since refusals never round-trip via tool_results.
pub fn pending_tool_call_for_exec(
    profile: ClientProfile,
    exec: &crate::upstream::cursor::proto::ExecRequest,
) -> Option<crate::cursor_agent::CursorToolCall> {
    match render_tool_call(profile, exec) {
        RenderedToolCall::Emit {
            tool_name,
            arguments,
            tool_call_id,
        } => {
            let id = if tool_call_id.trim().is_empty() {
                exec.exec_id.clone()
            } else {
                tool_call_id
            };
            Some(crate::cursor_agent::CursorToolCall {
                id,
                name: tool_name,
                arguments,
            })
        }
        RenderedToolCall::Refuse { .. } => None,
    }
}

async fn send_cursor_client_frame(
    transport_stream: &mut crate::upstream::cursor::transport::RunStream,
    response: Vec<u8>,
) -> Result<(), crate::upstream::cursor::transport::TransportError> {
    let frame = frame_connect_message(&response, 0);
    transport_stream.send_frame(Bytes::from(frame), false).await
}

fn hex_lower(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(HEX[(byte >> 4) as usize] as char);
        out.push(HEX[(byte & 0x0f) as usize] as char);
    }
    out
}

fn receiver_into_stream<T: Send + 'static>(
    rx: mpsc::Receiver<T>,
) -> impl Stream<Item = T> + Send + 'static {
    stream::unfold(rx, |mut rx| async move {
        rx.recv().await.map(|item| (item, rx))
    })
}

/// Convert neutral `CursorMessage`s into the flat `proto::Message` shape the
/// AgentRunRequest encoder consumes today. The proto layer renders the full
/// log to a single user-prompt string, so role/text fidelity is what matters
/// here.
fn build_proto_messages(request: &CursorAgentRequest) -> Vec<Message> {
    let mut out = Vec::new();
    if let Some(system) = request.system_instructions.as_deref() {
        if !system.is_empty() {
            out.push(Message {
                role: "system".to_string(),
                text: system.to_string(),
            });
        }
    }
    if let Some(developer) = request.developer_instructions.as_deref() {
        if !developer.is_empty() {
            out.push(Message {
                role: "developer".to_string(),
                text: developer.to_string(),
            });
        }
    }
    for message in &request.messages {
        match message {
            CursorMessage::System { content } => out.push(Message {
                role: "system".to_string(),
                text: content.clone(),
            }),
            CursorMessage::Developer { content } => out.push(Message {
                role: "developer".to_string(),
                text: content.clone(),
            }),
            CursorMessage::User { blocks } => out.push(Message {
                role: "user".to_string(),
                text: flatten_blocks(blocks),
            }),
            CursorMessage::Assistant { blocks, tool_calls } => {
                let mut text = flatten_blocks(blocks);
                for call in tool_calls {
                    append_line(
                        &mut text,
                        &format!(
                            "Tool call {} named {} with arguments {}",
                            call.id, call.name, call.arguments
                        ),
                    );
                }
                out.push(Message {
                    role: "assistant".to_string(),
                    text,
                });
            }
        }
    }
    for result in &request.tool_results {
        let rendered = match &result.error {
            Some(error) => format!(
                "Tool result for {} failed: {error}. Output: {}",
                result.call_id, result.output
            ),
            None => format!("Tool result for {}: {}", result.call_id, result.output),
        };
        out.push(Message {
            role: "tool".to_string(),
            text: rendered,
        });
    }
    out
}

fn append_line(target: &mut String, line: &str) {
    if !target.is_empty() {
        target.push('\n');
    }
    target.push_str(line);
}

fn flatten_blocks(blocks: &[CursorContentBlock]) -> String {
    let mut text = String::new();
    for block in blocks {
        match block {
            CursorContentBlock::Text(value) => {
                if !text.is_empty() {
                    text.push('\n');
                }
                text.push_str(value);
            }
        }
    }
    text
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use super::{build_proto_messages, public_tool_events_for_exec};
    use crate::cursor_agent::{
        CursorAgentEvent, CursorAgentRequest, CursorContentBlock, CursorMessage, CursorToolCall,
        CursorToolResult, CursorWorkspaceContext,
    };
    use crate::upstream::cursor::client_profile::ClientProfile;
    use crate::upstream::cursor::proto::{
        encode_message_field, encode_string_field, exec_message, parse_proto_fields, ExecKind,
        ExecRequest,
    };

    #[test]
    fn build_proto_messages_preserves_tool_calls_and_results() {
        let request = CursorAgentRequest {
            model: "composer-2-fast".to_string(),
            upstream_model: "composer-2-fast".to_string(),
            system_instructions: None,
            developer_instructions: None,
            messages: vec![CursorMessage::Assistant {
                blocks: vec![CursorContentBlock::Text("I need a lookup.".to_string())],
                tool_calls: vec![CursorToolCall {
                    id: "call-grep".to_string(),
                    name: "grep".to_string(),
                    arguments: serde_json::json!({
                        "pattern": "CursorAgentRequest",
                        "path": "src"
                    }),
                }],
            }],
            tools: Vec::new(),
            tool_results: vec![CursorToolResult {
                call_id: "call-grep".to_string(),
                output: serde_json::json!({"matches": ["src/cursor_agent.rs"]}),
                error: None,
            }],
            continuation_key: None,
            workspace: None,
            stream: false,
            request_id: Uuid::nil(),
            client_profile: Default::default(),
        };

        let messages = build_proto_messages(&request);

        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].role, "assistant");
        assert!(messages[0].text.contains("Tool call call-grep named grep"));
        assert!(messages[0].text.contains("CursorAgentRequest"));
        assert_eq!(messages[1].role, "tool");
        assert!(messages[1].text.contains("Tool result for call-grep"));
        assert!(messages[1].text.contains("src/cursor_agent.rs"));
    }

    #[test]
    fn public_tool_events_for_multiple_execs_preserve_all_calls() {
        let grep = ExecRequest {
            id: 1,
            exec_id: "grep-exec".to_string(),
            kind: ExecKind::Grep,
            args: [
                encode_string_field(1, "CursorAgentRequest"),
                encode_string_field(2, "src"),
                encode_string_field(3, "files_with_matches"),
            ]
            .concat(),
        };
        let read = ExecRequest {
            id: 2,
            exec_id: "read-exec".to_string(),
            kind: ExecKind::Read,
            args: encode_string_field(1, "src/cursor_agent.rs"),
        };

        let mut events = Vec::new();
        events.extend(public_tool_events_for_exec(
            ClientProfile::GenericOpenAi,
            &grep,
        ));
        events.extend(public_tool_events_for_exec(
            ClientProfile::GenericOpenAi,
            &read,
        ));

        assert_eq!(events.len(), 6);
        match &events[0] {
            CursorAgentEvent::ToolCallStarted { call_id, name, .. } => {
                assert_eq!(call_id, "grep-exec");
                assert_eq!(name, "grep");
            }
            other => panic!("expected grep tool start, got {other:?}"),
        }
        match &events[2] {
            CursorAgentEvent::ToolCallDone { call_id, arguments } => {
                assert_eq!(call_id, "grep-exec");
                assert_eq!(arguments["pattern"], "CursorAgentRequest");
                assert_eq!(arguments["path"], "src");
            }
            other => panic!("expected grep tool done, got {other:?}"),
        }
        match &events[3] {
            CursorAgentEvent::ToolCallStarted { call_id, name, .. } => {
                assert_eq!(call_id, "read-exec");
                assert_eq!(name, "read");
            }
            other => panic!("expected read tool start, got {other:?}"),
        }
        match &events[5] {
            CursorAgentEvent::ToolCallDone { call_id, arguments } => {
                assert_eq!(call_id, "read-exec");
                assert_eq!(arguments["path"], "src/cursor_agent.rs");
            }
            other => panic!("expected read tool done, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn cursor_codebase_search_mcp_exec_is_intercepted_as_internal_result() {
        let workspace = tempfile::tempdir().expect("workspace");
        std::fs::create_dir(workspace.path().join("src")).expect("src");
        std::fs::write(
            workspace.path().join("src").join("cursor_agent.rs"),
            "pub struct CursorAgentRequest;",
        )
        .expect("source");
        let root = std::fs::canonicalize(workspace.path()).expect("canonical workspace");
        let argument_entry = [
            encode_string_field(1, "query"),
            encode_message_field(2, br#""CursorAgentRequest""#),
        ]
        .concat();
        let exec = ExecRequest {
            id: 42,
            exec_id: "exec-mcp".to_string(),
            kind: ExecKind::Mcp,
            args: [
                encode_message_field(2, &argument_entry),
                encode_string_field(3, "call-index"),
                encode_string_field(5, "cursor_codebase_search"),
            ]
            .concat(),
        };
        let request = CursorAgentRequest {
            model: "composer-2-fast".to_string(),
            upstream_model: "composer-2-fast".to_string(),
            system_instructions: None,
            developer_instructions: None,
            messages: Vec::new(),
            tools: Vec::new(),
            tool_results: Vec::new(),
            continuation_key: None,
            workspace: Some(CursorWorkspaceContext {
                root: root.clone(),
                worktree: Some(root.clone()),
                branch: None,
                remote: None,
                status_summary: None,
                index_metadata: None,
                allowlist: vec![root],
            }),
            stream: false,
            request_id: Uuid::nil(),
            client_profile: Default::default(),
        };

        let response = super::maybe_cursor_codebase_search_result(&exec, &request, "")
            .await
            .expect("internal mcp result");
        let outer = parse_proto_fields(&response);
        let exec_client = outer
            .iter()
            .find(|field| field.number == 2)
            .expect("exec client")
            .value
            .clone();
        let exec_fields = parse_proto_fields(&exec_client);
        let mcp_result = exec_fields
            .iter()
            .find(|field| field.number == exec_message::MCP_ARGS)
            .expect("mcp result")
            .value
            .clone();

        assert!(
            String::from_utf8_lossy(&mcp_result).contains("CursorAgentRequest"),
            "encoded MCP result includes indexed hit body"
        );
    }
}
