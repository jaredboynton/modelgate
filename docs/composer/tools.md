# Cursor Composer Native Tools Mapping

This document describes the native tools expected by Cursor Composer models (`composer-2.5`, `composer-2`, `composer-1.5`) and how they map to standard agent harness tools.

## Protocol Context

Tools in Cursor are defined in the `aiserver.v1` package. They are primarily transmitted via the `BuiltinToolCall` message.

UMP currently talks to Cursor through the AgentService run path, where external tool advertisements are encoded as `mcp_tools`. Client-native harness tools must not be placed in that MCP lane. UMP filters native names per client profile and only advertises external/non-native tools plus the proxy-owned `cursor_codebase_search`.

Current shipped profile renderers are intentionally narrower than this native
Cursor table. Codex reads/lists/searches through `shell_command`, uses
`exec_command` for shell-stream requests, and refuses Cursor `Write` until the
proxy has enough edit context. Claude maps read to `Read` and foreground
commands to `Bash`. Droid maps read to `Read` with `file_path`, shell to
`Execute`, and background shell to `Execute` with `fireAndForget`.

### Service: `BackgroundComposerService`
- **Method**: `StreamConversation`
- **Request**: `GetComposerChatRequest`
- **Response**: `stream ConversationMessage` (contains `tool_calls`)

---

## Tool Mapping Table

| Native Tool Name | Protobuf Type | Harness Equivalent | Description |
| :--- | :--- | :--- | :--- |
| `EDIT` | `EditParams` | `replace_file_content` | Patch a file using line-based replacement. |
| `NEW_FILE` | `NewFileParams` | `write_to_file` | Create a new file in the workspace. |
| `RUN_TERMINAL_COMMANDS` | `RunTerminalCommandsParams` | `run_command` | Execute one or more shell commands. |
| `LIST_DIR` | `ListDirParams` | `list_dir` | List contents of a directory. |
| `READ_CHUNK` | `ReadChunkParams` | `view_file` | Read a specific range of lines from a file. |
| `SEARCH` | `SearchParams` | `grep_search` | Search for a pattern across the workspace. |
| `SEMANTIC_SEARCH` | `SemanticSearchParams` | `semantic_search` | Vector-based search for code snippets. |
| `READ_WITH_LINTER` | `ReadWithLinterParams` | `view_file` + Lints | Read file content along with current diagnostic errors. |
| `GET_PROJECT_STRUCTURE` | `GetProjectStructureParams` | `list_dir` (Recursive) | Get a high-level tree view of the project. |
| `CREATE_RM_FILES` | `CreateRmFilesParams` | `write_to_file` / `rm` | Bulk create or remove multiple files/directories. |

---

## Protobuf Message Definitions

### `EditParams`
Used by the model to apply changes to a file.
```protobuf
message EditParams {
  string relative_workspace_path = 1;
  optional int32 line_number = 2;
  int32 replace_num_lines = 3;
  repeated string new_lines = 4;
  optional bool replace_whole_file = 7;
  string edit_id = 5;
  FrontendEditType frontend_edit_type = 6;
  optional bool auto_fix_all_linter_errors_in_file = 8;
}
```

### `RunTerminalCommandsParams`
Used to execute shell commands.
```protobuf
message RunTerminalCommandsParams {
  repeated string commands = 1;
  string commands_uuid = 2;
}
```

### `SemanticSearchParams`
Used for codebase-wide retrieval.
```protobuf
message SemanticSearchParams {
  string query = 1;
  optional string include_pattern = 2;
  optional string exclude_pattern = 3;
  int32 top_k = 4;
  optional string index_id = 5;
  bool grab_whole_file = 6;
}
```

### `NewFileParams`
```protobuf
message NewFileParams {
  string relative_workspace_path = 1;
}
```

---

## Model-Specific Behavior

### Composer 2.0 / 2.5
- **Agentic Loop**: These models expect a multi-turn conversation where they can emit multiple tool calls in sequence.
- **Thinking Blocks**: The model often emits a `thinking` block (field 45 in `ConversationMessage`) before the tool call to explain its reasoning.
- **Linter Integration**: They frequently use `READ_WITH_LINTER` to verify that their edits didn't break anything.

### Composer 1.5
- **Direct Edits**: More likely to emit `EDIT` calls directly in response to a user prompt without extensive pre-planning "thinking" blocks.
- **Simpler Toolset**: Primarily relies on `EDIT`, `READ_CHUNK`, and `SEARCH`.
