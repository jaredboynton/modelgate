# Tool Mapping: Native vs. Agent Harnesses

This document maps the native Cursor `aiserver.v1` (Composer) tools to their equivalents in external agent harnesses (`codex`, `claude`, `droid`).

## Protocol Integration (`aiserver.v1`)
In Cursor, tool calls are delivered via the `BuiltinToolCall` message in the `StreamUnifiedChatResponse`.

```protobuf
message BuiltinToolCall {
  oneof params {
    EditFileParams edit_file = 1;
    ReadFileParams read_file = 2;
    ListDirParams list_dir = 3;
    GrepParams grep = 4;
    RunTerminalCommandParams run_command = 5;
    // ...
  }
}
```

## Mapping Table

| **Capability** | Cursor (Native Proto) | Codex | Claude | Droid |
| :--- | :--- | :--- | :--- | :--- |
| **Edit File** | `EDIT` | `edit_file` (Primary) / `apply_patch` | `Edit` | `MultiEdit` (Morph) / `Edit` |
| **Read File** | `READ_CHUNK` | `read_file` | `Read` | `Read` |
| **List Dir** | `LIST_DIR` | `ls` | `LS` | `LS` |
| **Search Content** | `GREP` | `grep` | `Grep` | `Grep` |
| **Search Paths** | (Implicit) | `glob` | `Glob` | `Glob` |
| **Semantic Search** | `SEMANTIC_SEARCH` | `omx_code_intel` | `mcp__claude-context` | `morph-mcp_codebase_search` |
| **Run Command** | `RUN_TERMINAL_COMMANDS` | `shell` (set `tty: true`) | `Bash` | `Execute` |
| **Terminal Stdin** | `WRITE_STDIN` | `write_stdin` | (Manual Stdin) | (Manual Stdin) |
| **Stop Command** | `TERMINATE_TERMINAL` | `kill` (shell) | `kill` (shell) | `kill` (shell) |
| **Spawn Subagent** | `BACKGROUND_COMPOSER` | `spawn_agent` | `Task` | `Task` |
| **Web Search** | `WEB_SEARCH` | `web_search` (needs `--search`) | `exa___web_search` | `exa___web_search` |
| **Get Structure** | `GET_PROJECT_STRUCTURE`| `ls` (recursive) | `LS` | `LS` / `morph-mcp_codebase_search` |
| **Undo Change** | `UNDO` | `shell` (`git checkout`) | `Bash` (`git checkout`) | `Execute` (`git checkout`) |
| **Read w/ Linter** | `READ_WITH_LINTER` | `omx_code_intel` MCP | LSP Plugins | (Manual Grep/Linter) |

## Detailed Mapping & Hardening Notes

### 1. Edit Operations & Transformation
- **Cursor**: `EditFileParams` provides suggested code and context.
- **Codex**: Use `edit_file` (semantic find-and-replace) for precise changes; use `apply_patch` only for large, well-defined diffs to avoid offset drift.
- **Claude**: Requires transforming Cursor's suggested code into Claude-specific find/replace JSON blocks.
- **Droid**: Ensure the Cursor `explanation` is mapped to Droid's `justification` parameter, ensuring it meets the minimum character length requirements.

### 2. Execution Environment & Interaction
- **PTY/TTY Support**: `RUN_TERMINAL_COMMANDS` should map to `shell` in Codex with `tty: true` to ensure parity with Cursor's terminal expectations (e.g., color output, interactive prompts).
- **Command Chaining**: Since `RUN_TERMINAL_COMMANDS` accepts a list of commands, the adapter should join them with `&&` or execute them sequentially.
- **Interactivity Gap**: Claude and Droid are primarily request-response. Interactive installers (e.g., `npm init`) may hang. Hardening: Use non-interactive flags (e.g., `-y`, `--yes`) where possible.
- **Undo Strategy**: Since harnesses lack a native `Undo` tool, map `UNDO` to shell-level Git operations: `git checkout -- <file>` or `git stash pop`.

### 3. Context & Intelligence
- **Linter Parity**: `ReadWithLinter` should be supplemented by enabling LSP plugins (Claude) or the `omx_code_intel` MCP (Codex) to ensure the agent doesn't lose visibility into diagnostics.
- **Search Dialects**: Note that Cursor/Codex use `ripgrep` regex patterns. Claude's built-in `Grep` might have slight variations; harden by using simple literal strings when possible.

### 4. Configuration & Runtime Flags
- **Codex**: `web_search` is only available if started with the `--search` flag. Note: Codex is **sandboxed** by default; system-level changes (e.g., `brew install`) will fail unless `--sandbox danger-full-access` is used.
- **Droid**: Tool availability depends on the `--auto` level. State-changing Cursor tools (`Edit`, `Create`) will fail in Droid's "read-only" (low autonomy) mode.

### 5. Multi-Agent & State Persistence
- **State Inheritance**: When mapping `BackgroundComposer` to `spawn_agent` (Codex) or `Task` (Droid/Claude), ensure that environmental state (credentials, project paths) is explicitly passed or inherited. Codex's `spawn_agent` can use `fork: true` to replicate the current session state.
- **Interactive Stdin**: Cursor's `WriteStdin` is natively supported in Codex (`write_stdin`). For Claude and Droid, use non-interactive command flags (`-y`) to avoid blocking, as direct stdin injection is restricted.
