# Coordination Board

Use this board for additive harness/runtime-safe work only. Do not use it to claim ownership of runtime proxy files unless a lead explicitly widens scope.

## Active Lanes

| Lane | Owner | Scope | Status | Blocker | Last Evidence |
|---|---|---|---|---|---|
| 1 |  |  | todo |  |  |
| 2 |  |  | todo |  |  |
| 3 |  | `.harness/**` | in progress |  |  |

## Status Values

- `todo`: not started.
- `in progress`: files are being edited.
- `review`: ready for another agent to inspect.
- `blocked`: cannot proceed without lead decision.
- `done`: validated and no follow-up remains in this scope.

## Coordination Rules

- Write scope before editing.
- Keep claims evidence-backed: command output, changed files, or blocker.
- Do not revert another lane. Escalate conflicts in `Blocker`.
