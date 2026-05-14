use axum::http::{HeaderMap, HeaderValue, Method};
use serde_json::json;
use unified_model_proxy_v2::route::websocket::{
    codex_bearer_public_route_policy, normalize_gpt_realtime_2_response_create,
    realtime_event_policy, realtime_headers_for_model, CodexBearerPublicRoutePolicy,
    RealtimeEventPolicy, RealtimeRoute,
};

#[test]
fn gpt_realtime_2_response_create_uses_ga_output_modalities() {
    let normalized = normalize_gpt_realtime_2_response_create(json!({
        "type": "response.create",
        "response": {
            "modalities": ["text"]
        }
    }))
    .unwrap();

    assert_eq!(
        normalized["response"]["output_modalities"],
        json!(["text"]),
        "old beta response.modalities should translate to the GA response.output_modalities field"
    );
    assert!(
        normalized["response"].get("modalities").is_none(),
        "gpt-realtime-2 requests must not send beta response.modalities upstream"
    );
}

#[test]
fn gpt_realtime_2_response_create_rejects_conflicting_beta_and_ga_modalities() {
    let err = normalize_gpt_realtime_2_response_create(json!({
        "type": "response.create",
        "response": {
            "modalities": ["audio"],
            "output_modalities": ["text"]
        }
    }))
    .unwrap_err();

    assert!(
        err.to_string().contains("response.modalities")
            && err.to_string().contains("response.output_modalities"),
        "conflicting beta and GA realtime modality fields should fail closed: {err}"
    );
}

#[test]
fn gpt_realtime_2_does_not_send_beta_realtime_header() {
    let mut headers = HeaderMap::new();
    headers.insert("OpenAI-Beta", HeaderValue::from_static("realtime=v1"));

    realtime_headers_for_model("gpt-realtime-2", &mut headers).unwrap();

    assert!(
        !headers.contains_key("OpenAI-Beta"),
        "gpt-realtime-2 GA must not carry OpenAI-Beta: realtime=v1"
    );
}

#[test]
fn realtime_route_accepts_ga_text_and_done_events() {
    for event in [
        json!({"type": "response.output_text.delta", "delta": "hel"}),
        json!({"type": "response.output_text.done", "text": "hello"}),
        json!({"type": "response.done", "response": {"id": "resp_rt"}}),
    ] {
        assert_eq!(
            realtime_event_policy(RealtimeRoute::Realtime, &event).unwrap(),
            RealtimeEventPolicy::Accept,
            "expected realtime route to accept {event}"
        );
    }
}

#[test]
fn responses_route_rejects_realtime_done_event() {
    let event = json!({"type": "response.done", "response": {"id": "resp_rt"}});
    assert_eq!(
        realtime_event_policy(RealtimeRoute::Responses, &event).unwrap(),
        RealtimeEventPolicy::Reject,
        "response.done belongs to realtime, not normal Responses SSE"
    );
}

#[test]
fn codex_bearer_public_route_policy_keeps_live_proven_auth_boundaries() {
    let cases = [
        (
            Method::GET,
            "/v1/models",
            CodexBearerPublicRoutePolicy::RouteAway,
        ),
        (
            Method::POST,
            "/v1/responses",
            CodexBearerPublicRoutePolicy::Reject,
        ),
        (
            Method::POST,
            "/v1/audio/speech",
            CodexBearerPublicRoutePolicy::Reject,
        ),
        (
            Method::POST,
            "/v1/realtime/transcription_sessions",
            CodexBearerPublicRoutePolicy::Allow,
        ),
    ];

    for (method, path, expected) in cases {
        assert_eq!(
            codex_bearer_public_route_policy(&method, path),
            expected,
            "{method} {path} Codex bearer policy drifted"
        );
    }
}
