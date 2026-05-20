//! Manual protobuf wire layer for the Cursor AgentService narrow path.
//! Plus complete generated structures from agent_pb.ts.

pub mod mod_impl;
pub mod services;
pub mod types;

pub use mod_impl::*;

// ---------------------------------------------------------------------------
// AgentClientMessage / AgentRunRequest tags
// ---------------------------------------------------------------------------

/// Field tags inside `AgentClientMessage` (proto index 118).
pub mod agent_client_message {
    pub const RUN_REQUEST: u32 = 1;
    pub const EXEC_CLIENT_MESSAGE: u32 = 2;
    pub const KV_CLIENT_MESSAGE: u32 = 3;
    pub const CONVERSATION_ACTION: u32 = 4;
    pub const CLIENT_HEARTBEAT: u32 = 7;
}

/// Field tags inside `AgentRunRequest` (proto index 91).
pub mod agent_run_request {
    pub const CONVERSATION_STATE: u32 = 1;
    pub const ACTION: u32 = 2;
    pub const MODEL_DETAILS: u32 = 3;
    pub const MCP_TOOLS: u32 = 4;
    pub const CONVERSATION_ID: u32 = 5;
    pub const REQUESTED_MODEL: u32 = 9;
}

/// Field tags inside `ConversationAction` (proto index 54).
pub mod conversation_action {
    pub const USER_MESSAGE_ACTION: u32 = 1;
}

/// Field tags inside `UserMessageAction` (proto index 55).
pub mod user_message_action {
    pub const USER_MESSAGE: u32 = 1;
    pub const REQUEST_CONTEXT: u32 = 2;
}

/// Field tags inside `UserMessage` (proto index 63).
pub mod user_message {
    pub const TEXT: u32 = 1;
    pub const MESSAGE_ID: u32 = 2;
    pub const MODE: u32 = 4;
}

/// Field tags inside `ModelDetails` (proto index 88).
pub mod model_details {
    pub const MODEL_ID: u32 = 1;
    pub const THINKING_DETAILS: u32 = 2;
    pub const DISPLAY_MODEL_ID: u32 = 3;
    pub const DISPLAY_NAME: u32 = 4;
    pub const DISPLAY_NAME_SHORT: u32 = 5;
    pub const ALIASES: u32 = 6;
}

/// Field tags inside `RequestedModel` (proto index 89).
pub mod requested_model {
    pub const MODEL_ID: u32 = 1;
    pub const MAX_MODE: u32 = 2;
    pub const PARAMETERS: u32 = 3;
}

/// Field tags inside `RequestedModel.ModelParameter` (proto index 90).
pub mod requested_model_parameter {
    pub const ID: u32 = 1;
    pub const VALUE: u32 = 2;
}

/// Field tags inside `RequestContext` (proto index 342).
pub mod request_context {
    pub const ENV: u32 = 4;
    pub const TOOLS: u32 = 7;
    pub const MCP_INSTRUCTIONS: u32 = 14;
}

/// Field tags inside `RequestContextEnv` (proto index 343).
pub mod request_context_env {
    pub const OS_VERSION: u32 = 1;
    pub const WORKSPACE_PATHS: u32 = 2;
    pub const SHELL: u32 = 3;
    pub const PROJECT_FOLDER: u32 = 11;
}

/// Field tags inside `RequestContextResult` (proto index 336).
pub mod request_context_result {
    pub const SUCCESS: u32 = 1;
}

/// Field tags inside `RequestContextSuccess` (proto index 337).
pub mod request_context_success {
    pub const REQUEST_CONTEXT: u32 = 1;
}

mod mcp_tool_definition {
    pub const NAME: u32 = 1;
    pub const DESCRIPTION: u32 = 2;
    pub const INPUT_SCHEMA: u32 = 3;
    pub const PROVIDER_IDENTIFIER: u32 = 4;
    pub const TOOL_NAME: u32 = 5;
}

mod mcp_instructions {
    pub const SERVER_NAME: u32 = 1;
    pub const INSTRUCTIONS: u32 = 2;
}

mod protobuf_value {
    pub const NULL_VALUE: u32 = 1;
    pub const NUMBER_VALUE: u32 = 2;
    pub const STRING_VALUE: u32 = 3;
    pub const BOOL_VALUE: u32 = 4;
    pub const STRUCT_VALUE: u32 = 5;
    pub const LIST_VALUE: u32 = 6;
}

mod protobuf_struct {
    pub const FIELDS: u32 = 1;
}

mod protobuf_struct_field_entry {
    pub const KEY: u32 = 1;
    pub const VALUE: u32 = 2;
}

mod protobuf_list_value {
    pub const VALUES: u32 = 1;
}

/// Field tags inside `ConversationStateStructure` (proto index 83).
pub mod conversation_state {
    pub const TOKEN_DETAILS: u32 = 5;
}

// ---------------------------------------------------------------------------
// AgentServerMessage / ExecServerMessage / KvServerMessage / InteractionUpdate
// ---------------------------------------------------------------------------

/// Field tags inside `AgentServerMessage` (proto index 119).
pub mod agent_server_message {
    pub const INTERACTION_UPDATE: u32 = 1;
    pub const EXEC_SERVER_MESSAGE: u32 = 2;
    pub const CONVERSATION_CHECKPOINT_UPDATE: u32 = 3;
    pub const KV_SERVER_MESSAGE: u32 = 4;
}

/// Field tags inside `ExecServerMessage` / `ExecClientMessage` (proto indexes
/// 243 / 244). Field numbers match across the request/response pair.
pub mod exec_message {
    pub const ID: u32 = 1;
    pub const SHELL_ARGS: u32 = 2;
    pub const WRITE_ARGS: u32 = 3;
    pub const DELETE_ARGS: u32 = 4;
    pub const GREP_ARGS: u32 = 5;
    pub const READ_ARGS: u32 = 7;
    pub const LS_ARGS: u32 = 8;
    pub const DIAGNOSTICS_ARGS: u32 = 9;
    pub const REQUEST_CONTEXT_ARGS: u32 = 10;
    pub const MCP_ARGS: u32 = 11;
    pub const SHELL_STREAM_ARGS: u32 = 14;
    pub const EXEC_ID: u32 = 15;
    pub const BACKGROUND_SHELL_SPAWN_ARGS: u32 = 16;
    pub const LIST_MCP_RESOURCES_EXEC_ARGS: u32 = 17;
    pub const READ_MCP_RESOURCE_EXEC_ARGS: u32 = 18;
    pub const SPAN_CONTEXT: u32 = 19;
    pub const FETCH_ARGS: u32 = 20;
    pub const RECORD_SCREEN_ARGS: u32 = 21;
    pub const COMPUTER_USE_ARGS: u32 = 22;
    pub const WRITE_SHELL_STDIN_ARGS: u32 = 23;
}

/// Field tags inside `KvServerMessage` / `KvClientMessage` (proto indexes
/// 271 / 272).
pub mod kv_message {
    pub const ID: u32 = 1;
    pub const GET_BLOB: u32 = 2;
    pub const SET_BLOB: u32 = 3;
    pub const SPAN_CONTEXT: u32 = 4;
}

/// Field tags inside `InteractionUpdate` (proto index 109).
pub mod interaction_update {
    pub const TEXT_DELTA: u32 = 1;
    pub const THINKING_DELTA: u32 = 4;
    pub const TOKEN_DELTA: u32 = 8;
    pub const HEARTBEAT: u32 = 13;
    pub const TURN_ENDED: u32 = 14;
}

// ---------------------------------------------------------------------------
// GetUsableModels (unary)
// ---------------------------------------------------------------------------

/// Field tags inside `GetUsableModelsResponse` (proto index 123).
pub mod get_usable_models_response {
    pub const MODELS: u32 = 1;
}

// ---------------------------------------------------------------------------
// High-level encoders / decoders
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Message {
    pub role: String,
    pub text: String,
}

pub struct AgentRunRequestInput<'a> {
    pub model: &'a str,
    pub requested_model: Option<RequestedModelInput<'a>>,
    pub messages: &'a [Message],
    pub message_id: &'a str,
    pub conversation_id: Option<&'a str>,
    pub os_version: &'a str,
    pub workspace_path: &'a str,
    pub shell: &'a str,
    pub tools: &'a [crate::cursor_agent::CursorTool],
}

#[derive(Debug, Clone, Copy)]
pub struct RequestedModelParameter<'a> {
    pub id: &'a str,
    pub value: &'a str,
}

#[derive(Debug, Clone, Copy)]
pub struct RequestedModelInput<'a> {
    pub model_id: &'a str,
    pub max_mode: bool,
    pub parameters: &'a [RequestedModelParameter<'a>],
}

/// Heartbeat encoder. Returns the framed `AgentClientMessage` body bytes
/// (no Connect envelope).
pub fn encode_client_heartbeat() -> Vec<u8> {
    encode_message_field(agent_client_message::CLIENT_HEARTBEAT, &[])
}

/// Encode the minimal `AgentRunRequest` for first delivery.
pub fn encode_agent_run_request(input: AgentRunRequestInput<'_>) -> Vec<u8> {
    let prompt = messages_to_prompt(input.messages);
    let user_message_bytes = concat_bytes(&[
        encode_string_field(user_message::TEXT, &prompt),
        encode_string_field(user_message::MESSAGE_ID, input.message_id),
        encode_int32_field(user_message::MODE, 1),
    ]);

    let env_body = concat_bytes(&[
        encode_string_field(request_context_env::OS_VERSION, input.os_version),
        encode_string_field(request_context_env::WORKSPACE_PATHS, input.workspace_path),
        encode_string_field(request_context_env::SHELL, input.shell),
        encode_string_field(request_context_env::PROJECT_FOLDER, input.workspace_path),
    ]);
    let request_context = encode_message_field(request_context::ENV, &env_body);

    let user_action = concat_bytes(&[
        encode_message_field(user_message_action::USER_MESSAGE, &user_message_bytes),
        encode_message_field(user_message_action::REQUEST_CONTEXT, &request_context),
    ]);
    let conversation_action_bytes =
        encode_message_field(conversation_action::USER_MESSAGE_ACTION, &user_action);
    let model_details_bytes = encode_string_field(model_details::MODEL_ID, input.model);
    let mcp_tools = encode_mcp_tools(input.tools);

    let mut run_parts = vec![
        encode_message_field(agent_run_request::CONVERSATION_STATE, &[]),
        encode_message_field(agent_run_request::ACTION, &conversation_action_bytes),
        encode_message_field(agent_run_request::MODEL_DETAILS, &model_details_bytes),
    ];
    if !mcp_tools.is_empty() {
        run_parts.push(encode_message_field(
            agent_run_request::MCP_TOOLS,
            &mcp_tools,
        ));
    }
    if let Some(id) = input.conversation_id {
        run_parts.push(encode_string_field(agent_run_request::CONVERSATION_ID, id));
    }
    if let Some(requested_model) = input.requested_model {
        run_parts.push(encode_message_field(
            agent_run_request::REQUESTED_MODEL,
            &encode_requested_model(requested_model),
        ));
    }
    let run_request_bytes = concat_bytes(&run_parts);
    encode_message_field(agent_client_message::RUN_REQUEST, &run_request_bytes)
}

fn encode_requested_model(input: RequestedModelInput<'_>) -> Vec<u8> {
    let parameter_messages = input
        .parameters
        .iter()
        .map(|parameter| {
            concat_bytes(&[
                encode_string_field(requested_model_parameter::ID, parameter.id),
                encode_string_field(requested_model_parameter::VALUE, parameter.value),
            ])
        })
        .collect::<Vec<_>>();

    let mut parts = vec![
        encode_string_field(requested_model::MODEL_ID, input.model_id),
        encode_bool_field(requested_model::MAX_MODE, input.max_mode),
    ];
    parts.extend(
        parameter_messages
            .iter()
            .map(|message| encode_message_field(requested_model::PARAMETERS, message)),
    );
    concat_bytes(&parts)
}

fn messages_to_prompt(messages: &[Message]) -> String {
    let mut out = String::new();
    for message in messages {
        out.push_str(&capitalize(&message.role));
        out.push_str(": ");
        out.push_str(&message.text);
        out.push('\n');
    }
    out
}

fn capitalize(input: &str) -> String {
    let mut chars = input.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().chain(chars).collect(),
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum InteractionEvent {
    Text(String),
    Thinking(String),
    TokenDelta(i32),
    Heartbeat,
    TurnEnded,
}

#[derive(Debug, Clone)]
pub struct ExecRequest {
    pub id: u64,
    pub exec_id: String,
    pub kind: ExecKind,
    pub args: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecKind {
    Shell,
    Write,
    Delete,
    Grep,
    Read,
    Ls,
    Diagnostics,
    RequestContext,
    Mcp,
    ShellStream,
    BackgroundShellSpawn,
    ListMcpResources,
    ReadMcpResource,
    Fetch,
    RecordScreen,
    ComputerUse,
    WriteShellStdin,
    Other(u32),
}

impl ExecKind {
    pub fn from_field(field: u32) -> Self {
        match field {
            exec_message::SHELL_ARGS => ExecKind::Shell,
            exec_message::WRITE_ARGS => ExecKind::Write,
            exec_message::DELETE_ARGS => ExecKind::Delete,
            exec_message::GREP_ARGS => ExecKind::Grep,
            exec_message::READ_ARGS => ExecKind::Read,
            exec_message::LS_ARGS => ExecKind::Ls,
            exec_message::DIAGNOSTICS_ARGS => ExecKind::Diagnostics,
            exec_message::REQUEST_CONTEXT_ARGS => ExecKind::RequestContext,
            exec_message::MCP_ARGS => ExecKind::Mcp,
            exec_message::SHELL_STREAM_ARGS => ExecKind::ShellStream,
            exec_message::BACKGROUND_SHELL_SPAWN_ARGS => ExecKind::BackgroundShellSpawn,
            exec_message::LIST_MCP_RESOURCES_EXEC_ARGS => ExecKind::ListMcpResources,
            exec_message::READ_MCP_RESOURCE_EXEC_ARGS => ExecKind::ReadMcpResource,
            exec_message::FETCH_ARGS => ExecKind::Fetch,
            exec_message::RECORD_SCREEN_ARGS => ExecKind::RecordScreen,
            exec_message::COMPUTER_USE_ARGS => ExecKind::ComputerUse,
            exec_message::WRITE_SHELL_STDIN_ARGS => ExecKind::WriteShellStdin,
            other => ExecKind::Other(other),
        }
    }
}

#[derive(Debug, Clone)]
pub struct KvRequest {
    pub id: u64,
    pub kind: KvKind,
    pub args: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KvKind {
    GetBlob,
    SetBlob,
    Other(u32),
}

impl KvKind {
    pub fn from_field(field: u32) -> Self {
        match field {
            kv_message::GET_BLOB => KvKind::GetBlob,
            kv_message::SET_BLOB => KvKind::SetBlob,
            other => KvKind::Other(other),
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct AgentServerMessage {
    pub events: Vec<InteractionEvent>,
    pub exec_requests: Vec<ExecRequest>,
    pub kv_requests: Vec<KvRequest>,
    pub saw_checkpoint: bool,
    pub checkpoint_update: Option<Vec<u8>>,
}

pub fn decode_agent_server_message(bytes: &[u8]) -> AgentServerMessage {
    let mut out = AgentServerMessage::default();
    for field in parse_proto_fields(bytes) {
        match field.number {
            v if v == agent_server_message::INTERACTION_UPDATE => {
                out.events.extend(parse_interaction_update(&field.value));
            }
            v if v == agent_server_message::EXEC_SERVER_MESSAGE => {
                if let Some(req) = parse_exec_server_message(&field.value) {
                    out.exec_requests.push(req);
                }
            }
            v if v == agent_server_message::CONVERSATION_CHECKPOINT_UPDATE => {
                out.saw_checkpoint = true;
                out.checkpoint_update = Some(field.value);
            }
            v if v == agent_server_message::KV_SERVER_MESSAGE => {
                if let Some(req) = parse_kv_server_message(&field.value) {
                    out.kv_requests.push(req);
                }
            }
            _ => {}
        }
    }
    out
}

fn parse_interaction_update(data: &[u8]) -> Vec<InteractionEvent> {
    let mut events = Vec::new();
    for field in parse_proto_fields(data) {
        match field.number {
            v if v == interaction_update::TEXT_DELTA && field.wire_type == 2 => {
                if let Some(text) = take_string_subfield(&field.value, 1) {
                    events.push(InteractionEvent::Text(text));
                }
            }
            v if v == interaction_update::THINKING_DELTA && field.wire_type == 2 => {
                if let Some(text) = take_string_subfield(&field.value, 1) {
                    events.push(InteractionEvent::Thinking(text));
                }
            }
            v if v == interaction_update::TOKEN_DELTA && field.wire_type == 2 => {
                if let Some(tokens) = take_int_subfield(&field.value, 1) {
                    events.push(InteractionEvent::TokenDelta(tokens));
                }
            }
            v if v == interaction_update::HEARTBEAT => {
                events.push(InteractionEvent::Heartbeat);
            }
            v if v == interaction_update::TURN_ENDED => {
                events.push(InteractionEvent::TurnEnded);
            }
            _ => {}
        }
    }
    events
}

fn parse_exec_server_message(data: &[u8]) -> Option<ExecRequest> {
    let mut id = 0u64;
    let mut exec_id = String::new();
    let mut kind: Option<(u32, Vec<u8>)> = None;
    for field in parse_proto_fields(data) {
        match field.number {
            v if v == exec_message::ID => {
                id = decode_varint(&field.value, 0)
                    .map(|(value, _)| value)
                    .unwrap_or(0);
            }
            v if v == exec_message::EXEC_ID => {
                exec_id = String::from_utf8_lossy(&field.value).into_owned();
            }
            v if v == exec_message::SPAN_CONTEXT => {}
            other if field.wire_type == 2 => {
                kind = Some((other, field.value));
            }
            _ => {}
        }
    }
    let (field, args) = kind?;
    Some(ExecRequest {
        id,
        exec_id,
        kind: ExecKind::from_field(field),
        args,
    })
}

fn parse_kv_server_message(data: &[u8]) -> Option<KvRequest> {
    let mut id = 0u64;
    let mut kind: Option<(u32, Vec<u8>)> = None;
    for field in parse_proto_fields(data) {
        match field.number {
            v if v == kv_message::ID => {
                id = decode_varint(&field.value, 0)
                    .map(|(value, _)| value)
                    .unwrap_or(0);
            }
            v if (v == kv_message::GET_BLOB || v == kv_message::SET_BLOB)
                && field.wire_type == 2 =>
            {
                kind = Some((v, field.value));
            }
            v if v == kv_message::SPAN_CONTEXT => {}
            _ => {}
        }
    }
    let (field, args) = kind?;
    Some(KvRequest {
        id,
        kind: KvKind::from_field(field),
        args,
    })
}

pub fn decode_get_blob_args(data: &[u8]) -> Option<Vec<u8>> {
    parse_proto_fields(data)
        .into_iter()
        .find(|field| field.number == 1 && field.wire_type == 2)
        .map(|field| field.value)
}

pub fn decode_set_blob_args(data: &[u8]) -> Option<(Vec<u8>, Vec<u8>)> {
    let mut blob_id = None;
    let mut blob_data = None;
    for field in parse_proto_fields(data) {
        match field.number {
            1 if field.wire_type == 2 => blob_id = Some(field.value),
            2 if field.wire_type == 2 => blob_data = Some(field.value),
            _ => {}
        }
    }
    Some((blob_id?, blob_data.unwrap_or_default()))
}

pub fn encode_get_blob_result(id: u64, blob_data: Option<&[u8]>) -> Vec<u8> {
    let result = blob_data
        .map(|data| encode_message_field(1, data))
        .unwrap_or_default();
    encode_kv_client_message(id, kv_message::GET_BLOB, &result)
}

pub fn encode_set_blob_result(id: u64) -> Vec<u8> {
    encode_kv_client_message(id, kv_message::SET_BLOB, &[])
}

fn encode_kv_client_message(id: u64, result_field: u32, result_body: &[u8]) -> Vec<u8> {
    let kv_client = concat_bytes(&[
        encode_int64_field(kv_message::ID, id),
        encode_message_field(result_field, result_body),
    ]);
    encode_message_field(agent_client_message::KV_CLIENT_MESSAGE, &kv_client)
}

pub fn encode_request_context_result(
    exec: &ExecRequest,
    tools: &[crate::cursor_agent::CursorTool],
    os_version: &str,
    workspace_path: &str,
    shell: &str,
) -> Vec<u8> {
    let env = concat_bytes(&[
        encode_string_field(1, os_version),
        encode_repeated_string_field(2, &[workspace_path.to_string()]),
        encode_string_field(3, shell),
        encode_string_field(11, workspace_path),
    ]);
    let tool_defs = encode_mcp_tool_definitions(tools);
    let request_context = concat_bytes(&[
        encode_message_field(request_context::ENV, &env),
        encode_repeated_message_field(request_context::TOOLS, &tool_defs),
        encode_message_field(
            request_context::MCP_INSTRUCTIONS,
            &concat_bytes(&[
                encode_string_field(mcp_instructions::SERVER_NAME, "opencode"),
                encode_string_field(
                    mcp_instructions::INSTRUCTIONS,
                    "Use the MCP tools listed in this request context. For codebase index searches, call cursor_codebase_search when it is listed; do not substitute grep, read, or shell for index-search requests.",
                ),
            ]),
        ),
    ]);
    let success = encode_message_field(1, &request_context);
    let result = encode_message_field(1, &success);
    encode_exec_result(exec, exec_message::REQUEST_CONTEXT_ARGS, result)
}

fn encode_mcp_tools(tools: &[crate::cursor_agent::CursorTool]) -> Vec<u8> {
    let tool_defs = encode_mcp_tool_definitions(tools);
    if tool_defs.is_empty() {
        Vec::new()
    } else {
        encode_repeated_message_field(1, &tool_defs)
    }
}

fn encode_mcp_tool_definitions(tools: &[crate::cursor_agent::CursorTool]) -> Vec<Vec<u8>> {
    tools
        .iter()
        .map(|tool| {
            let schema = encode_protobuf_value(&tool.parameters_schema);
            concat_bytes(&[
                encode_string_field(mcp_tool_definition::NAME, &tool.name),
                encode_string_field(
                    mcp_tool_definition::DESCRIPTION,
                    tool.description.as_deref().unwrap_or(""),
                ),
                encode_message_field(mcp_tool_definition::INPUT_SCHEMA, &schema),
                encode_string_field(mcp_tool_definition::PROVIDER_IDENTIFIER, "opencode"),
                encode_string_field(mcp_tool_definition::TOOL_NAME, &tool.name),
            ])
        })
        .collect()
}

fn encode_protobuf_value(value: &serde_json::Value) -> Vec<u8> {
    match value {
        serde_json::Value::Null => encode_varint_field_always(protobuf_value::NULL_VALUE, 0),
        serde_json::Value::Bool(value) => {
            encode_bool_field_always(protobuf_value::BOOL_VALUE, *value)
        }
        serde_json::Value::Number(value) => value
            .as_f64()
            .map(|number| encode_double_field_always(protobuf_value::NUMBER_VALUE, number))
            .unwrap_or_default(),
        serde_json::Value::String(value) => {
            encode_string_field_always(protobuf_value::STRING_VALUE, value)
        }
        serde_json::Value::Array(values) => {
            let encoded_values: Vec<Vec<u8>> = values.iter().map(encode_protobuf_value).collect();
            let list = encode_repeated_message_field(protobuf_list_value::VALUES, &encoded_values);
            encode_message_field(protobuf_value::LIST_VALUE, &list)
        }
        serde_json::Value::Object(map) => {
            let fields: Vec<Vec<u8>> = map
                .iter()
                .map(|(key, value)| {
                    concat_bytes(&[
                        encode_string_field(protobuf_struct_field_entry::KEY, key),
                        encode_message_field(
                            protobuf_struct_field_entry::VALUE,
                            &encode_protobuf_value(value),
                        ),
                    ])
                })
                .collect();
            let struct_value = encode_repeated_message_field(protobuf_struct::FIELDS, &fields);
            encode_message_field(protobuf_value::STRUCT_VALUE, &struct_value)
        }
    }
}

pub fn decode_mcp_args(data: &[u8]) -> (String, String, String) {
    let name = decode_string_field(data, 5)
        .or_else(|| decode_string_field(data, 1))
        .unwrap_or_default();
    let tool_call_id = decode_string_field(data, 3).unwrap_or_default();
    let arguments = decode_mcp_args_map(data);
    (name, tool_call_id, arguments)
}

pub fn decode_exec_public_tool_call(exec: &ExecRequest) -> (String, String, serde_json::Value) {
    if matches!(exec.kind, ExecKind::Mcp) {
        let (name, tool_call_id, arguments) = decode_mcp_args(&exec.args);
        let value = serde_json::from_str(&arguments).unwrap_or_else(|_| serde_json::json!({}));
        return (name, tool_call_id, value);
    }

    let name = exec_public_tool_name(exec.kind).to_string();
    let args = match exec.kind {
        ExecKind::Shell | ExecKind::ShellStream | ExecKind::BackgroundShellSpawn => {
            serde_json::json!({
                "command": decode_string_field(&exec.args, 1).unwrap_or_default(),
                "working_directory": decode_string_field(&exec.args, 2).unwrap_or_default(),
            })
        }
        ExecKind::Write | ExecKind::Delete | ExecKind::Read | ExecKind::Ls => {
            serde_json::json!({
                "path": decode_string_field(&exec.args, 1).unwrap_or_default(),
            })
        }
        ExecKind::Grep => serde_json::json!({
            "pattern": decode_string_field(&exec.args, 1).unwrap_or_default(),
            "path": decode_string_field(&exec.args, 2).unwrap_or_default(),
            "output_mode": decode_string_field(&exec.args, 3).unwrap_or_default(),
        }),
        ExecKind::Fetch => serde_json::json!({
            "url": decode_string_field(&exec.args, 1).unwrap_or_default(),
        }),
        ExecKind::WriteShellStdin => serde_json::json!({
            "shell_id": decode_u64_field(&exec.args, 1).unwrap_or_default(),
            "input": decode_string_field(&exec.args, 2).unwrap_or_default(),
        }),
        ExecKind::Diagnostics
        | ExecKind::ListMcpResources
        | ExecKind::ReadMcpResource
        | ExecKind::RecordScreen
        | ExecKind::ComputerUse
        | ExecKind::Other(_) => decode_unknown_exec_args(&exec.args),
        ExecKind::RequestContext | ExecKind::Mcp => serde_json::json!({}),
    };
    (name, exec.exec_id.clone(), args)
}

fn exec_public_tool_name(kind: ExecKind) -> &'static str {
    match kind {
        ExecKind::Shell => "shell",
        ExecKind::Write => "write",
        ExecKind::Delete => "delete",
        ExecKind::Grep => "grep",
        ExecKind::Read => "read",
        ExecKind::Ls => "ls",
        ExecKind::Diagnostics => "diagnostics",
        ExecKind::RequestContext => "request_context",
        ExecKind::Mcp => "mcp",
        ExecKind::ShellStream => "shell_stream",
        ExecKind::BackgroundShellSpawn => "background_shell_spawn",
        ExecKind::ListMcpResources => "list_mcp_resources",
        ExecKind::ReadMcpResource => "read_mcp_resource",
        ExecKind::Fetch => "fetch",
        ExecKind::RecordScreen => "record_screen",
        ExecKind::ComputerUse => "computer_use",
        ExecKind::WriteShellStdin => "write_shell_stdin",
        ExecKind::Other(_) => "cursor_exec",
    }
}

fn decode_unknown_exec_args(data: &[u8]) -> serde_json::Value {
    let mut object = serde_json::Map::new();
    for field in parse_proto_fields(data) {
        let key = format!("field_{}", field.number);
        let value = match field.wire_type {
            0 => decode_varint(&field.value, 0)
                .map(|(value, _)| serde_json::json!(value))
                .unwrap_or(serde_json::Value::Null),
            2 => String::from_utf8(field.value.clone())
                .map(serde_json::Value::String)
                .unwrap_or_else(|_| {
                    serde_json::json!(base64::Engine::encode(
                        &base64::engine::general_purpose::STANDARD,
                        field.value
                    ))
                }),
            _ => serde_json::json!(base64::Engine::encode(
                &base64::engine::general_purpose::STANDARD,
                field.value
            )),
        };
        object.insert(key, value);
    }
    serde_json::Value::Object(object)
}

fn decode_mcp_args_map(data: &[u8]) -> String {
    let mut object = serde_json::Map::new();
    for field in parse_proto_fields(data) {
        if field.number != 2 || field.wire_type != 2 {
            continue;
        }
        let key = decode_string_field(&field.value, 1).unwrap_or_default();
        let value = parse_proto_fields(&field.value)
            .into_iter()
            .find(|inner| inner.number == 2 && inner.wire_type == 2)
            .map(|inner| inner.value)
            .unwrap_or_default();
        if key.is_empty() {
            continue;
        }
        let decoded = serde_json::from_slice(&value).unwrap_or_else(|_| {
            serde_json::Value::String(String::from_utf8_lossy(&value).into_owned())
        });
        object.insert(key, decoded);
    }
    serde_json::Value::Object(object).to_string()
}

fn encode_exec_result(exec: &ExecRequest, result_field: u32, result_body: Vec<u8>) -> Vec<u8> {
    let exec_client = concat_bytes(&[
        encode_int64_field(exec_message::ID, exec.id),
        encode_string_field(exec_message::EXEC_ID, &exec.exec_id),
        encode_message_field(result_field, &result_body),
    ]);
    encode_message_field(agent_client_message::EXEC_CLIENT_MESSAGE, &exec_client)
}

fn decode_string_field(data: &[u8], field_number: u32) -> Option<String> {
    parse_proto_fields(data)
        .into_iter()
        .find(|field| field.number == field_number && field.wire_type == 2)
        .map(|field| String::from_utf8_lossy(&field.value).into_owned())
}

fn decode_u64_field(data: &[u8], field_number: u32) -> Option<u64> {
    let field = parse_proto_fields(data)
        .into_iter()
        .find(|field| field.number == field_number && field.wire_type == 0)?;
    decode_varint(&field.value, 0).map(|(value, _)| value)
}

fn take_string_subfield(data: &[u8], number: u32) -> Option<String> {
    parse_proto_fields(data)
        .into_iter()
        .find(|field| field.number == number && field.wire_type == 2)
        .map(|field| String::from_utf8_lossy(&field.value).into_owned())
}

fn take_int_subfield(data: &[u8], number: u32) -> Option<i32> {
    let field = parse_proto_fields(data)
        .into_iter()
        .find(|field| field.number == number && field.wire_type == 0)?;
    let (value, _) = decode_varint(&field.value, 0)?;
    Some(value as i32)
}

// ---------------------------------------------------------------------------
// GetUsableModels encode / decode
// ---------------------------------------------------------------------------

pub fn encode_get_usable_models_request() -> Vec<u8> {
    Vec::new()
}

#[derive(Debug, Clone)]
pub struct ModelDescriptorRaw {
    pub model_id: String,
    pub display_name: Option<String>,
    pub display_name_short: Option<String>,
    pub display_model_id: Option<String>,
    pub aliases: Vec<String>,
    pub supports_reasoning: bool,
}

pub fn decode_get_usable_models_response(bytes: &[u8]) -> Vec<ModelDescriptorRaw> {
    let mut out = Vec::new();
    for field in parse_proto_fields(bytes) {
        if field.number == get_usable_models_response::MODELS && field.wire_type == 2 {
            out.push(decode_model_details(&field.value));
        }
    }
    out
}

fn decode_model_details(bytes: &[u8]) -> ModelDescriptorRaw {
    let mut model_id = String::new();
    let mut display_name = None;
    let mut display_name_short = None;
    let mut display_model_id = None;
    let mut aliases = Vec::new();
    let mut supports_reasoning = false;

    for field in parse_proto_fields(bytes) {
        match field.number {
            v if v == model_details::MODEL_ID && field.wire_type == 2 => {
                model_id = String::from_utf8_lossy(&field.value).into_owned();
            }
            v if v == model_details::THINKING_DETAILS => {
                supports_reasoning = true;
            }
            v if v == model_details::DISPLAY_MODEL_ID && field.wire_type == 2 => {
                display_model_id = Some(String::from_utf8_lossy(&field.value).into_owned());
            }
            v if v == model_details::DISPLAY_NAME && field.wire_type == 2 => {
                display_name = Some(String::from_utf8_lossy(&field.value).into_owned());
            }
            v if v == model_details::DISPLAY_NAME_SHORT && field.wire_type == 2 => {
                display_name_short = Some(String::from_utf8_lossy(&field.value).into_owned());
            }
            v if v == model_details::ALIASES && field.wire_type == 2 => {
                aliases.push(String::from_utf8_lossy(&field.value).into_owned());
            }
            _ => {}
        }
    }

    ModelDescriptorRaw {
        model_id,
        display_name,
        display_name_short,
        display_model_id,
        aliases,
        supports_reasoning,
    }
}
