use unified_model_proxy_v2::{
    rate_limit::parse_codex_ws_protocol, upstream::codex, AppError, AppState,
};

#[test]
fn unit_codex_protocol_defaults_to_rfc6455_only() {
    assert!(parse_codex_ws_protocol(None).is_ok());
    assert!(parse_codex_ws_protocol(Some("rfc6455")).is_ok());
    assert!(parse_codex_ws_protocol(Some("rfc8441")).is_err());
    assert!(parse_codex_ws_protocol(Some("rfc9220")).is_err());
}

#[test]
fn unit_codex_body_gets_required_defaults() {
    let body = serde_json::json!({
        "model": "openai:gpt-5.5",
        "include": "not-an-array"
    });
    let prepared = codex::prepare_responses_body(body).unwrap();
    assert_eq!(prepared["model"], "gpt-5.5");
    assert_eq!(prepared["stream"], true);
    assert_eq!(prepared["store"], false);
    assert_eq!(prepared["instructions"], "You are a helpful assistant.");
    assert_eq!(prepared["reasoning"]["summary"], "auto");
    assert!(prepared["include"]
        .as_array()
        .unwrap()
        .iter()
        .any(|value| value == "reasoning.encrypted_content"));
}

#[test]
fn unit_codex_preserves_list_shaped_input_upstream() {
    let body = serde_json::json!({
        "model": "openai/gpt-5.4",
        "input": [{
            "role": "user",
            "content": [{
                "type": "input_text",
                "text": "hello"
            }]
        }]
    });

    let prepared = codex::prepare_responses_body(body).unwrap();
    assert_eq!(prepared["input"][0]["role"], "user");
    assert_eq!(prepared["input"][0]["content"][0]["type"], "input_text");
    assert_eq!(prepared["input"][0]["content"][0]["text"], "hello");
}

#[test]
fn unit_codex_normalizes_string_input_to_list_message() {
    let body = serde_json::json!({
        "model": "openai/gpt-5.4",
        "input": "hello from a public Responses client"
    });

    let prepared = codex::prepare_responses_body(body).unwrap();
    assert_eq!(
        prepared["input"],
        serde_json::json!([{
            "role": "user",
            "content": [{
                "type": "input_text",
                "text": "hello from a public Responses client"
            }]
        }])
    );
}

#[test]
fn unit_codex_keeps_prompt_cache_key_but_strips_retention_hint() {
    let body = serde_json::json!({
        "model": "openai:gpt-5.5",
        "input": "hello",
        "prompt_cache_key": "factory-droid",
        "prompt_cache_retention": "24h"
    });

    let prepared = codex::prepare_responses_body(body).unwrap();
    assert!(prepared.get("prompt_cache_retention").is_none());
    assert_eq!(prepared["prompt_cache_key"], "factory-droid");
}

#[test]
fn unit_codex_response_create_payload_is_flat_http_body() {
    let body = serde_json::json!({
        "model": "openai/gpt-5.4",
        "reasoning": { "effort": "high" },
        "include": ["file_search_call.results"]
    });

    let payload = codex::prepare_response_create_payload(body).unwrap();
    assert_eq!(payload["model"], "gpt-5.4");
    assert_eq!(payload["stream"], true);
    assert_eq!(payload["store"], false);
    assert!(payload.get("type").is_none());
    assert!(payload.get("response").is_none());
    assert_eq!(payload["reasoning"]["effort"], "high");
    assert_eq!(payload["reasoning"]["summary"], "auto");
    let include = payload["include"].as_array().unwrap();
    assert!(include
        .iter()
        .any(|value| value == "file_search_call.results"));
    assert!(include
        .iter()
        .any(|value| value == "reasoning.encrypted_content"));
}

#[test]
fn unit_codex_response_create_event_payload_uses_flat_ws_shape() {
    let body = serde_json::json!({
        "type": "response.create",
        "model": "openai:gpt-5.5",
        "stream": false,
        "input": "hello",
        "safety_identifier": "droid-user",
        "temperature": 0.4
    });

    let payload = codex::prepare_response_create_event_payload_with_resolver(
        body,
        unified_model_proxy_v2::model_alias::resolve_model_required,
    )
    .unwrap();

    assert_eq!(payload["type"], "response.create");
    assert_eq!(payload["model"], "gpt-5.5");
    assert_eq!(payload["stream"], true);
    assert_eq!(payload["store"], false);
    assert!(payload.get("response").is_none());
    assert!(payload.get("safety_identifier").is_none());
    assert!(payload.get("temperature").is_none());
    assert_eq!(payload["input"][0]["content"][0]["text"], "hello");
}

#[test]
fn unit_codex_nested_response_create_payload_is_explicit_compatibility_shape() {
    let body = serde_json::json!({
        "model": "openai:gpt-5.5",
        "input": "hello"
    });

    let payload = codex::prepare_nested_response_create_payload_with_resolver(
        body,
        unified_model_proxy_v2::model_alias::resolve_model_required,
    )
    .unwrap();

    assert_eq!(payload["type"], "response.create");
    assert_eq!(payload["response"]["model"], "gpt-5.5");
    assert_eq!(payload["response"]["stream"], true);
    assert_eq!(payload["response"]["store"], false);
    assert_eq!(
        payload["response"]["input"][0]["content"][0]["text"],
        "hello"
    );
}

#[test]
fn unit_codex_strips_fields_outside_backend_allowlist() {
    for (field, value) in [
        ("background", serde_json::json!(true)),
        (
            "context_management",
            serde_json::json!({ "strategy": "auto" }),
        ),
        ("conversation", serde_json::json!({ "id": "conv_123" })),
        ("max_tool_calls", serde_json::json!(2)),
        ("max_output_tokens", serde_json::json!(100)),
        ("max_tokens", serde_json::json!(100)),
        ("prompt", serde_json::json!({ "id": "pmpt_123" })),
        ("safety_identifier", serde_json::json!("safe-user")),
        (
            "stream_options",
            serde_json::json!({ "include_obfuscation": true }),
        ),
        ("temperature", serde_json::json!(0.4)),
        ("top_logprobs", serde_json::json!(1)),
        ("top_p", serde_json::json!(0.9)),
        ("truncation", serde_json::json!("auto")),
        ("unknown_future_field", serde_json::json!("future")),
    ] {
        let mut body = serde_json::json!({ "model": "openai:gpt-5.5" });
        body[field] = value;
        let prepared = codex::prepare_responses_body(body).unwrap();
        assert!(
            prepared.get(field).is_none(),
            "{field} should not be forwarded: {prepared}"
        );
    }

    let prepared = codex::prepare_responses_body(serde_json::json!({
        "model": "openai:gpt-5.5",
        "store": true
    }))
    .unwrap();
    assert_eq!(prepared["store"], false);
}

#[test]
fn unit_codex_preserves_supported_request_controls() {
    let body = serde_json::json!({
        "model": "openai:gpt-5.5",
        "input": "hello",
        "client_metadata": { "client": "droid" },
        "generate": false,
        "include": ["file_search_call.results"],
        "parallel_tool_calls": true,
        "prompt_cache_key": "cache-key",
        "service_tier": "default",
        "text": { "format": { "type": "json_object" } },
        "tool_choice": "auto",
        "tools": [{ "type": "function", "name": "lookup" }]
    });

    let prepared = codex::prepare_responses_body(body).unwrap();
    assert_eq!(prepared["client_metadata"]["client"], "droid");
    assert_eq!(prepared["generate"], false);
    assert!(prepared["include"]
        .as_array()
        .unwrap()
        .iter()
        .any(|value| value == "file_search_call.results"));
    assert_eq!(prepared["parallel_tool_calls"], true);
    assert_eq!(prepared["prompt_cache_key"], "cache-key");
    assert_eq!(prepared["service_tier"], "default");
    assert_eq!(prepared["text"]["format"]["type"], "json_object");
    assert_eq!(prepared["tool_choice"], "auto");
    assert_eq!(prepared["tools"][0]["name"], "lookup");
}

#[test]
fn unit_codex_normalizes_unsupported_service_tier() {
    let prepared = codex::prepare_responses_body(serde_json::json!({
        "model": "openai:gpt-5.5",
        "input": "hello",
        "service_tier": "auto"
    }))
    .unwrap();

    assert_eq!(prepared["service_tier"], "priority");
}

#[test]
fn unit_codex_rejects_public_file_audio_and_unmapped_tools() {
    for (case, patch, expected) in [
        (
            "audio input",
            serde_json::json!({ "input": [{ "role": "user", "content": [{ "type": "input_audio" }] }] }),
            "input_audio",
        ),
        (
            "file input",
            serde_json::json!({ "input": [{ "role": "user", "content": [{ "type": "input_file" }] }] }),
            "input_file",
        ),
        (
            "local image path",
            serde_json::json!({ "input": [{ "role": "user", "content": [{ "type": "localImage", "path": "/tmp/image.png" }] }] }),
            "localImage",
        ),
        (
            "image file id",
            serde_json::json!({ "input": [{ "role": "user", "content": [{ "type": "input_image", "file_id": "file_123" }] }] }),
            "input_image.file_id",
        ),
        (
            "apply patch tool",
            serde_json::json!({ "tools": [{ "type": "apply_patch" }] }),
            "apply_patch",
        ),
        (
            "hosted file search",
            serde_json::json!({ "tools": [{ "type": "file_search" }] }),
            "file_search",
        ),
        (
            "computer tool",
            serde_json::json!({ "tools": [{ "type": "computer" }] }),
            "computer",
        ),
    ] {
        let mut body = serde_json::json!({ "model": "openai:gpt-5.5" });
        body.as_object_mut()
            .unwrap()
            .extend(patch.as_object().unwrap().clone());
        let error = codex::prepare_responses_body(body).unwrap_err();
        assert!(
            matches!(&error, AppError::BadRequest(message) if message.contains(expected)),
            "unexpected error for {case}: {error}"
        );
    }
}

#[test]
fn unit_codex_headers_use_codex_originator() {
    let temp = tempfile::tempdir().unwrap();
    std::fs::write(
        temp.path().join("auth.json"),
        r#"{
            "tokens": {
                "access_token": "access-token",
                "account_id": "account-123"
            }
        }"#,
    )
    .unwrap();
    let auth_home = tempfile::tempdir().unwrap();
    let state = AppState::for_tests(temp.path().to_path_buf(), auth_home.path().to_path_buf());

    let headers = codex::codex_headers(&state).unwrap();
    assert_eq!(headers["originator"], "codex_cli_rs");
    assert_eq!(headers["OpenAI-Beta"], "responses_websockets=2026-02-06");
    assert_eq!(headers["ChatGPT-Account-Id"], "account-123");
}
