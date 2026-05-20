//! Client-native tool registries for Cursor MCP advertisement filtering.
//!
//! Cursor's native client tools and MCP tools are separate lanes. The current
//! AgentService path only carries MCP advertisements, so raw client-native
//! names must be filtered out before they are sent to Cursor as `mcp_tools`.

use crate::{cursor_agent::CursorTool, upstream::cursor::client_profile::ClientProfile};

const CURSOR_CODEBASE_SEARCH: &str = "cursor_codebase_search";

const DROID_NATIVE_TOOLS: &[&str] = &[
    "Read",
    "LS",
    "Grep",
    "Glob",
    "Create",
    "Edit",
    "ApplyPatch",
    "Execute",
    "FetchUrl",
    "WebSearch",
    "Task",
    "TodoWrite",
    "DismissHandoffItems",
    "EndFeatureRun",
    "ExitSpecMode",
    "GenerateDroid",
    "Skill",
    "ToolSearch",
    "ProposeMission",
    "StartMissionRun",
    "MultiEdit",
];

const CLAUDE_NATIVE_TOOLS: &[&str] = &[
    "Agent",
    "AskUserQuestion",
    "Bash",
    "BashOutput",
    "KillBash",
    "Edit",
    "MultiEdit",
    "Write",
    "Glob",
    "Grep",
    "LS",
    "Read",
    "ListMcpResourcesTool",
    "ReadMcpResourceTool",
    "LSP",
    "Monitor",
    "NotebookEdit",
    "NotebookRead",
    "PowerShell",
    "TodoWrite",
    "WebFetch",
    "WebSearch",
    "Task",
    "TaskCreate",
    "TaskGet",
    "TaskList",
    "TaskUpdate",
    "TaskStop",
    "TaskOutput",
    "Skill",
    "ToolSearch",
    "WaitForMcpServers",
    "ExitPlanMode",
    "EnterPlanMode",
    "EnterWorktree",
    "ExitWorktree",
    "CronCreate",
    "CronDelete",
    "CronList",
    "SendMessage",
    "TeamCreate",
    "TeamDelete",
    "PushNotification",
    "RemoteTrigger",
    "ShareOnboardingGuide",
];

const CODEX_NATIVE_TOOLS: &[&str] = &[
    "shell",
    "apply_patch",
    "spawn_agent",
    "get_goal",
    "create_goal",
    "update_goal",
    "read_file",
    "edit_file",
    "ls",
    "grep",
    "glob",
    "request_user_input",
    "request_plugin_install",
    "request_permissions",
    "web_search",
    "shell_command",
    "exec_command",
    "write_stdin",
    "read_mcp_resource",
    "list_mcp_resources",
    "list_mcp_resource_templates",
    "update_plan",
    "send_input",
    "resume_agent",
    "wait_agent",
    "close_agent",
    "view_image",
];

const DEVIN_NATIVE_TOOLS: &[&str] = &[
    "read", "edit", "delete", "move", "search", "execute", "think", "fetch",
];

pub fn is_client_native_tool(profile: ClientProfile, name: &str) -> bool {
    match profile {
        ClientProfile::Droid => DROID_NATIVE_TOOLS.contains(&name),
        ClientProfile::ClaudeCode => CLAUDE_NATIVE_TOOLS.contains(&name),
        ClientProfile::CodexCli => CODEX_NATIVE_TOOLS.contains(&name),
        ClientProfile::Devin => DEVIN_NATIVE_TOOLS.contains(&name),
        ClientProfile::GenericAnthropic | ClientProfile::GenericOpenAi => false,
    }
}

pub fn is_cursor_codebase_search(name: &str) -> bool {
    name == CURSOR_CODEBASE_SEARCH
}

pub fn is_already_namespaced_external_tool(name: &str) -> bool {
    if let Some(rest) = name.strip_prefix("mcp__") {
        return rest
            .split_once("__")
            .is_some_and(|(server, tool)| !server.is_empty() && !tool.is_empty());
    }

    name.split_once("___")
        .is_some_and(|(server, tool)| !server.is_empty() && !tool.is_empty())
}

pub fn tools_visible_to_cursor(profile: ClientProfile, tools: &[CursorTool]) -> Vec<CursorTool> {
    tools
        .iter()
        .filter(|tool| {
            is_cursor_codebase_search(&tool.name) || !is_client_native_tool(profile, &tool.name)
        })
        .cloned()
        .collect()
}

pub fn profile_mcp_tool_name(profile: ClientProfile, server: &str, raw_tool_name: &str) -> String {
    if server.is_empty() || server == "opencode" {
        return raw_tool_name.to_string();
    }

    match profile {
        ClientProfile::Droid => format!("{server}___{raw_tool_name}"),
        ClientProfile::CodexCli | ClientProfile::ClaudeCode | ClientProfile::Devin => {
            format!("mcp__{server}__{raw_tool_name}")
        }
        ClientProfile::GenericAnthropic | ClientProfile::GenericOpenAi => raw_tool_name.to_string(),
    }
}

pub fn is_synthetic_mcp_native_leak(
    profile: ClientProfile,
    server: &str,
    raw_tool_name: &str,
) -> bool {
    (server.is_empty() || server == "opencode") && is_client_native_tool(profile, raw_tool_name)
}
