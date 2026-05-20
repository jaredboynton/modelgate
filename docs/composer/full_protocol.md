# aiserver.v1 Protocol Mapping (Thorough)

This document contains a thorough mapping of the `aiserver.v1` protocol core messages used by Cursor Composer.

## Service: BackgroundComposerService
Found at offset `32429281` in `workbench.desktop.main.js`.

| Method | Request Type | Response Type | Kind |
| :--- | :--- | :--- | :--- |
| `ListBackgroundComposers` | `ListBackgroundComposersRequest` | `ListBackgroundComposersResponse` | Unary |
| `AttachBackgroundComposer` | `AttachBackgroundComposerRequest` | `stream ConversationMessage` | ServerStreaming |
| `StreamConversation` | `StreamUnifiedChatRequest` | `stream StreamUnifiedChatResponse` | ServerStreaming |
| `GetLatestAgentConversationState` | `GetLatestAgentConversationStateRequest` | `ConversationMessage` | Unary |
| `GetBlobForAgentKV` | `GetBlobForAgentKVRequest` | `GetBlobForAgentKVResponse` | Unary |
| `AttachBackgroundComposerLogs` | `AttachBackgroundComposerLogsRequest` | `stream LogUpdate` | ServerStreaming |
| `GetBackgroundComposerStatus` | `GetBackgroundComposerStatusRequest` | `BackgroundComposerStatus` | Unary |
| `PauseBackgroundComposer` | `PauseBackgroundComposerRequest` | `PauseBackgroundComposerResponse` | Unary |
| `ResumeBackgroundComposer` | `ResumeBackgroundComposerRequest` | `ResumeBackgroundComposerResponse` | Unary |
| `DeleteBackgroundComposer` | `DeleteBackgroundComposerRequest` | `DeleteBackgroundComposerResponse` | Unary |

---

## Modern Request/Response (Composer 2.0+)

### StreamUnifiedChatRequest
This is the primary request type for agentic Composer interactions.

| Field | Name | Type | Description |
| :--- | :--- | :--- | :--- |
| 1 | `conversation` | `repeated ConversationMessage` | Full message history. |
| 30 | `full_conversation_headers_only` | `repeated ConversationMessageHeader` | Lightweight history summary. |
| 3 | `explicit_context` | `ExplicitContext` | User-provided context. |
| 5 | `model_details` | `ModelDetails` | Model name, API keys, etc. |
| 6 | `linter_errors` | `LinterErrors` | Active linter issues. |
| 10 | `project_context` | `ConversationMessage` | High-level project state. |
| 15 | `current_file` | `CurrentFileInfo` | File content and cursor position. |
| 18 | `file_diff_histories` | `repeated FileDiffTrajectory` | History of changes in the session. |
| 23 | `conversation_id` | `string` | Unique ID. |
| 27 | `is_agentic` | `bool` | Enables autonomous tool use. |
| 29 | `supported_tools` | `repeated enum BuiltinTool` | Tools the model is allowed to call. |
| 34 | `mcp_tools` | `repeated McpTool` | External MCP-provided tools. |
| 42 | `uses_codebase_results` | `CodebaseResults` | Semantic/Vector search context. |
| 46 | `unified_mode` | `enum UnifiedMode` | CHAT, AGENT, EDIT, PLAN, etc. |
| 49 | `thinking_level` | `enum ThinkingLevel` | MEDIUM, HIGH (for O1/Composer-2). |
| 89 | `current_plan` | `CurrentPlan` | The active agentic plan. |

### StreamUnifiedChatResponse

| Field | Name | Type | Description |
| :--- | :--- | :--- | :--- |
| 1 | `text` | `string` | Incremental markdown content. |
| 12 | `status_updates` | `StatusUpdate` | Progress messages for the UI. |
| 13 | `tool_call` | `BuiltinToolCall` | Model request to execute a tool. |
| 36 | `tool_call_v2` | `ToolCallV2` | Enhanced tool call format. |
| 25 | `thinking` | `ThinkingBlock` | Reasoning text (hidden from final output). |
| 37 | `thinking_style` | `enum ThinkingStyle` | Style of reasoning used. |
| 29 | `subagent_return` | `SubagentReturn` | Result from a spawned sub-agent. |

---

## Tooling & Parameters

### BuiltinToolCall (oneof `params`)
| Field | Name | Type (Params) |
| :--- | :--- | :--- |
| 2 | `search` | `SearchParams` |
| 3 | `read_chunk` | `ReadChunkParams` |
| 5 | `edit` | `EditParams` |
| 8 | `new_file` | `NewFileParams` |
| 15 | `semantic_search` | `SemanticSearchParams` |
| 16 | `get_project_structure` | `GetProjectStructureParams` |
| 17 | `create_rm_files` | `CreateRmFilesParams` |
| 18 | `run_terminal_commands` | `RunTerminalCommandsParams` |
| 20 | `read_with_linter` | `ReadWithLinterParams` |

### BuiltinToolResult (oneof `result`)
Maps 1:1 to the fields in `BuiltinToolCall`.

---

## Core Enums

### UnifiedMode
- `UNSPECIFIED` = 0
- `CHAT` = 1
- `AGENT` = 2
- `EDIT` = 3
- `CUSTOM` = 4
- `PLAN` = 5
- `DEBUG` = 6

### ThinkingStyle
- `UNSPECIFIED` = 0
- `DEFAULT` = 1
- `CODEX` = 2
- `GPT5` = 3

### ThinkingLevel
- `UNSPECIFIED` = 0
- `MEDIUM` = 1
- `HIGH` = 2

### MessageType
- `UNSPECIFIED` = 0
- `HUMAN` = 1
- `AI` = 2
