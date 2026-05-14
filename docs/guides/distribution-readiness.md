# Distribution readiness

Use this guide before creating a remote, sharing the repo, or shipping binaries.
It describes the current blocker state; it does not change runtime behavior.

## Current status

- Local development can validate the current checkout with Cargo commands.
- Remote clone portability is blocked by the local `specter` path dependency in `Cargo.toml`.
- Hosted CI is manual/pending or dependency-gated until `specter` is portable.
- Runtime secrets, `.env`, `.omx/`, `target/`, logs, and live captures must stay untracked.

## Pre-remote blocker

`Cargo.toml` currently contains:

```toml
specter = { package = "specters", path = "/Users/jaredboynton/__devlocal/specter" }
```

This is a clone-affecting local path. Treat it as a pre-remote blocker. Do not
hide it, paper over it in CI, or call the repository distribution-ready until one
of these decisions lands:

- publish the needed `specters` crate version and depend on it normally;
- use a stable git dependency with the required WebSocket APIs;
- vendor the dependency with an explicit policy; or
- replace the dependency through a reviewed transport decision.

This lane records the blocker only. It does not fix or replace `specter`.

## Manual checklist

Before remote creation:

- `git status --short` shows only intended tracked files.
- No tracked `.env`, `.omx/`, `target/`, logs, live captures, or local auth files.
- `Cargo.toml` has no clone-breaking local dependency paths.
- `cargo metadata --locked --format-version=1` works outside the original machine path assumptions.
- `cargo build --locked` works from a clean clone.
- License, visibility, package naming, and release target are decided.
- README can be updated after active provider/WSS work lands.

Before distribution:

- `cargo fmt --check`
- `cargo check`
- `cargo test`
- `cargo clippy --all-targets --all-features -- -D warnings`
- launch assets reviewed for local-only paths and developer-specific state.

## CI truthfulness

Do not describe GitHub-hosted CI as required-green for a clean remote clone while
`specter` points at `/Users/jaredboynton/__devlocal/specter`. The honest current
claim is:

> CI scaffolding is manual/pending or dependency-gated; remote-required CI waits
> on `specter` portability.

When `specter` is portable, CI can require the normal gates: format, check,
test, clippy, and build, with no live credentials.

## Local path inventory

Absolute local paths in docs and planning notes are inventory items unless they
affect build or clone behavior. Absolute local paths in manifests, workflows,
launch assets, or scripts need stronger review. The manifest `specter` path is
the known hard pre-remote blocker.
