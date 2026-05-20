# Codex Tool Catalog

This document defines the tools used by the Codex agent, including built-in primitives and extended agentic capabilities.

## Built-in Handlers

### Shell (`shell`)
- **Description**: Runs a command in a PTY, returning output or a session ID.
- **Parameters**:
  - `command`: The shell command to execute.
  - `cwd`: (optional) Working directory.
  - `shell`: (optional) Shell binary (defaults to user default).
  - `tty`: (bool) Allocate a TTY.
  - `login`: (bool) Run as login shell.

### Apply Patch (`apply_patch`)
- **Description**: Applies a git-compatible patch or a semantic diff to the local working tree.

### Spawn Agent (`spawn_agent`)
- **Description**: Spawns a sub-agent with a forked history or fresh context.
- **Parameters**:
  - `message`: (optional) Initial task.
  - `items`: (optional) Structured input items (mentions, images).
  - `fork`: (bool) Fork the current thread history.
  - `model`: (optional) Model override.

## UMP Cursor Profile Names

The Cursor profile currently preserves existing compatibility output shapes where they differ from the local Codex catalog:

- Native/compat names filtered from Cursor MCP ads include `shell`, `apply_patch`, `spawn_agent`, `get_goal`, `create_goal`, `update_goal`, `read_file`, `edit_file`, `ls`, `grep`, `glob`, `web_search`, `shell_command`, `exec_command`, `write_stdin`, `list_mcp_resources`, `read_mcp_resource`, `list_mcp_resource_templates`, `update_plan`, `send_input`, `resume_agent`, `wait_agent`, `close_agent`, and `view_image`.
- External MCP names should be namespaced as `mcp__server__tool`.
- `opencode` MCP calls with raw Codex-native names are refused as native-tool leaks; `opencode` calls with non-native names pass through unchanged.

---

## State & Goal Management

### Create Goal (`create_goal`)
- **Description**: Starts a new active objective for the current thread.
- **Parameters**:
  - `objective`: Concrete goal description.
  - `budget`: (optional) Token budget for the goal.

### Update Goal (`update_goal`)
- **Description**: Marks a goal as complete, blocked, or updates progress.

---

## Workspace Tools

### File Operations
- `read_file`: Read content from a file path.
- `edit_file`: Apply edits to a file.
- `ls`: List directory contents.
- `grep`: Search for strings using ripgrep.
- `glob`: Search for files by pattern.

### User Interaction
- `request_user_input`: Present a questionnaire to the user.
- `request_plugin_install`: Suggest a plugin for installation.
- `request_permissions`: Escalation for filesystem/network access.

---

## MCP & Plugins
Codex supports any MCP server configured via `codex mcp add`. Built-in MCP groups often include:
- `omx_code_intel`: Semantic code navigation.
- `omx_state`: Persistence and mode management.
- `omx_trace`: Debugging and causal tracing.
