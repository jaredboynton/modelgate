use serde_json::{json, Value};

use crate::{upstream::windsurf::build_get_chat_message_request, AppResult};

pub const FAST_CONTEXT_MODEL: &str = "swe-1-6-fast";
pub const SWE_GREP_MODEL: &str = "swe-grep";
pub const SWE_GREP_MINI_MODEL: &str = "swe-grep-mini";
pub const FAST_CONTEXT_TOOL_NAME: &str = "cascade-find-code-context";
pub const FAST_CONTEXT_SENTINEL: &str = "<|context_request|>";

pub fn build_initial_request(query: &str) -> Value {
    json!({
        "model": FAST_CONTEXT_MODEL,
        "messages": [{
            "role": "user",
            "content": format!(
                "{FAST_CONTEXT_SENTINEL}\nUse {FAST_CONTEXT_TOOL_NAME} to find relevant code context.\nQuery: {query}"
            )
        }],
        "stream": false
    })
}

pub fn build_swe_grep_request(query: &str, repo_path: &str, model: &str) -> Value {
    json!({
        "model": model,
        "messages": [{
            "role": "user",
            "content": format!(
                "Fast Context repository search request.\nRepository root: {repo_path}\nQuery: {query}\nReturn the most relevant files, line ranges, and concise evidence snippets."
            )
        }],
        "stream": false
    })
}

pub fn build_followup_request(query: &str, tool_result: &str) -> Value {
    json!({
        "model": FAST_CONTEXT_MODEL,
        "messages": [
            {
                "role": "user",
                "content": format!(
                    "{FAST_CONTEXT_SENTINEL}\nUse {FAST_CONTEXT_TOOL_NAME} to find relevant code context.\nQuery: {query}"
                )
            },
            {
                "role": "tool",
                "tool_call_id": FAST_CONTEXT_TOOL_NAME,
                "content": tool_result
            }
        ],
        "stream": false
    })
}

pub fn build_initial_payload(query: &str, api_key: &str, version: &str) -> AppResult<Vec<u8>> {
    build_get_chat_message_request(
        &build_initial_request(query),
        api_key,
        version,
        FAST_CONTEXT_MODEL,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initial_request_contains_captured_fast_context_markers() {
        let request = build_initial_request("find windsurf assign model");
        let text = serde_json::to_string(&request).unwrap();

        assert!(text.contains(FAST_CONTEXT_MODEL));
        assert!(text.contains(FAST_CONTEXT_SENTINEL));
        assert!(text.contains(FAST_CONTEXT_TOOL_NAME));
    }

    #[test]
    fn swe_grep_request_targets_selected_model() {
        let request = build_swe_grep_request("find auth", "/tmp/repo", SWE_GREP_MINI_MODEL);
        let text = serde_json::to_string(&request).unwrap();

        assert!(text.contains(SWE_GREP_MINI_MODEL));
        assert!(text.contains("/tmp/repo"));
        assert!(text.contains("find auth"));
    }
}
