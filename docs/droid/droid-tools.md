# Droid Tool Catalog

This document lists the tools available to Factory Droid, including their parameters and functional descriptions.

## Core Tools

Current Droid native tool IDs observed locally include:

- Read/explore: `Read`, `LS`, `Grep`, `Glob`
- Edit: `Create`, `Edit`, `ApplyPatch`
- Execute/agentic: `Execute`, `Task`, `Skill`, `ToolSearch`, `GenerateDroid`
- Web: `WebSearch`, `FetchUrl`
- Planning/session: `TodoWrite`, `DismissHandoffItems`, `EndFeatureRun`, `ExitSpecMode`, `ProposeMission`, `StartMissionRun`

### Edit (`edit-cli`)
- **llmId**: `Edit`
- **Description**: Edit the contents of a file by finding and replacing text. Requires the file to be read first.
- **Parameters**:
  - `path`: Absolute file path.
  - `old_str`: Unique string to be replaced.
  - `new_str`: Replacement string.
  - `change_all`: (bool) Replace all occurrences if true.

### Create (`create-cli`)
- **llmId**: `Create`
- **Description**: Creates a new file on the file system.
- **Parameters**:
  - `path`: Absolute file path.
  - `content`: File content.

### Execute (`execute-cli`)
- **llmId**: `Execute`
- **Description**: Execute a shell command. Each command runs in a new, isolated shell process.
- **Parameters**:
  - `command`: The command to run.
  - `timeout`: (optional) Timeout in seconds (default 90).
  - `fireAndForget`: (bool) Run in background.

### LS (`list-directory`)
- **llmId**: `LS`
- **Description**: List directory contents.
- **Parameters**:
  - `path`: Directory path.

### Read (`read-cli`)
- **llmId**: `Read`
- **Description**: Read file contents. Supports text (truncated at 2400 lines), images (up to 5MB), and PDFs.
- **Parameters**:
  - `file_path`: Absolute path.
  - `offset`: (optional) Byte offset.
  - `limit`: (optional) Byte limit.

### Grep (`grep_tool_cli`)
- **llmId**: `Grep`
- **Description**: High-performance content search using ripgrep.
- **Parameters**:
  - `pattern`: Regex pattern.
  - `path`: (optional) Absolute file or directory to search in.
  - `glob_pattern`: (optional) File glob pattern.
  - `case_insensitive`: (bool).

### Glob (`glob-search-cli`)
- **llmId**: `Glob`
- **Description**: File path search using glob patterns.
- **Parameters**:
  - `patterns`: Glob pattern string or array of glob patterns (e.g. `**/*.ts`).
  - `excludePatterns`: (optional) Glob pattern string or array to exclude.
  - `folder`: (optional) Absolute directory to search in.

### WebSearch / FetchUrl
- **llmId**: `WebSearch`, `FetchUrl`
- **Description**: Search the web or fetch URL content.

---

## Agentic & Mission Tools

### Task (`task-cli`)
- **llmId**: `Task`
- **Description**: Launch a subagent (custom droid) to handle a complex task.
- **Parameters**:
  - `subagent_type`: Droid identifier (e.g. `worker`, `codebase-scout`).
  - `prompt`: The task description.

### ProposeMission (`propose-mission`)
- **Description**: Present a multi-feature mission plan for user review.

### StartMissionRun (`start-mission-run`)
- **Description**: Start the sequential execution of features in a mission.

---

## MCP Tools (Pre-installed)

UMP treats Droid native tools and external MCP tools as separate namespaces:

- Droid native names are filtered out of Cursor `mcp_tools` advertisements.
- External MCP tools should stay namespaced as `server___tool`.
- `opencode` MCP calls with raw Droid-native names such as `Read` or `TodoWrite` are refused as native-tool leaks.
- `opencode` MCP calls with non-native names, including already-namespaced names such as `ref___ref_search_documentation`, pass through unchanged.

### Exa (Search)
- `exa___web_search_exa`: Search the web.
- `exa___web_fetch_exa`: Fetch webpage content as markdown.

### Octocode (GitHub)
- `octocode___githubSearchCode`: Search code on GitHub.
- `octocode___githubGetFileContent`: Read GitHub files.

### Morph (Codebase)
- `morph-mcp___codebase_search`: Natural language codebase exploration.
- `morph-mcp___edit_file`: Semantic multi-edit tool.
