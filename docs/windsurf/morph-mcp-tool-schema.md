# Morph MCP Tool Schema Reference

Observed from local `@morphllm/morphmcp` version `0.8.181` at:

`/Users/jaredboynton/.local/state/fnm_multishells/97453_1779846321202/bin/morph-mcp`

The binary is a Node stdio MCP server. It is invoked as:

```bash
morph-mcp [options] [allowed-directory ...]
```

Options:

- `--api-key <key>`: Morph API key, taking priority over `MORPH_API_KEY`.
- `-h`, `--help`: print help.
- `-v`, `--version`: print version.

Runtime behavior relevant to `fast-context-mcp`:

- Uses MCP stdio transport.
- Supports MCP Roots; otherwise falls back to command-line directories or workspace mode.
- Filters all API-backed tools out of `tools/list` when no API key is configured.
- Supports `DISABLED_TOOLS` as a comma-separated allowlist removal mechanism.

## Advertised Tools

### `edit_file`

Purpose: semantic file editing through Morph FastApply.

Input schema:

```json
{
  "type": "object",
  "properties": {
    "path": {
      "type": "string"
    },
    "code_edit": {
      "type": "string",
      "description": "Changed lines with minimal context. Use placeholders intelligently like \"// ... existing code ...\" to represent unchanged code."
    },
    "instruction": {
      "type": "string",
      "description": "A brief single first-person sentence instruction describing changes being made to this file. Useful to disambiguate uncertainty in the edit."
    },
    "dryRun": {
      "type": "boolean",
      "default": false,
      "description": "Preview changes without applying them."
    }
  },
  "required": ["path", "code_edit", "instruction"],
  "additionalProperties": false
}
```

The implementation also accepts call-time aliases such as `target_file`,
`target_filepath`, `file_path`, `instructions`, `code`, `new_string`,
`content`, and `text`, but these aliases are not part of the advertised schema.

### `codebase_search`

Purpose: Morph Fast Context local repository search.

Input schema:

```json
{
  "type": "object",
  "properties": {
    "search_string": {
      "type": "string",
      "description": "Natural-language question/description about the code you want to understand. This tool does NOT accept regex, keyword dumps, or symbol-only queries."
    },
    "repo_path": {
      "type": "string",
      "description": "The absolute path of the folder where the search should be performed. In multi-repo workspaces, specify a subfolder to avoid searching across all repos."
    },
    "search_type": {
      "type": "string",
      "enum": ["all", "node_modules"],
      "description": "Search type hint. Use 'node_modules' when searching inside node_modules or other dependency directories that are normally excluded."
    }
  },
  "required": ["search_string", "repo_path"],
  "additionalProperties": false
}
```

### `github_codebase_search`

Purpose: Morph Fast Context search over a GitHub repository.

Input schema:

```json
{
  "type": "object",
  "properties": {
    "search_string": {
      "type": "string",
      "description": "Natural-language question/description about the code you want to understand. This tool does NOT accept regex, keyword dumps, or symbol-only queries."
    },
    "github_url": {
      "type": "string",
      "description": "GitHub repository URL to search, e.g. 'https://github.com/vercel/next.js'. Provide either github_url or owner_repo."
    },
    "owner_repo": {
      "type": "string",
      "description": "Repository owner/repo shorthand, e.g. 'vercel/next.js'. Provide either github_url or owner_repo."
    },
    "branch": {
      "type": "string",
      "description": "Branch to search, defaulting to the repository default branch."
    }
  },
  "required": ["search_string"],
  "additionalProperties": false
}
```

The implementation validates that either `github_url` or `owner_repo` is
present at call time.

## Shape To Mirror

For `fast-context-mcp`, mirror the `codebase_search` ergonomics rather than the
edit surface:

- One primary tool.
- Natural-language query field.
- Absolute local repository path field.
- Optional search-type or result-shaping knobs.
- Read-only behavior.
- Plain text MCP content result, with structured diagnostics embedded only when
  useful.
