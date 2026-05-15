# Switchyard Atlas browser config routing diagram

`/config` is the local route-map UI for UMP's static routing state. It shows the effective route map the proxy will use, including built-in catalog routes when the hot config has no overrides. Switchyard Atlas keeps editing graph-backed: typed editor controls and any raw JSON fallback must both validate through the same server projection before save.

Canonical plan artifacts:

- `.omx/plans/prd-browser-config-routing-diagram.md`
- `.omx/plans/test-spec-browser-config-routing-diagram.md`

## Route surfaces

- `GET /config` serves the browser route map. It is a local admin UI, not a public dashboard and not observed traffic.
- `GET /api/config` returns the raw persisted hot config. An empty hot config can still produce effective catalog routes in the graph.
- `PUT /api/config` strictly saves a complete hot config JSON document. Unknown fields, unsupported formats/providers, and secret-shaped keys fail before any write. The proxy re-reads this file on each request, so saved routing applies to later requests without restart.
- `GET /api/config/graph` returns the persisted server projection used by the UI table, graph, typed editor, inspector, and diagnostics.
- `POST /api/config/graph` validates a draft JSON config and returns a projected graph without writing it. It uses the same no-secret and no-unknown-field parser as persistence and must not echo secret values in errors.

The graph projection is server-owned. UI code should render `effective_routes`, diagnostics, nodes, and edges from `/api/config/graph` instead of reimplementing routing precedence in JavaScript.

## Graph v2 and typed editor behavior

Switchyard Atlas treats the graph endpoint as the v2 UI contract even while the wire field name remains `schema_version`. Final schema numbering still needs confirmation from the implementation lane before release notes are cut.

Expected graph fields:

- `schema_version`, `generated_at`, and `raw_hot_config` identify the projection and the exact draft or persisted JSON that produced it.
- `runtime_formats` names active request runtimes such as `responses`, `chat_completions`, and `anthropic_messages`; inactive accepted formats can appear as diagnostics or inactive rows.
- `config_routes` describes mutable hot-config rows with `row_id`, `config_index`, `enabled`, source/target fields, and row `state`.
- `effective_routes` is the winning route set the proxy will use after catalog and hot-config precedence are applied.
- `nodes` and `edges` are display primitives only. The UI may style and select them, but route semantics still come from server-projected route rows.
- `diagnostics` and `validation_issues` drive banners, row warnings, and editor feedback. The UI should display them without suppressing unknown codes.

Typed editor expectations:

- Load starts from `GET /api/config` plus `GET /api/config/graph`; current config is read-only until the user edits a draft.
- Field-level edits serialize back to the canonical hot-config shape: top-level `routes`, each route with `source`, `target`, and optional `enabled`.
- Validate uses `POST /api/config/graph`; success renders the draft projection and must not mutate `GET /api/config` or the file behind `UMP_V2_CONFIG`.
- Save uses `PUT /api/config`; it is the only browser write path and must send the complete hot-config document, not a patch.
- Keyboard paths should keep the same split: validate shortcut triggers dry-run `POST`, save shortcut triggers strict `PUT`.
- Unknown fields, forbidden secret keys such as `api-key`, `token`, `authorization`, `password`, or `credential`, unsupported target providers, and malformed JSON must surface as validation errors without persisting.

## Security expectations

`/config`, `/config/assets/*`, `/api/config`, and `/api/config/graph` are local admin surfaces:

- Require loopback `Host`: `localhost`, `127.0.0.1`, or `[::1]`, with optional port.
- Allow missing `Origin` for CLI/local clients, but reject unsafe methods with foreign `Origin` or `Sec-Fetch-Site: cross-site`.
- Treat `POST /api/config/graph` as unsafe for browser-origin checks even though it is non-persistent; it can parse sensitive-looking draft content and must stay loopback-only.
- Return `Cache-Control: no-store` on HTML, assets, JSON, and guard failures.
- Return `X-Content-Type-Options: nosniff` on UI/assets and guard failures.
- Serve `/config` with a restrictive CSP that allows same-origin CSS/JS assets and same-origin API calls only.
- Keep scripts and styles in `/config/assets/config.js` and `/config/assets/config.css`; do not add inline scripts, inline styles, event attributes, or remote assets.

## Safe browser and CDP smoke

Run browser/CDP smoke only against a temporary local process so live auth homes and routing files are not touched.

```sh
tmpdir="$(mktemp -d)"
export UMP_V2_AUTH_HOME="$tmpdir/auth"
export UMP_V2_CODEX_HOME="$tmpdir/codex"
export UMP_V2_CONFIG="$tmpdir/config.json"
export UMP_V2_LISTEN_ADDR="127.0.0.1:0"
printf '{"routes":[]}' > "$UMP_V2_CONFIG"
cargo run
```

Use the ephemeral address printed by the process logs. Do not point CDP, Playwright, Chrome remote debugging, or the in-app browser at a developer's normal `127.0.0.1:18743` process for this checklist.

Final smoke checklist:

1. Wait for `GET /health` to return `200`.
2. Open `http://<ephemeral-addr>/config`.
3. Capture one screenshot or DOM dump and browser console output.
4. Compare the visible route table count with `GET /api/config/graph` `effective_routes.length`.
5. Confirm empty config shows built-in effective routes and the `No hot overrides` message.
6. Confirm the graph response includes `config_routes`, `effective_routes`, `nodes`, `edges`, `diagnostics`, `runtime_formats`, and `raw_hot_config`.
7. Select a route and confirm the inspector or typed editor shows the same route identity.
8. Edit a draft route and validate through `POST /api/config/graph`; confirm the draft projection renders and `GET /api/config` did not change.
9. Add an unknown field to the draft and confirm `POST /api/config/graph` returns `invalid_routing_config` without writing.
10. Add a forbidden secret-shaped key such as `api-key` to the draft and confirm `POST /api/config/graph` rejects it without echoing the secret value.
11. Save a valid full config through the UI or `PUT /api/config`, reload, and confirm the saved route appears.
12. Send malformed JSON or a route missing required `source`/`target` fields to `PUT /api/config` and confirm persistence is unchanged.
13. Exercise validate and save keyboard paths and confirm the console has no errors.
14. Confirm all requests in the browser network log target the temp ephemeral address and temp `UMP_V2_CONFIG`.

Clean up the temp directory after the process exits.

## Assumptions needing final confirmation

- Whether Switchyard Atlas graph v2 should increment the wire `schema_version` from the currently observed `1`.
- Whether the final typed editor is form-first with raw JSON fallback, or raw JSON remains the primary editing affordance for this release.
- Whether strict `PUT` should continue treating `{}` as an empty config through the current `routes` default, or require an explicit top-level `routes` array before release.
- Whether browser smoke will use Playwright, Chrome DevTools Protocol directly, or the Codex in-app browser; all are acceptable only when pointed at the temp-home process.
- Whether code lanes add route-level integration tests for strict `PUT` rejecting unknown fields and forbidden secret keys, or rely on shared hot-config/parser tests plus graph POST coverage.

## Validation prep

Expected release checks once implementation lanes are integrated:

```sh
cargo fmt --check
cargo test hot_config_graph
cargo test --test integration_routes integration_routes_config
cargo test router_config_admin_guard
cargo test
cargo clippy --all-targets --all-features -- -D warnings
```

Current lane note: this document should stay aligned with the implemented route-map contract. If `/api/config/graph`, external `/config/assets/*`, or the admin guard are absent in a checkout, treat browser smoke as blocked until those implementation lanes land.
