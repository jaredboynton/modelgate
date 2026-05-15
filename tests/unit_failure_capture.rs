mod common;

use std::fs;

use unified_model_proxy_v2::failure_capture::{
    failure_dir, failure_path, generate_request_id, list_failure_filenames, redact_failure_value,
    write_failure_json,
};

#[test]
fn generates_uuid_request_ids() {
    let first = generate_request_id();
    let second = generate_request_id();

    assert_ne!(first, second);
    assert_eq!(first.len(), 36);
    assert_eq!(first.chars().filter(|ch| *ch == '-').count(), 4);
}

#[test]
fn builds_failure_paths_under_auth_home() {
    let homes = common::TestHomes::new();
    let path = failure_path(&homes.state, "req-123", "google:generateContent").unwrap();

    assert_eq!(
        failure_dir(&homes.state),
        homes.state.auth_home.join("v2-failures")
    );
    assert!(path.starts_with(homes.state.auth_home.join("v2-failures")));
    assert_eq!(
        path.file_name().and_then(|name| name.to_str()),
        Some("req-123-google-generateContent.json")
    );
}

#[test]
fn redacts_sensitive_headers_tokens_and_base64_image_payloads() {
    let mut value = serde_json::json!({
        "headers": {
            "Authorization": "Bearer secret",
            "x-api-key": "api-secret",
            "x-goog-api-key": "goog-secret",
            "cookie": "session=secret",
            "content-type": "application/json"
        },
        "refresh_token": "refresh-secret",
        "input": [{
            "image": {"b64_json": "base64-secret"},
            "inline_data": {"mime_type": "image/png", "data": "inline-secret"}
        }]
    });

    redact_failure_value(&mut value);

    assert_eq!(value["headers"]["Authorization"], "[redacted]");
    assert_eq!(value["headers"]["x-api-key"], "[redacted]");
    assert_eq!(value["headers"]["x-goog-api-key"], "[redacted]");
    assert_eq!(value["headers"]["cookie"], "[redacted]");
    assert_eq!(value["headers"]["content-type"], "application/json");
    assert_eq!(value["refresh_token"], "[redacted]");
    assert_eq!(value["input"][0]["image"]["b64_json"], "[redacted]");
    assert_eq!(value["input"][0]["inline_data"]["data"], "[redacted]");
}

#[test]
fn serialized_failure_json_does_not_leak_live_sensitive_payload_classes() {
    let homes = common::TestHomes::new();
    let leaked_values = [
        "fake-access-token-lane-5",
        "fake-refresh-token-lane-5",
        "fake-id-token-lane-5",
        "Bearer fake-bearer-token-lane-5",
        "session=fake-cookie-lane-5",
        "fake-api-key-lane-5",
        "fake-client-secret-lane-5",
        "acct_fake_chatgpt_lane_5",
        "secret_query_lane_5",
        "v=0\r\no=- 46117326 2 IN IP4 127.0.0.1\r\ns=fake-sdp-offer-lane-5",
        "fake-audio-bytes-lane-5",
        "fake multipart body lane 5",
        "ZmFrZS1iYXNlNjQtcGF5bG9hZC1sYW5lLTU=",
        "fake transcript text lane 5",
        "fake transcript delta lane 5",
    ];
    let path = write_failure_json(
        &homes.state,
        "req-redaction",
        "codex-realtime",
        serde_json::json!({
            "headers": {
                "Authorization": "Bearer fake-bearer-token-lane-5",
                "cookie": "session=fake-cookie-lane-5",
                "x-api-key": "fake-api-key-lane-5",
                "ChatGPT-Account-Id": "acct_fake_chatgpt_lane_5"
            },
            "tokens": {
                "access_token": "fake-access-token-lane-5",
                "refresh_token": "fake-refresh-token-lane-5",
                "id_token": "fake-id-token-lane-5",
                "client_secret": "fake-client-secret-lane-5",
                "bearer": "Bearer fake-bearer-token-lane-5"
            },
            "account_id": "acct_fake_chatgpt_lane_5",
            "urls": [
                "wss://chatgpt.com/backend-api/codex/responses?access_token=secret_query_lane_5",
                "https://api.openai.com/v1/realtime?api_key=secret_query_lane_5"
            ],
            "realtime": {
                "offer": "v=0\r\no=- 46117326 2 IN IP4 127.0.0.1\r\ns=fake-sdp-offer-lane-5",
                "answer_sdp": "v=0\r\ns=fake-sdp-answer-lane-5",
                "audio_bytes": "fake-audio-bytes-lane-5",
                "input_audio_buffer": {"audio": "fake-audio-bytes-lane-5"}
            },
            "multipart_body": "fake multipart body lane 5",
            "base64_payload": "ZmFrZS1iYXNlNjQtcGF5bG9hZC1sYW5lLTU=",
            "transcript": {
                "text": "fake transcript text lane 5",
                "delta": "fake transcript delta lane 5"
            }
        }),
    )
    .unwrap();

    let serialized = fs::read_to_string(path).unwrap();
    for leaked_value in leaked_values {
        assert!(
            !serialized.contains(leaked_value),
            "serialized failure JSON leaked {leaked_value:?}: {serialized}"
        );
    }
}

#[test]
fn compact_failure_capture_keeps_policy_but_redacts_prompt_and_pack_contents() {
    let homes = common::TestHomes::new();
    let raw_prompt = "raw compact prompt lane 4 must not leak";
    let raw_pack = "ump.compaction.v1.protected.nonce.ciphertext-lane-4";
    let path = write_failure_json(
        &homes.state,
        "req-compact-redaction",
        "responses-compact",
        serde_json::json!({
            "provider": "bedrock",
            "model": "anthropic.claude-opus-4-7",
            "compaction_policy": "proxy_visible_summary",
            "input": [{
                "type": "message",
                "role": "user",
                "content": [{ "type": "input_text", "text": raw_prompt }]
            }],
            "output": [{
                "type": "compaction",
                "encrypted_content": raw_pack
            }]
        }),
    )
    .unwrap();

    let serialized = fs::read_to_string(path).unwrap();
    assert!(serialized.contains(r#""compaction_policy": "proxy_visible_summary""#));
    assert!(
        !serialized.contains(raw_prompt),
        "compact logs must not include raw source prompt: {serialized}"
    );
    assert!(
        !serialized.contains(raw_pack),
        "compact logs must not include encrypted pack contents: {serialized}"
    );
}

#[test]
fn common_live_assertions_catch_fake_sensitive_sentinels() {
    let output = "status=500 fake-access-token-lane-5";

    assert!(
        std::panic::catch_unwind(|| common::assert_no_unredacted_sensitive_values(output)).is_err()
    );
    assert_eq!(
        common::redact_sensitive_values(output),
        "status=500 [REDACTED]"
    );
}

#[test]
fn lists_failure_filenames_with_cap() {
    let homes = common::TestHomes::new();
    let dir = failure_dir(&homes.state);
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("b.json"), "{}").unwrap();
    fs::write(dir.join("a.json"), "{}").unwrap();
    fs::write(dir.join("c.txt"), "skip").unwrap();
    fs::create_dir(dir.join("nested.json")).unwrap();

    assert_eq!(
        list_failure_filenames(&homes.state, 1).unwrap(),
        vec!["a.json".to_string()]
    );
    assert_eq!(
        list_failure_filenames(&homes.state, 10).unwrap(),
        vec!["a.json".to_string(), "b.json".to_string()]
    );
}

#[test]
#[should_panic(expected = "test home must live under temp dir")]
fn test_harness_refuses_real_home_paths() {
    let realish_home = dirs::home_dir().unwrap_or_else(|| "/Users/example".into());
    let _ = common::temp_state(realish_home.join(".codex"), realish_home.join(".ump"));
}
