mod common;

use std::fs;

use unified_model_proxy_v2::failure_capture::{
    failure_dir, failure_path, generate_request_id, list_failure_filenames, redact_failure_value,
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
