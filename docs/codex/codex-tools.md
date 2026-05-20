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
