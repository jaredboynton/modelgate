# Droid Tool Catalog

This document lists the tools available to Factory Droid, including their parameters and functional descriptions.

## Core Tools

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

### Read (`read-cli`)
- **llmId**: `Read`
- **Description**: Read file contents. Supports text (truncated at 2400 lines), images (up to 5MB), and PDFs.
- **Parameters**:
  - `path`: Absolute path.
  - `offset`: (optional) Byte offset.
  - `limit`: (optional) Byte limit.

### Grep (`grep_tool_cli`)
- **llmId**: `Grep`
- **Description**: High-performance content search using ripgrep.
- **Parameters**:
  - `pattern`: Regex pattern.
  - `glob`: (optional) File glob pattern.
  - `case_insensitive`: (bool).

### Glob (`glob-search-cli`)
- **llmId**: `Glob`
- **Description**: File path search using glob patterns.
- **Parameters**:
  - `pattern`: Glob pattern (e.g. `**/*.ts`).

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

### Exa (Search)
- `exa___web_search_exa`: Search the web.
- `exa___web_fetch_exa`: Fetch webpage content as markdown.

### Octocode (GitHub)
- `octocode___githubSearchCode`: Search code on GitHub.
- `octocode___githubGetFileContent`: Read GitHub files.

### Morph (Codebase)
- `morph-mcp___codebase_search`: Natural language codebase exploration.
- `morph-mcp___edit_file`: Semantic multi-edit tool.
