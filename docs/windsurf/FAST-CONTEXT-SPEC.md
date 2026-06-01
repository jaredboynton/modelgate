# Windsurf Fast Context Specification

This document defines and specifies the **Fast Context** retrieval system inside the Windsurf AI-native IDE (owned by Cognition, formerly developed by Codeium) and details how the local model gateway (`unified-model-proxy-v2`) represents, routes, and maps these models and protocols.

---

## 1. Executive Summary

**Fast Context** is a specialized, high-performance codebase indexing and parallel context-retrieval engine designed for the Windsurf AI-native editor. In agentic software development workflows, models often face a tradeoff between speed and intelligence: retrieving extensive files sequentially increases latency, interrupting developer flow, while failing to retrieve context leads to inaccurate edits.

Fast Context solves this by utilizing a specialized, reinforcement-learning (RL)-optimized sub-agent model family (**SWE-grep** and **SWE-grep-mini**) to execute parallel code search and codebase navigation tools, returning a dense and filtered set of relevant source code snippets to the primary LLM (such as Claude 3.5 Sonnet or `swe-1.6`).

---

## 2. Technical Architecture & Mechanism

Fast Context acts as an automated, agentic pre-retrieval layer that runs prior to or in parallel with the main Cascade conversation loop:

### 2.1 The SWE-grep Sub-Agent Family
*   **SWE-grep-mini**: A lightweight model optimized for extreme throughput. It is served on specialized hardware to achieve generation speeds of over **2,800 tokens per second**.
*   **SWE-grep**: A larger variant optimized for complex semantic search and multi-step file relationship analysis.
*   **Reinforcement Learning Fine-Tuning**: Both models are trained specifically to perform codebase exploration and tool navigation rather than general-purpose coding. They excel at mapping user intents to concrete locations in a repository.

### 2.2 Execution Model
*   **Parallel Tool Execution**: During a retrieval turn, the subagent can execute up to **8 parallel search tool calls** (e.g., executing multiple `grep`, `glob`, and `read` calls simultaneously).
*   **Bounded Multi-Turn Loop**: The subagent runs for a maximum of **4 turns** to gather files, ensuring retrieval finishes in under a second (frequently a few hundred milliseconds).
*   **Context Window Optimization**: By filtering out noise and irrelevant directories, Fast Context prevents context window pollution in the main editing model, significantly reducing token consumption and minimizing upstream context-window limits.

### 2.3 User Triggering Mechanics
*   **Automatic Mode**: Activated automatically in Windsurf's Cascade panel whenever a user query requires repository-wide understanding.
*   **Forced Mode**: Users can explicitly trigger Fast Context search by submitting a chat prompt with `Cmd + Enter` (on macOS) or `Ctrl + Enter` (on Windows/Linux), forcing a codebase-wide retrieval pass.

---

## 3. Proxy and Gateway Implementation

In the `unified-model-proxy-v2` gateway, Windsurf models are defined, aliased, and translated to interface with the upstream Codeium/Windsurf APIs.

### 3.1 Model Definitions & Aliases
Model configurations reside in [model_alias.rs](file:///Users/jaredboynton/__devlocal/unified-model-proxy-v2/src/model_alias.rs). The proxy exposes specific slugs that route to Windsurf's fast-retrieval upstream endpoints:

*   **`swe-1.6-fast` / `swe-1-6-fast`**: Triggers Windsurf's upstream `swe-1-6-fast` model, which enforces Fast Context sub-agent pre-retrieval.
*   **`swe-1.5-fast` / `swe-1-5-fast`**: Triggers the older `swe-1-5` fast-variant upstream.
*   **`swe-grep-mini` / `windsurf/swe-grep-mini`**: Exposes the lightweight Fast Context retrieval sub-agent directly for client model pickers and targeted probes.
*   **`swe-grep` / `windsurf/swe-grep`**: Exposes the larger Fast Context retrieval sub-agent directly for client model pickers and targeted probes.
*   **`adaptive`**: An auto-routing alias that chooses between fast retrieval and smart editing based on query complexity.

#### Model Visibility Diagnostics
The aggregate OpenAI-compatible facade is `GET /v1/models`. It includes the static Windsurf aliases above, hot-route aliases, and live Cursor discovery. The provider-scoped Codex route, `GET /api/provider/openai/v1/models`, is intentionally not the aggregate catalog: it fetches the private Codex model list and therefore will not show Windsurf rows.

If a client cannot see `swe-grep-mini`, `swe-grep`, or the `swe-1.6` aliases:
1. Verify the client base URL is `http://127.0.0.1:18743/v1`, not `http://127.0.0.1:18743/api/provider/openai/v1`.
2. Verify the running daemon has the catalog change with `curl -fsS http://127.0.0.1:18743/v1/models | jq -r '.data[].id' | grep '^swe'`.
3. Restart the launchd service or active development binary if the source tree has the rows but the live daemon does not.
4. Treat model visibility separately from protected-model authorization: the rows can appear in `/v1/models` while upstream inference still requires the `AssignModel` flow described below.

### 3.2 Upstream Transport (Connect Protocol)
The gateway communicates with the Codeium API via the gRPC-Web-compatible Connect protocol over HTTP/2. The implementation details are managed in [windsurf.rs](file:///Users/jaredboynton/__devlocal/unified-model-proxy-v2/src/upstream/windsurf.rs):

*   **Endpoint**: `/exa.api_server_pb.ApiServerService/GetChatMessage`
*   **Request Construction**:
    1.  **Metadata Envelope**: Includes api-key, client version (`1.13.104`), application details, and unique session/request IDs.
    2.  **Connect Frame Parsing**: Responses are streamed or buffered using big-endian length-prefixed frames. Chunks are collected via `drain_text_chunks` and mapped back to the client.

### 3.3 Client Tool Translation
When clients (such as Devin, Claude Code, or Codex CLI) talk to the proxy, their native tool footprints must be translated into the format expected by the Windsurf/SWE backend. This translation is handled by [windsurf_chat.rs](file:///Users/jaredboynton/__devlocal/unified-model-proxy-v2/src/adapter/windsurf_chat.rs):

*   **Devin / Claude Code / Codex CLI $\rightarrow$ Windsurf**:
    *   `read` / `read_file` $\rightarrow$ `Read` (with argument translation from `path` to `file_path`)
    *   `ls` $\rightarrow$ `LS` (with directory path translation)
    *   `search` / `grep` $\rightarrow$ `Grep` (maps pattern parameters to `glob_pattern`)
    *   `execute` / `shell` / `Bash` $\rightarrow$ `Execute` (maps directory to `cwd`)
    *   `edit` / `edit_file` $\rightarrow$ `Edit`
*   **Response Reconstruction**: Handled in [windsurf_responses.rs](file:///Users/jaredboynton/__devlocal/unified-model-proxy-v2/src/adapter/windsurf_responses.rs), structuring the raw assistant tool-calls and outputs back to the OpenAI-shaped or Responses-shaped downstream client.

### 3.4 Authorization Flow & Protected Model Access

Windsurf enforces an authorization check when calling protected models like `swe-grep` or `swe-grep-mini` to ensure they are only accessed by users with the appropriate subscription tiers (e.g. Pro, Teams, or Devin-connected seats).

#### The Authentication / Key Migration Path
The Windsurf application supports three main credential token formats:
1. **`sk-ws-01-`**: Modern Windsurf session/API tokens.
2. **`devin-session-token$`**: Session tokens minted automatically when the user is authenticated via a Devin-enabled organization/plan.
3. **`cog_`**: Cognition API keys.

All three credential formats are accepted in the `api_key` field of the request metadata envelope (`Metadata` field `3`). If a legacy Codeium API key (e.g. `wsk_...`) is provided, the Windsurf client automatically triggers an online migration via the `/exa.seat_management_pb.SeatManagementService/MigrateApiKey` RPC endpoint, which takes a `MigrateApiKeyRequest` (field `1` `api_key`) and returns a `MigrateApiKeyResponse` (field `1` `session_token` starting with `sk-ws-01-`).

#### The Model Assignment Step (`AssignModel`)
For protected models, the client cannot call the inference endpoint directly without proving model entitlement.

1. **Requesting Assignment**: The client executes a gRPC-Web/Connect call to `/exa.api_server_pb.ApiServerService/AssignModel`. The `AssignModelRequest` layout uses the following tags:
   *   `metadata` (**Field 1**, message `.exa.codeium_common_pb.Metadata`)
       *   `api_key` (**Field 3** of `Metadata`)
       *   `user_jwt` (**Field 21** of `Metadata`, optional for API key auth)
       *   `plan_name` (**Field 26** of `Metadata`)
       *   `team_id` (**Field 32** of `Metadata`)
   *   `model_router_uid` (**Field 2**, string) — e.g. `"swe-grep-mini"`
   *   `cascade_id` (**Field 3**, string) — A stable UUID representing the chat thread/session
   *   `chat_message_prompt` (**Field 5**, message `.exa.chat_pb.ChatMessagePrompt`)
       *   `id` (**Field 1**, string)
       *   `source` (**Field 2**, uint64) — role (user = 1)
       *   `prompt` (**Field 3**, string)

2. **Obtaining the JWT**: If authorized, the backend returns an `AssignModelResponse` with the layout:
   *   `assignment` (**Field 1**, message `.exa.api_server_pb.ModelAssignment`)
       *   `assignment_jwt` (**Field 1** of `ModelAssignment`, string)
       *   `model_uid` (**Field 2** of `ModelAssignment`, string)
       *   `harness_uids` (**Field 3** of `ModelAssignment`, repeated string)

3. **Inference with Signed JWT**: The client includes this token in the subsequent `GetChatMessageRequest` using:
   *   `model_assignment_jwt` (**Field 26**, string, oneof)
   *   `arena_assignment_jwt` (**Field 25**, string, oneof)

#### JWT Lifetime, Semantics, and Telemetry
*   **JWT Reuse / Caching**: The proxy can cache the `assignment_jwt` keyed by `(model_router_uid, cascade_id)`. The token is short-lived (typically 5–15 minutes) but can be safely reused for subsequent chat turns within the same Cascade session thread without calling `/AssignModel` again.
*   **Prompt Binding**: The returned `assignment_jwt` is not cryptographically bound to the hash of the `chat_message_prompt`. The prompt in the request is used by the backend for abuse detection, safety filtering, and routing telemetry, but the resulting token covers any request under the authorized model + session combination.
*   **User Identity**: For self-serve and Devin-seat seats, authentication is governed entirely by the API key in `Metadata.api_key` (Field `3`). The `Metadata.user_jwt` (Field `21`) is only used when the user logs in via browser OAuth in the first-party IDE, and can be safely left empty for proxy-based API key authentication.

#### Root Cause of Proxy `permission_denied` Errors
When an external client attempts to call `swe-grep` or `swe-grep-mini` through the ModelGate proxy without executing the `/AssignModel` protocol step:
- The proxy forwards the request directly to `/exa.api_server_pb.ApiServerService/GetChatMessage`.
- Because the request does not include the signed `model_assignment_jwt` (field 26), the upstream Codeium/Windsurf server rejects the connection with a `permission_denied` error (accompanied by an internal trace ID).
- Access to these models requires either a fully authorized, first-party Cascade IDE session token or implementing the `/AssignModel` interception and token injection flow inside the proxy gateway.

#### Catalog of Protected Models
The assignment flow governs a wide variety of sub-agent and premium inference models. The following router UIDs are protected under the `AssignModel` protocol:

*   **Sub-Agent & Context Retrieval Family**:
    *   `swe-grep` & `swe-grep-mini` (Fast Context retrieval agents)
    *   `swe-1`, `swe-1-6-fast`, `swe-1-6-live`, `swe-1-6-slow`, `swe-1-6-self-hosted`
    *   `swe-1p5` / `swe-1p5-thinking`
    *   `swe-1p6` / `swe-1p6-cognition` / `swe-1p6-throughput` / `swe-1p6-self-hosted`
*   **Premium LLM / Editing Models**:
    *   **Claude Family**:
        *   `claude-3-sonnet-20240229`, `claude-3-haiku-20240307`, `claude-3-5-haiku-20241022`
        *   `claude-3-5-sonnet` (v1/v2), `claude-3-7-sonnet` (including BYOK and OpenRouter variants)
        *   `claude-sonnet-4` (including databricks, thinking, and 1M context variants)
        *   `claude-opus-4` / `claude-opus-4-6` / `claude-opus-4-7` (including thinking, fast, and priority/tier variants from `low` to `xhigh-fast`)
        *   `claude-4-5-sonnet-thinking` / `claude-4-5-sonnet-thinking-1m`
        *   `claude-4-5-opus` / `claude-4-5-opus-thinking`
    *   **GPT Family**:
        *   `gpt-4` / `gpt-4o` / `gpt-4o-mini` / `chatgpt-4o-latest` (including legacy & vision snapshots)
        *   `gpt-5` / `gpt-5-nano` / `gpt-5-codex` / `gpt-5-pro`
        *   `gpt-5-3-codex` (low, medium, high, xhigh priority)
        *   `gpt-5-4` (medium, high, xhigh, low, none, mini)
        *   `gpt-5-5` (low, medium, high, none, xhigh)
    *   **Gemini Family**:
        *   `gemini-2-5-flash-thinking`
        *   `gemini-3` (preview, flash, pro, pro-preview, pro-high)
    *   **DeepSeek Family**:
        *   `deepseek-r1` / `deepseek-r1-slow` / `deepseek-r1-distill-llama-70b`
        *   `deepseek-v3` / `deepseek-v3-0324` / `deepseek-v3p2`
        *   `deepseek-v4-pro`
    *   **xAI Grok Family**:
        *   `grok-2`, `grok-3` / `grok-3-mini`, `grok-4`
        *   `grok-code-fast-1`, `grok-build-latest`, `grok-build-0415b`, `grok-build-0410b`
    *   **Qwen Family**:
        *   `qwen-3-coder-480b` / `qwen3-coder-fast`

---


## 4. Architectural Comparison

| Dimension | Fast Context (Windsurf) | Model Context Protocol (MCP) | Traditional RAG |
| :--- | :--- | :--- | :--- |
| **Primary Goal** | Ultra-fast codebase indexing and sub-agent retrieval for IDE context. | Open standard for connecting agents to arbitrary external tools/services. | Embedding-based document similarity search. |
| **Retrieval Method** | Agentic multi-turn parallel tool calling (`SWE-grep`). | Client-server JSON-RPC resource and tool declarations. | Vector search (Cosine similarity) on document chunks. |
| **Latency** | < 1 second (via Cerebras hardware optimization). | Variable (dependent on the external MCP server). | Low (retrieval-only, no agent loop). |
| **Flexibility** | Highly optimized for directories, files, and symbols. | General-purpose (Slack, databases, browser integration, etc.). | General text documents. |
