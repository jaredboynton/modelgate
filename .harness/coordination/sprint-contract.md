# Sprint Contract

## Goal

Add repo harness surfaces that make future agent work safer without changing proxy runtime behavior.

## Non-Negotiables

- Runtime behavior stays unchanged.
- No live provider credentials in tests, examples, captures, or outputs.
- `.env`, `.omx/`, `target/`, logs, and coverage artifacts stay untracked.
- Clone/distribution readiness is not claimed while local path dependencies remain.

## Lane Contract

Each lane records:

- `Owner`: agent or person responsible.
- `Write scope`: exact paths it may change.
- `Inputs`: plan, issue, or handoff consumed.
- `Done when`: validation evidence and no known blockers.

## Conflict Rule

If two lanes need the same file, stop editing that file and record the conflict in `coordination/board.md`.
