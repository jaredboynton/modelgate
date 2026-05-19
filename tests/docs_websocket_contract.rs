use std::fs;

fn read_doc(path: &str) -> String {
    fs::read_to_string(format!("{}/{}", env!("CARGO_MANIFEST_DIR"), path))
        .unwrap_or_else(|error| panic!("failed to read {path}: {error}"))
}

#[test]
fn websocket_docs_describe_mixed_downstream_contract() {
    let readme = read_doc("README.md");
    let websocket = read_doc("docs/golden-principles/WEBSOCKET_RESPONSES.md");
    let provider = read_doc("docs/golden-principles/PROVIDER_BOUNDARIES.md");
    let combined = format!("{readme}\n{websocket}\n{provider}");

    for required in [
        "mixed-provider facade",
        "post-terminal",
        "response_already_in_flight",
        "previous_response_id` is connection-local",
        "must exactly match the prior route/model fingerprint",
        "Codex WSS, Bedrock HTTP/SSE, and Google HTTP/SSE calls keep separate auth sources",
    ] {
        assert!(
            combined.contains(required),
            "WebSocket docs must mention `{required}`"
        );
    }

    for stale in [
        "The downstream socket binds to the first route/model on that connection",
        "One socket owns one route/model fingerprint",
        "Non-Codex Responses routes must reject instead of tunneling to Codex",
        "currently describe that policy as intended",
        "Update or add these tests before changing implementation",
        "## Implementation Plan",
        "The fix is complete when",
        "Add coverage proving",
        "Update docs and planning notes",
    ] {
        assert!(
            !combined.contains(stale),
            "WebSocket docs still contain stale contract text: `{stale}`"
        );
    }
}
