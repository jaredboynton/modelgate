# Claude Tool Catalog (Claude Code)

Claude Code utilizes a set of built-in tools for filesystem and shell interaction, supplemented by an extensive plugin system and MCP support.

## Core Built-in Tools

### Bash
- **Description**: Executes a shell command in the local environment.
- **Parameters**:
  - `command`: The command to execute.
  - `restart`: (optional) Restart the shell process.

### Read
- **Description**: Reads a file from the filesystem.
- **Parameters**:
  - `file_path`: Absolute file path.
  - `offset`: (optional) Byte offset to start reading.
  - `limit`: (optional) Maximum bytes to read.

### Edit
- **Description**: Modifies a file. Typically uses a semantic diff or "search and replace" block. (Cursor `Write` / `Edit` requests are currently refused by the adapter).

### Glob
- **Description**: Finds files matching a glob pattern.
- **Parameters**:
  - `pattern`: The glob pattern to search for.

### Grep
- **Description**: Searches for text within files using a pattern.
- **Parameters**:
  - `pattern`: The search string or regex.
  - `path`: (optional) Subdirectory/file path to limit search scope.
  - `output_mode`: (optional) Match output formatting.

### LS
- **Description**: Lists files in a directory. Note: In `claude_code.rs` this maps directly to `Bash` executing `ls {path}`.


---

## Plugin & MCP Tools
Claude Code loads tools dynamically from plugins located in `~/.claude/plugins`.

### Common Plugins
- **LSP Plugins**: `typescript-lsp`, `rust-analyzer-lsp`, `swift-lsp` provide symbol navigation and type-checking.
- **MCP Servers**: Configured via `mcp.json` or `claude mcp add`.
- **Oh-My-ClaudeCode**: Enhanced context and HUD management.

### Specialized Tools
- `ViewHierarchy`: Visualizes the project or UI structure.
- `NotebookRead/Edit`: For interactive Jupyter or markdown notebooks.
