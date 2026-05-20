use unified_model_proxy_v2::cursor_agent::{CursorTool, CursorToolKind};
use unified_model_proxy_v2::upstream::cursor::{
    client_profile::ClientProfile,
    profiles::native_tools::{is_already_namespaced_external_tool, tools_visible_to_cursor},
    proto::{
        agent_client_message, agent_run_request, encode_agent_run_request,
        encode_request_context_result, parse_proto_fields, request_context, AgentRunRequestInput,
        ExecKind, ExecRequest, Message,
    },
};

fn tool(name: &str) -> CursorTool {
    CursorTool {
        name: name.to_string(),
        description: Some(format!("{name} tool")),
        parameters_schema: serde_json::json!({ "type": "object" }),
        kind: CursorToolKind::Function,
    }
}

fn visible_names(profile: ClientProfile, names: &[&str]) -> Vec<String> {
    let tools: Vec<CursorTool> = names.iter().map(|name| tool(name)).collect();
    tools_visible_to_cursor(profile, &tools)
        .into_iter()
        .map(|tool| tool.name)
        .collect()
}

fn agent_run_mcp_tool_names(tools: &[CursorTool]) -> Vec<String> {
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
        tools,
    });
    let outer = parse_proto_fields(&encoded);
    let run_payload = outer
        .iter()
        .find(|field| field.number == agent_client_message::RUN_REQUEST)
        .expect("run request")
        .value
        .clone();
    let run_fields = parse_proto_fields(&run_payload);
    let Some(mcp_tools) = run_fields
        .iter()
        .find(|field| field.number == agent_run_request::MCP_TOOLS)
        .map(|field| field.value.clone())
    else {
        return Vec::new();
    };

    parse_proto_fields(&mcp_tools)
        .into_iter()
        .filter(|field| field.number == 1)
        .filter_map(|entry| {
            parse_proto_fields(&entry.value)
                .into_iter()
                .find(|field| field.number == 1)
                .map(|field| String::from_utf8_lossy(&field.value).into_owned())
        })
        .collect()
}

fn request_context_tool_names(tools: &[CursorTool]) -> Vec<String> {
    let exec = ExecRequest {
        id: 5,
        exec_id: "exec-request-context".to_string(),
        kind: ExecKind::RequestContext,
        args: Vec::new(),
    };
    let encoded = encode_request_context_result(&exec, tools, "Darwin", "/tmp/workspace", "zsh");
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

    parse_proto_fields(&context)
        .into_iter()
        .filter(|field| field.number == request_context::TOOLS)
        .filter_map(|entry| {
            parse_proto_fields(&entry.value)
                .into_iter()
                .find(|field| field.number == 1)
                .map(|field| String::from_utf8_lossy(&field.value).into_owned())
        })
        .collect()
}

#[test]
fn droid_native_names_are_suppressed_from_cursor_ads() {
    let names = visible_names(
        ClientProfile::Droid,
        &[
            "Read",
            "TodoWrite",
            "ToolSearch",
            "DismissHandoffItems",
            "WebSearch",
        ],
    );
    assert!(names.is_empty());
}

#[test]
fn claude_native_names_are_suppressed_from_cursor_ads() {
    let names = visible_names(
        ClientProfile::ClaudeCode,
        &[
            "Read",
            "Bash",
            "TodoWrite",
            "WebSearch",
            "ListMcpResourcesTool",
        ],
    );
    assert!(names.is_empty());
}

#[test]
fn codex_native_names_are_suppressed_from_cursor_ads() {
    let names = visible_names(
        ClientProfile::CodexCli,
        &[
            "shell",
            "read_file",
            "exec_command",
            "get_goal",
            "web_search",
            "list_mcp_resource_templates",
        ],
    );
    assert!(names.is_empty());
}

#[test]
fn generic_profiles_do_not_suppress_tools() {
    assert_eq!(
        visible_names(ClientProfile::GenericOpenAi, &["Read"]),
        vec!["Read"]
    );
    assert_eq!(
        visible_names(ClientProfile::GenericAnthropic, &["Read"]),
        vec!["Read"]
    );
}

#[test]
fn cursor_codebase_search_is_preserved_for_all_profiles() {
    assert_eq!(
        visible_names(ClientProfile::Droid, &["Read", "cursor_codebase_search"]),
        vec!["cursor_codebase_search"]
    );
    assert_eq!(
        visible_names(
            ClientProfile::CodexCli,
            &["read_file", "cursor_codebase_search"],
        ),
        vec!["cursor_codebase_search"]
    );
    assert_eq!(
        visible_names(
            ClientProfile::ClaudeCode,
            &["Read", "cursor_codebase_search"],
        ),
        vec!["cursor_codebase_search"]
    );
    for profile in [
        ClientProfile::GenericOpenAi,
        ClientProfile::GenericAnthropic,
    ] {
        assert_eq!(
            visible_names(profile, &["Read", "cursor_codebase_search"]),
            vec!["Read", "cursor_codebase_search"]
        );
    }
}

#[test]
fn namespaced_external_tools_are_preserved_for_profile_ads() {
    assert!(is_already_namespaced_external_tool(
        "ref___ref_search_documentation"
    ));
    assert!(is_already_namespaced_external_tool("mcp__github__list_prs"));
    assert_eq!(
        visible_names(
            ClientProfile::Droid,
            &[
                "Read",
                "ref___ref_search_documentation",
                "exa___web_search_exa"
            ],
        ),
        vec!["ref___ref_search_documentation", "exa___web_search_exa"]
    );
    assert_eq!(
        visible_names(ClientProfile::CodexCli, &["shell", "mcp__github__list_prs"]),
        vec!["mcp__github__list_prs"]
    );
}

#[test]
fn raw_non_native_tools_are_preserved_for_profile_ads() {
    assert_eq!(
        visible_names(ClientProfile::Droid, &["Read", "lookup"]),
        vec!["lookup"]
    );
}

#[test]
fn agent_run_request_filters_native_tools_before_mcp_encoding() {
    let original_tools = vec![
        tool("Read"),
        tool("cursor_codebase_search"),
        tool("ref___ref_search_documentation"),
        tool("lookup"),
    ];
    let filtered = tools_visible_to_cursor(ClientProfile::Droid, &original_tools);

    assert_eq!(
        agent_run_mcp_tool_names(&filtered),
        vec![
            "cursor_codebase_search",
            "ref___ref_search_documentation",
            "lookup",
        ]
    );
}

#[test]
fn request_context_filters_native_tools_before_context_encoding() {
    let original_tools = vec![
        tool("Read"),
        tool("cursor_codebase_search"),
        tool("ref___ref_search_documentation"),
        tool("lookup"),
    ];
    let filtered = tools_visible_to_cursor(ClientProfile::Droid, &original_tools);

    assert_eq!(
        request_context_tool_names(&filtered),
        vec![
            "cursor_codebase_search",
            "ref___ref_search_documentation",
            "lookup",
        ]
    );
}

#[test]
fn cursor_visible_tool_filter_does_not_mutate_original_request_tools() {
    let original_tools = vec![tool("Read"), tool("lookup")];
    let filtered = tools_visible_to_cursor(ClientProfile::Droid, &original_tools);

    assert_eq!(
        filtered
            .into_iter()
            .map(|tool| tool.name)
            .collect::<Vec<_>>(),
        vec!["lookup"]
    );
    assert_eq!(
        original_tools
            .iter()
            .map(|tool| tool.name.as_str())
            .collect::<Vec<_>>(),
        vec!["Read", "lookup"]
    );
}
