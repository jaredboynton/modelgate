use unified_model_proxy_v2::adapter::google_generate_content::{
    format_generate_content_response_for_caller, google_generate_content_sse_to_text,
    parse_google_generate_content_route, GoogleGenerateContentAction, GoogleGenerateContentCaller,
};

#[test]
fn parses_gemini_generate_content_routes() {
    let route =
        parse_google_generate_content_route("/v1beta/models/gemini-3.1-flash-lite:generateContent")
            .unwrap();

    assert_eq!(route.caller, GoogleGenerateContentCaller::Gemini);
    assert_eq!(route.action, GoogleGenerateContentAction::GenerateContent);
    assert_eq!(route.api_version, "v1beta");
    assert_eq!(route.model, "gemini-3.1-flash-lite");
    assert!(!route.stream());

    let route = parse_google_generate_content_route(
        "/v1/models/gemini-3.1-flash-lite:streamGenerateContent?alt=sse",
    )
    .unwrap();
    assert_eq!(
        route.action,
        GoogleGenerateContentAction::StreamGenerateContent
    );
    assert!(route.stream());
}

#[test]
fn parses_vertex_generate_content_routes() {
    let route = parse_google_generate_content_route(
        "/v1/projects/proj/locations/us-central1/publishers/google/models/gemini-3.1-flash-lite:streamGenerateContent",
    )
    .unwrap();

    assert_eq!(route.caller, GoogleGenerateContentCaller::Vertex);
    assert_eq!(route.api_version, "v1");
    assert_eq!(route.model, "gemini-3.1-flash-lite");
    assert_eq!(route.project.as_deref(), Some("proj"));
    assert_eq!(route.location.as_deref(), Some("us-central1"));
    assert!(route.stream());
}

#[test]
fn rejects_unsafe_google_route_segments() {
    for path in [
        "/v1beta/models/gemini%2Fbad:generateContent",
        "/v1beta/models/../gemini:generateContent",
        "/v1beta/models/:generateContent",
        "/v1beta/models/gemini:unknownAction",
        "/v1/projects/proj/locations/us/publishers/other/models/gemini:generateContent",
    ] {
        assert!(parse_google_generate_content_route(path).is_err(), "{path}");
    }
}

#[test]
fn formats_gemini_generate_content_without_vertex_fields() {
    let shaped = format_generate_content_response_for_caller(
        serde_json::json!({
            "candidates": [],
            "usageMetadata": { "totalTokenCount": 1 },
            "modelVersion": "gemini-test",
            "modelStatus": { "healthy": true },
            "createTime": "2026-05-13T00:00:00Z"
        }),
        GoogleGenerateContentCaller::Gemini,
    )
    .unwrap();

    assert!(shaped.get("modelStatus").is_some());
    assert!(shaped.get("createTime").is_none());
}

#[test]
fn formats_vertex_generate_content_without_gemini_fields() {
    let shaped = format_generate_content_response_for_caller(
        serde_json::json!({
            "candidates": [],
            "usageMetadata": { "totalTokenCount": 1 },
            "modelVersion": "gemini-test",
            "modelStatus": { "healthy": true },
            "createTime": "2026-05-13T00:00:00Z"
        }),
        GoogleGenerateContentCaller::Vertex,
    )
    .unwrap();

    assert!(shaped.get("createTime").is_some());
    assert!(shaped.get("modelStatus").is_none());
}

#[test]
fn formats_direct_google_sse_as_data_only_chunks() {
    let input = r#"data: {"candidates":[{"content":{"parts":[{"text":"hi"}]},"finishReason":"STOP"}],"modelStatus":{"healthy":true},"createTime":"drop"}"#;
    let sse = google_generate_content_sse_to_text(
        &format!("{input}\n\n"),
        GoogleGenerateContentCaller::Gemini,
    )
    .unwrap();

    assert!(sse.starts_with("data: "));
    assert!(!sse.contains("event:"));
    assert!(sse.contains("\"modelStatus\""));
    assert!(!sse.contains("\"createTime\""));
    assert!(!sse.contains("[DONE]"));
}
