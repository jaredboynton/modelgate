mod common;

use http::StatusCode;
use serde_json::Value;

#[tokio::test]
#[ignore = "requires a locally running unified-model-proxy-v2 server"]
async fn live_health_route_reports_ok() {
    let client = specter::Client::new().unwrap();
    let response = client
        .get(format!("{}/health", common::live_base_url()))
        .send()
        .await
        .expect("send live health request");

    assert_eq!(response.status(), StatusCode::OK);
    let body: Value = response.json().expect("parse health response");
    assert_eq!(body["status"], "ok");
    assert_eq!(body["version"], env!("CARGO_PKG_VERSION"));
    assert!(body["git_revision"]
        .as_str()
        .is_some_and(|value| !value.is_empty()));
    assert!(body["build_time_utc"]
        .as_str()
        .is_some_and(|value| !value.is_empty()));
}

#[tokio::test]
#[ignore = "requires a locally running unified-model-proxy-v2 server"]
async fn live_models_route_returns_at_least_one_model_without_leaking_known_env_secrets() {
    let client = specter::Client::new().unwrap();
    let response = client
        .get(format!("{}/v1/models", common::live_base_url()))
        .send()
        .await
        .expect("send live models request");

    assert_eq!(response.status(), StatusCode::OK);
    let text = response.text().expect("read models response");
    common::assert_no_unredacted_sensitive_values(&text);
    let body: Value = serde_json::from_str(&text).expect("parse models response");
    assert!(body["data"]
        .as_array()
        .is_some_and(|models| !models.is_empty()));
}

#[tokio::test]
#[ignore = "requires live provider credentials and UMP_V2_LIVE_CHAT_MODEL"]
async fn live_chat_completion_returns_success_when_opted_in() {
    let Some(model) = common::optional_env("UMP_V2_LIVE_CHAT_MODEL") else {
        eprintln!("skipping live chat completion: UMP_V2_LIVE_CHAT_MODEL is not set");
        return;
    };

    let client = specter::Client::new().unwrap();
    let response = client
        .post(format!("{}/v1/chat/completions", common::live_base_url()))
        .json(&serde_json::json!({
            "model": model,
            "messages": [{ "role": "user", "content": "Reply with exactly: ok" }],
            "max_tokens": 16
        }))
        .send()
        .await
        .expect("send live chat completion request");

    let status = response.status();
    let text = response.text().expect("read chat completion response");
    common::assert_no_unredacted_sensitive_values(&text);
    assert!(
        status.is_success(),
        "live chat completion failed with {status}: {}",
        common::redact_sensitive_values(&text)
    );
}
