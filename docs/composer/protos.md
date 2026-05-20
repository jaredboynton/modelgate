# Cursor Composer Protobuf Definitions

These definitions were extracted from `/Applications/Cursor.app/Contents/Resources/app/out/vs/workbench/workbench.desktop.main.js`.

## Service: `BackgroundComposerService`

```protobuf
service BackgroundComposerService {
  rpc ListBackgroundComposers(ListBackgroundComposersRequest) returns (ListBackgroundComposersResponse);
  rpc AttachBackgroundComposer(AttachBackgroundComposerRequest) returns (stream BackgroundComposerServerMessage);
  rpc StreamConversation(GetComposerChatRequest) returns (stream ConversationMessage);
  rpc GetLatestAgentConversationState(GetLatestAgentConversationStateRequest) returns (GetLatestAgentConversationStateResponse);
  rpc GetBlobForAgentKV(GetBlobForAgentKVRequest) returns (GetBlobForAgentKVResponse);
  rpc AttachBackgroundComposerLogs(AttachBackgroundComposerLogsRequest) returns (stream BackgroundComposerLog);
  rpc StartBackgroundComposerFromSnapshot(StartBackgroundComposerFromSnapshotRequest) returns (StartBackgroundComposerFromSnapshotResponse);
  rpc MakePRBackgroundComposer(MakePRBackgroundComposerRequest) returns (MakePRBackgroundComposerResponse);
  rpc OpenPRBackgroundComposer(OpenPRBackgroundComposerRequest) returns (OpenPRBackgroundComposerResponse);
  rpc GetBackgroundComposerStatus(GetBackgroundComposerStatusRequest) returns (GetBackgroundComposerStatusResponse);
  rpc AddAsyncFollowupBackgroundComposer(AddAsyncFollowupBackgroundComposerRequest) returns (AddAsyncFollowupBackgroundComposerResponse);
  // ... and many more
}
```

## Message: `GetComposerChatRequest`

```protobuf
message GetComposerChatRequest {
  repeated ConversationMessage conversation = 1;
  optional bool allow_long_file_scan = 2;
  ExplicitContext explicit_context = 3;
  optional bool can_handle_filenames_after_language_ids = 4;
  ModelDetails model_details = 5;
  LinterErrors linter_errors = 6;
  repeated string documentation_identifiers = 7;
  optional string use_web = 8;
  repeated ComposerExternalLink external_links = 9;
  optional ConversationMessage project_context = 10;
  repeated RedDiff diffs_for_compressing_files = 11;
  optional bool compress_edits = 12;
  optional bool should_cache = 13;
  repeated LinterErrors multi_file_linter_errors = 14;
  CurrentFile current_file = 15;
  RecentEdits recent_edits = 16;
  optional bool use_reference_composer_diff_prompt = 17;
  repeated FileDiffHistory file_diff_histories = 18;
  optional bool use_new_compression_scheme = 19;
  repeated AdditionalRankedContext additional_ranked_context = 20;
  repeated Quote quotes = 21;
  optional bool willing_to_pay_extra_for_speed = 22;
  string conversation_id = 23;
  optional bool use_unified_chat_prompt = 24;
  optional bool use_full_inputs_context = 25;
  optional bool is_resume = 26;
  optional string context_bank_session_id = 27;
  optional int32 context_bank_version = 28;
  optional bytes context_bank_encryption_key = 31;
  CodebaseResults uses_codebase_results = 29;

  message RedDiff {
    string relative_workspace_path = 1;
    repeated Range red_ranges = 2;
    repeated Range red_ranges_reversed = 3;
    string start_hash = 4;
    string end_hash = 5;
  }
}
```

## Message: `ComposerCapabilityRequest`

```protobuf
message ComposerCapabilityRequest {
  enum Type {
    UNSPECIFIED = 0;
    LOOP_ON_LINTS = 1;
    LOOP_ON_TESTS = 2;
    MEGA_PLANNER = 3;
    LOOP_ON_COMMAND = 4;
    TOOL_CALL = 5;
    DIFF_REVIEW = 6;
    CONTEXT_PICKING = 7;
    EDIT_TRAIL = 8;
    AUTO_CONTEXT = 9;
    CONTEXT_PLANNER = 10;
    DIFF_HISTORY = 11;
    REMEMBER_THIS = 12;
    DECOMPOSER = 13;
    CURSOR_RULES = 14;
  }

  Type type = 1;

  oneof data {
    LoopOnLints loop_on_lints = 2;
    LoopOnTests loop_on_tests = 3;
    MegaPlanner mega_planner = 4;
    LoopOnCommand loop_on_command = 5;
    ToolCall tool_call = 6;
    DiffReview diff_review = 7;
    ContextPicking context_picking = 8;
    EditTrail edit_trail = 9;
    AutoContext auto_context = 10;
    ContextPlanner context_planner = 11;
    RememberThis remember_this = 12;
    Decomposer decomposer = 13;
    CursorRules cursor_rules = 14;
  }
}
```

## Message: `ModelDetails`

```protobuf
message ModelDetails {
  optional string model_name = 1;
  optional string api_key = 2;
  optional bool enable_ghost_mode = 3;
  optional AzureState azure_state = 4;
  optional bool enable_slow_pool = 5;
  optional string openai_api_base_url = 6;
  optional BedrockState bedrock_state = 7;
  optional bool max_mode = 8;
}
```

## Message: `ConversationMessage` (Partial)

```protobuf
message ConversationMessage {
  string text = 1;
  ConversationMessageType type = 2;
  repeated AttachedCodeChunk attachedCodeChunks = 3;
  repeated CodebaseContextChunk codebaseContextChunks = 4;
  repeated Commit commits = 5;
  repeated PullRequest pullRequests = 6;
  repeated GitDiff gitDiffs = 7;
  repeated AssistantSuggestedDiff assistantSuggestedDiffs = 8;
  repeated InterpreterResult interpreterResults = 9;
  repeated Image images = 10;
  repeated AttachedFolder attachedFolders = 11;
  repeated ApproximateLintError approximateLintErrors = 12;
  string bubbleId = 13;
  optional string serverBubbleId = 14;
  // ... and many more (40+ fields)
  bool isAgentic = 31;
  // ...
}
```
