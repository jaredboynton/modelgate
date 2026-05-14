# Outputs

Store durable agent outputs here when they should outlive a chat turn.

## Good Outputs

- Validation summaries with command output excerpts.
- Harness diagnosis reports.
- Distribution-preflight inventories.
- Review notes tied to exact files.

## Do Not Store

- Secrets, tokens, cookies, OAuth payloads, or provider captures.
- `target/`, logs, coverage, or generated binaries.
- Large raw transcripts when a short summary is enough.

## Template

```md
# Output: <short title>

Date:
Source task:
Scope:

## Result

## Evidence

## Follow-Up
```
