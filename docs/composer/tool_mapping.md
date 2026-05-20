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

## UMP Mapping Policy

UMP keeps Cursor native tools and MCP tools separate:

- Client-native tool names are not advertised to Cursor as `mcp_tools` for Droid, Codex, or Claude profiles.
- External MCP tools must be distinguishable from native names. Prefer already-namespaced names: Droid `server___tool`; Codex/Claude `mcp__server__tool`.
- Raw external tool names that collide with a profile-native tool are suppressed at advertisement time because the normalized `CursorTool` has no server/source field.
- UMP-owned `cursor_codebase_search` remains visible to Cursor and is intercepted internally before public client rendering.
- If Cursor returns an MCP call with `server = "opencode"` and a raw native tool name, UMP refuses it as a native-tool MCP leak instead of mapping it back to a client-native tool.
- If Cursor returns `server = "opencode"` with a non-native raw tool name, UMP passes the raw tool name through unchanged to avoid double namespacing.

## Mapping Table

This table describes the desired native capability correspondence. The current
UMP AgentService renderer is narrower for some clients; see the shipped-profile
notes after the table before treating any row as implemented behavior.

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

## Current UMP Profile Renderers

The shipped AgentService profile renderers currently expose these concrete
tool-call names when Cursor emits native exec requests:

| **Capability** | Codex profile | Claude profile | Droid profile |
| :--- | :--- | :--- | :--- |
| **Read File** | `shell_command` with `cat` | `Read` with `file_path` | `Read` with `file_path` |
| **List Dir** | `shell_command` with `ls` | `Bash` with `ls` | `LS` |
| **Search Content** | `shell_command` with `rg` | `Grep` | `Grep` |
| **Run Command** | `shell_command` / `exec_command` | `Bash` | `Execute` |
| **Background Command** | refused | `Bash` with `run_in_background` | `Execute` with `fireAndForget` |
| **Edit/Create** | currently refused for Cursor `Write` | `Write` for Cursor `Write`; `Edit` for Cursor delete fallback | Droid `Create` currently refused for Cursor `Write`; delete maps to `Execute rm` |

Cursor MCP advertisements stay separate from these profile-native tool calls:
native names are filtered before `AgentRunRequest.mcp_tools` and
`RequestContext.tools`, while external MCP tools are preserved or namespaced as
described above.

## Detailed Mapping & Hardening Notes

### 1. Edit Operations & Transformation
- **Cursor**: `EditFileParams` provides suggested code and context.
- **Codex desired mapping**: use `edit_file` for precise semantic edits and `apply_patch` only for large, well-defined diffs. Current Cursor `Write` is refused until Cursor edit payloads carry enough replacement context.
- **Claude desired mapping**: transform Cursor edits into Claude-specific find/replace JSON blocks. Current profile maps Cursor `Write` to `Write` and delete fallback to `Edit`.
- **Droid desired mapping**: map Cursor edit explanations to Droid `justification` when edit support is enabled. Current profile refuses Cursor `Write` because it carries a path without file content.

### 2. Execution Environment & Interaction
- **PTY/TTY Support**: the desired Codex mapping is `shell` with `tty: true` for Cursor-like terminal behavior. Current shipped profile emits `shell_command` for simple foreground shell requests and `exec_command` for shell-stream requests.
- **Command Chaining**: Since `RUN_TERMINAL_COMMANDS` accepts a list of commands, the adapter should join them with `&&` or execute them sequentially.
- **Interactivity Gap**: Claude and Droid are primarily request-response. Interactive installers (e.g., `npm init`) may hang. Hardening: Use non-interactive flags (e.g., `-y`, `--yes`) where possible.
- **Undo Strategy**: Since harnesses lack a native `Undo` tool, map `UNDO` to shell-level Git operations: `git checkout -- <file>` or `git stash pop`.

### 3. Context & Intelligence
- **Linter Parity**: `ReadWithLinter` should be supplemented by enabling LSP plugins (Claude) or the `omx_code_intel` MCP (Codex) to ensure the agent doesn't lose visibility into diagnostics.
- **Search Dialects**: Note that Cursor/Codex use `ripgrep` regex patterns. Claude's built-in `Grep` might have slight variations; harden by using simple literal strings when possible.

### 4. Configuration & Runtime Flags
- **Codex**: `web_search` is only available if started with the `--search` flag. Note: Codex is **sandboxed** by default; system-level changes (e.g., `brew install`) will fail unless `--sandbox danger-full-access` is used.
- **Droid**: Tool availability depends on the `--auto` level. State-changing Cursor tools (`Edit`, `Create`) will fail in Droid's "read-only" (low autonomy) mode.
- **Profile filtering**: UMP filters profile-native names out of Cursor MCP advertisements before both the initial run request and request-context response.

### 5. Multi-Agent & State Persistence
- **State Inheritance**: When mapping `BackgroundComposer` to `spawn_agent` (Codex) or `Task` (Droid/Claude), ensure that environmental state (credentials, project paths) is explicitly passed or inherited. Codex's `spawn_agent` can use `fork: true` to replicate the current session state.
- **Interactive Stdin**: Cursor's `WriteStdin` is natively supported in Codex (`write_stdin`). For Claude and Droid, use non-interactive command flags (`-y`) to avoid blocking, as direct stdin injection is restricted.
