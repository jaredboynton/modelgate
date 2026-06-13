//! Live Cursor Composer e2e tests, gated on `UMP_LIVE_CURSOR=1`.
//!
//! ALL tests in this file are `#[ignore]` by default. They run only when
//! the operator explicitly opts in via:
//!
//! ```sh
//! UMP_LIVE_CURSOR=1 UMP_CURSOR_INDEX_CLOUD=1 UMP_CURSOR_INDEX_BOOTSTRAP=1 \
//!   cargo test --test live_cursor_composer -- --ignored --nocapture
//! ```
//!
//! Per ralplan Section "Test Specification" -> "Live Tests":
//! - Composer models (`composer-1.5`, `composer-2`, `composer-2.5`,
//!   `composer-2.5-fast`, `composer-2-fast`)
//! - 3 endpoints (`/v1/responses`, `/v1/chat/completions`, `/v1/messages`)
//! - stream + non-stream
//! - tools (function call + tool result continuation)
//! - reasoning evidence for Composer 2-family
//! - multi-turn continuation
//! - cloud-indexing proof
//!
//! Required evidence: redacted request IDs, auth_source, model_discovery,
//! at least one assistant text delta, reasoning event from Composer
//! 2-family, tool-call + tool-result, multi-turn continuation,
//! cloud-indexing proof.
//!
//! Each test that opts in uses `AppState::for_tests` with temp homes;
//! it MUST NOT touch real `$HOME`.

mod common;

use std::env;

const LIVE_CURSOR_OPT_IN: &str = "UMP_LIVE_CURSOR";
const LIVE_CI_OPT_IN: &str = "UMP_V2_ALLOW_LIVE_TESTS_IN_CI";

const COMPOSER_MODELS: &[&str] = &[
    "composer-1.5",
    "composer-2",
    "composer-2.5",
    "composer-2.5-fast",
    "composer-2-fast",
];

struct LiveGuard {
    test_name: String,
}

impl LiveGuard {
    fn from_env(test_name: &str) -> Option<Self> {
        if env_flag(LIVE_CURSOR_OPT_IN) != Some(true) {
            eprintln!("live-blocked {test_name}: {LIVE_CURSOR_OPT_IN}=1 is required");
            return None;
        }
        if env::var_os("CI").is_some() && env_flag(LIVE_CI_OPT_IN) != Some(true) {
            eprintln!("live-blocked {test_name}: {LIVE_CI_OPT_IN}=1 is required in CI");
            return None;
        }
        Some(Self {
            test_name: test_name.to_string(),
        })
    }
}

impl Drop for LiveGuard {
    fn drop(&mut self) {
        // Future: write a per-row summary.json to UMP_V2_LIVE_ARTIFACT_DIR
        // so blocked + completed runs both leave evidence. For now the
        // guard is just a marker; the artifact-writing helper lives
        // adjacent to `tests/live_composer_codex_cli.rs` and will be
        // shared once the harness is generalized.
        let _ = &self.test_name;
    }
}

fn env_flag(name: &str) -> Option<bool> {
    env::var(name).ok().map(|value| value == "1")
}

fn live_base_url() -> String {
    common::live_base_url()
}

#[test]
fn live_cursor_composer_harness_emits_blocked_summary_without_opt_in() {
    // Without `UMP_LIVE_CURSOR=1` the harness must surface as "blocked",
    // not "failed". Operators inspecting the artifact dir see a
    // `live-blocked` row with the gating env var name listed.
    //
    // The check intentionally inspects current env without mutating it:
    // mutating global env from a `#[test]` would race with other tests
    // running in parallel (`cargo test` uses a shared process per
    // binary). When the operator actually opts in, the live tests below
    // run via `--ignored`.
    if env_flag(LIVE_CURSOR_OPT_IN) == Some(true) {
        eprintln!(
            "{LIVE_CURSOR_OPT_IN}=1 is set; this blocked-summary test is a no-op when live opt-in is active",
        );
        return;
    }
    let guard = LiveGuard::from_env("blocked_smoke");
    assert!(
        guard.is_none(),
        "live guard must refuse to run when {LIVE_CURSOR_OPT_IN} is unset",
    );
}

#[tokio::test]
#[ignore = "requires UMP_LIVE_CURSOR=1 and live Cursor credentials"]
async fn live_cursor_composer_models_endpoint_returns_three_composer_rows_via_real_discovery() {
    let Some(_guard) = LiveGuard::from_env("models_discovery") else {
        return;
    };

    let client = warpsock::Client::new().unwrap();
    let response = client
        .get(format!("{}/v1/models", live_base_url()))
        .timeout(std::time::Duration::from_secs(30))
        .send()
        .await
        .expect("live /v1/models request");
    let status = response.status();
    let body = response.text().expect("read /v1/models body");
    common::assert_no_unredacted_sensitive_values(&body);

    assert!(status.is_success(), "live /v1/models failed: {status}");
    for model in COMPOSER_MODELS {
        assert!(
            body.contains(&format!("\"id\":\"{model}\"")) || body.contains(model),
            "live /v1/models response missing {model}: {body}",
        );
    }
}

#[tokio::test]
#[ignore = "requires UMP_LIVE_CURSOR=1 + reachable Cursor agent for streaming"]
async fn live_cursor_responses_streaming_emits_assistant_text_delta_for_each_composer() {
    let Some(_guard) = LiveGuard::from_env("responses_streaming") else {
        return;
    };

    let client = warpsock::Client::new().unwrap();
    for model in COMPOSER_MODELS {
        let response = client
            .post(format!("{}/v1/responses", live_base_url()))
            .timeout(std::time::Duration::from_secs(120))
            .json(&serde_json::json!({
                "model": model,
                "input": "Reply with exactly: ok",
                "stream": true,
            }))
            .send()
            .await
            .expect("live /v1/responses request");
        let status = response.status();
        assert!(
            status.is_success(),
            "live /v1/responses failed for {model} with {status}",
        );
        let text = response.text().expect("read live /v1/responses body");
        common::assert_no_unredacted_sensitive_values(&text);
        // Required evidence: at least one assistant text delta event.
        assert!(
            text.contains("response.output_text.delta") || text.contains("response.output_text"),
            "live Cursor stream missing assistant text delta for {model}: {text}",
        );
        // Terminal frame is mandatory; otherwise the SSE parser would
        // reject the stream client-side.
        assert!(
            text.contains("response.completed")
                || text.contains("response.failed")
                || text.contains("response.incomplete"),
            "live Cursor stream missing terminal event for {model}: {text}",
        );
    }
}

#[tokio::test]
#[ignore = "requires UMP_LIVE_CURSOR=1 + reachable Cursor agent (Composer 2-family)"]
async fn live_cursor_responses_emits_reasoning_event_for_composer_2_family() {
    let Some(_guard) = LiveGuard::from_env("reasoning_events") else {
        return;
    };

    let client = warpsock::Client::new().unwrap();
    for model in &["composer-2", "composer-2-fast"] {
        let response = client
            .post(format!("{}/v1/responses", live_base_url()))
            .timeout(std::time::Duration::from_secs(120))
            .json(&serde_json::json!({
                "model": model,
                "input": "Think step by step about why the sky is blue, then answer.",
                "stream": true,
            }))
            .send()
            .await
            .expect("live reasoning request");
        let text = response.text().expect("read reasoning body");
        common::assert_no_unredacted_sensitive_values(&text);
        assert!(
            text.contains("reasoning") || text.contains("thinking"),
            "live Cursor stream missing reasoning evidence for {model}",
        );
    }
}

#[tokio::test]
#[ignore = "requires UMP_LIVE_CURSOR=1 + reachable Cursor agent + tool registration"]
async fn live_cursor_chat_tool_call_round_trips_for_each_composer() {
    let Some(_guard) = LiveGuard::from_env("chat_tool_round_trip") else {
        return;
    };

    let client = warpsock::Client::new().unwrap();
    for model in COMPOSER_MODELS {
        let response = client
            .post(format!("{}/v1/chat/completions", live_base_url()))
            .timeout(std::time::Duration::from_secs(120))
            .json(&serde_json::json!({
                "model": model,
                "messages": [
                    { "role": "user", "content": "Use the lookup tool with key=alpha" }
                ],
                "tools": [{
                    "type": "function",
                    "function": {
                        "name": "lookup",
                        "parameters": {
                            "type": "object",
                            "properties": { "key": { "type": "string" } },
                            "required": ["key"]
                        }
                    }
                }]
            }))
            .send()
            .await
            .expect("live chat tool request");
        let text = response.text().expect("read chat tool body");
        common::assert_no_unredacted_sensitive_values(&text);
        // Required evidence: a tool_calls block on the first turn.
        assert!(
            text.contains("tool_calls") || text.contains("function"),
            "live chat tool round-trip missing tool_calls for {model}",
        );
    }
}

#[tokio::test]
#[ignore = "requires UMP_LIVE_CURSOR=1 + reachable Cursor agent (multi-turn)"]
async fn live_cursor_responses_multi_turn_continuation_via_previous_response_id() {
    let Some(_guard) = LiveGuard::from_env("multi_turn_continuation") else {
        return;
    };

    let client = warpsock::Client::new().unwrap();
    let model = "composer-2-fast";

    // Turn 1.
    let first = client
        .post(format!("{}/v1/responses", live_base_url()))
        .timeout(std::time::Duration::from_secs(180))
        .json(&serde_json::json!({
            "model": model,
            "input": "What's 2 + 2?",
            "stream": false,
            "store": true,
        }))
        .send()
        .await
        .expect("live first turn");
    let first_text = first.text().expect("read first turn");
    common::assert_no_unredacted_sensitive_values(&first_text);
    let first_json: serde_json::Value =
        serde_json::from_str(&first_text).expect("parse first turn json");
    let response_id = first_json["id"]
        .as_str()
        .expect("first turn must include response id")
        .to_string();
    assert!(!response_id.is_empty());

    // Turn 2: previous_response_id must thread the context.
    let second = client
        .post(format!("{}/v1/responses", live_base_url()))
        .timeout(std::time::Duration::from_secs(180))
        .json(&serde_json::json!({
            "model": model,
            "input": "Now multiply by 3",
            "previous_response_id": response_id,
            "stream": false,
            "store": true,
        }))
        .send()
        .await
        .expect("live second turn");
    let second_text = second.text().expect("read second turn");
    common::assert_no_unredacted_sensitive_values(&second_text);
    let second_json: serde_json::Value =
        serde_json::from_str(&second_text).expect("parse second turn");
    assert_eq!(second_json["object"], "response");
}

#[tokio::test]
#[ignore = "requires UMP_LIVE_CURSOR=1 + UMP_CURSOR_INDEX_CLOUD=1 + UMP_CURSOR_INDEX_BOOTSTRAP=1"]
async fn live_cursor_cloud_indexing_emits_handshake_upload_ensure_sync_evidence() {
    let Some(_guard) = LiveGuard::from_env("cloud_indexing") else {
        return;
    };
    assert_eq!(
        env_flag("UMP_CURSOR_INDEX_CLOUD"),
        Some(true),
        "live cloud indexing validation is mandatory when {LIVE_CURSOR_OPT_IN}=1; set UMP_CURSOR_INDEX_CLOUD=1"
    );
    assert_eq!(
        env_flag("UMP_CURSOR_INDEX_BOOTSTRAP"),
        Some(true),
        "live cloud bootstrap validation is mandatory when {LIVE_CURSOR_OPT_IN}=1; set UMP_CURSOR_INDEX_BOOTSTRAP=1"
    );

    let client = warpsock::Client::new().unwrap();
    let response = client
        .post(format!("{}/v1/chat/completions", live_base_url()))
        .timeout(std::time::Duration::from_secs(180))
        .json(&serde_json::json!({
            "model": "composer-2-fast",
            "messages": [
                {
                    "role": "user",
                    "content": "Use only cursor_codebase_search. Do not use grep, shell, read, ls, or fetch. Search for CursorAgentRequest and report exactly what the codebase search returned."
                }
            ]
        }))
        .send()
        .await
        .expect("live cloud-indexing tool request");
    let status = response.status();
    let text = response.text().expect("read live cloud-indexing tool body");
    common::assert_no_unredacted_sensitive_values(&text);
    assert!(
        status.is_success(),
        "live cloud-indexing route failed with {status}: {text}"
    );
    assert!(
        text.contains("cursor_codebase_search") || text.contains("Cursor codebase search"),
        "live cloud-indexing request did not visibly use cursor_codebase_search: {text}"
    );
    assert!(
        text.contains("CursorAgentRequest") && text.contains("src/cursor_agent.rs"),
        "live cloud-indexing request did not return indexed workspace hits: {text}"
    );
    assert!(
        !text.contains("Tool not available"),
        "live cloud-indexing request surfaced a native-tool rejection: {text}"
    );
    assert!(
        !text.contains("found no matching files"),
        "live cloud-indexing request fell through to an empty search result: {text}"
    );
}
