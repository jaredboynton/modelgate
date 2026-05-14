use unified_model_proxy_v2::{
    adapter::google_responses::{
        google_generate_content_sse_to_responses_sse_text, google_generate_content_to_responses,
        is_google_responses_stream_request, responses_to_google_generate_content,
    },
    AppError,
};

#[test]
fn google_responses_string_input_maps_to_generate_content() {
    let google = responses_to_google_generate_content(
        serde_json::json!({
            "model": "gemini-3.1-flash-lite",
            "instructions": "Be concise.",
            "input": "hello",
            "max_output_tokens": 32,
            "temperature": 0.2,
            "top_p": 0.9,
            "stop": ["DONE"]
        }),
        "gemini-3.1-flash-lite",
    )
    .unwrap();

    assert_eq!(google["contents"][0]["role"], "user");
    assert_eq!(google["contents"][0]["parts"][0]["text"], "hello");
    assert_eq!(
        google["systemInstruction"]["parts"][0]["text"],
        "Be concise."
    );
    assert_eq!(google["generationConfig"]["maxOutputTokens"], 32);
    assert_eq!(google["generationConfig"]["temperature"], 0.2);
    assert_eq!(google["generationConfig"]["topP"], 0.9);
    assert_eq!(google["generationConfig"]["stopSequences"][0], "DONE");
}

#[test]
fn google_responses_message_array_maps_text_roles() {
    let google = responses_to_google_generate_content(
        serde_json::json!({
            "model": "gemini-3.1-flash-lite",
            "input": [
                {
                    "type": "message",
                    "role": "developer",
                    "content": [{ "type": "input_text", "text": "Follow policy." }]
                },
                {
                    "type": "message",
                    "role": "user",
                    "content": [{ "type": "input_text", "text": "ping" }]
                },
                {
                    "type": "message",
                    "role": "assistant",
                    "content": [{ "type": "output_text", "text": "pong" }]
                }
            ]
        }),
        "gemini-3.1-flash-lite",
    )
    .unwrap();

    assert_eq!(
        google["systemInstruction"]["parts"][0]["text"],
        "Follow policy."
    );
    assert_eq!(google["contents"][0]["role"], "user");
    assert_eq!(google["contents"][0]["parts"][0]["text"], "ping");
    assert_eq!(google["contents"][1]["role"], "model");
    assert_eq!(google["contents"][1]["parts"][0]["text"], "pong");
}

#[test]
fn google_responses_rejects_unsupported_semantics() {
    assert!(matches!(
        responses_to_google_generate_content(
            serde_json::json!({
                "model": "gemini-3.1-flash-lite",
                "input": [{ "type": "function_call", "call_id": "call_1" }]
            }),
            "gemini-3.1-flash-lite"
        )
        .unwrap_err(),
        AppError::BadRequest(_)
    ));
}

#[test]
fn google_responses_hosted_tools_are_ignored() {
    let google = responses_to_google_generate_content(
        serde_json::json!({
            "model": "gemini-3.1-flash-lite",
            "input": "hello",
            "tools": [
                { "type": "local_shell" },
                { "type": "web_search_preview" },
                {
                    "type": "function",
                    "name": "lookup",
                    "parameters": { "type": "object" }
                }
            ]
        }),
        "gemini-3.1-flash-lite",
    )
    .unwrap();

    let declarations = google["tools"][0]["functionDeclarations"]
        .as_array()
        .unwrap();
    assert_eq!(declarations.len(), 1);
    assert_eq!(declarations[0]["name"], "lookup");
}

#[test]
fn google_responses_hosted_only_tools_do_not_emit_google_tools() {
    let google = responses_to_google_generate_content(
        serde_json::json!({
            "model": "gemini-3.1-flash-lite",
            "input": "hello",
            "tool_choice": "auto",
            "tools": [
                { "type": "local_shell" },
                { "type": "web_search" }
            ]
        }),
        "gemini-3.1-flash-lite",
    )
    .unwrap();

    assert!(google.get("tools").is_none());
    assert!(google.get("toolConfig").is_none());
}

#[test]
fn google_responses_custom_and_freeform_tools_map_to_input_function() {
    let google = responses_to_google_generate_content(
        serde_json::json!({
            "model": "gemini-3.1-flash-lite",
            "input": "hello",
            "tools": [
                { "type": "freeform", "name": "apply_patch", "description": "Apply a patch" },
                { "type": "custom", "name": "draft_note" }
            ]
        }),
        "gemini-3.1-flash-lite",
    )
    .unwrap();

    let declarations = google["tools"][0]["functionDeclarations"]
        .as_array()
        .unwrap();
    assert_eq!(declarations[0]["name"], "apply_patch");
    assert_eq!(
        declarations[0]["parameters"]["properties"]["input"]["type"],
        "string"
    );
    assert_eq!(declarations[1]["name"], "draft_note");
    assert_eq!(declarations[1]["parameters"]["required"][0], "input");
}

#[test]
fn google_responses_forced_filtered_tool_choice_is_rejected() {
    let error = responses_to_google_generate_content(
        serde_json::json!({
            "model": "gemini-3.1-flash-lite",
            "input": "hello",
            "tool_choice": { "type": "function", "name": "local_shell" },
            "tools": [{ "type": "local_shell" }]
        }),
        "gemini-3.1-flash-lite",
    )
    .unwrap_err();

    assert!(error.to_string().contains("tool_choice function"));
}

#[test]
fn google_responses_function_parameters_are_sanitized_for_gemini_schema_subset() {
    let google = responses_to_google_generate_content(
        serde_json::json!({
            "model": "gemini-3.1-flash-lite",
            "input": "hello",
            "tools": [{
                "type": "function",
                "name": "edit",
                "parameters": {
                    "$schema": "https://json-schema.org/draft/2020-12/schema",
                    "type": "object",
                    "additionalProperties": false,
                    "properties": {
                        "path": {
                            "type": ["string", "null"],
                            "description": "File path",
                            "additionalProperties": false
                        },
                        "edits": {
                            "type": "array",
                            "items": {
                                "type": "object",
                                "additionalProperties": false,
                                "properties": {
                                    "old": { "type": "string" },
                                    "new": { "anyOf": [{ "type": "string" }, { "type": "null" }] }
                                },
                                "required": ["old"]
                            }
                        }
                    },
                    "required": ["path", "edits"]
                }
            }]
        }),
        "gemini-3.1-flash-lite",
    )
    .unwrap();

    let parameters = &google["tools"][0]["functionDeclarations"][0]["parameters"];
    let serialized = serde_json::to_string(parameters).unwrap();
    assert!(!serialized.contains("additionalProperties"));
    assert!(!serialized.contains("$schema"));
    assert_eq!(parameters["properties"]["path"]["nullable"], true);
    assert_eq!(
        parameters["properties"]["edits"]["items"]["properties"]["new"]["nullable"],
        true
    );
}

#[test]
fn google_responses_reasoning_effort_maps_to_thinking_config() {
    let flash = responses_to_google_generate_content(
        serde_json::json!({
            "model": "gemini-3.1-flash-lite",
            "input": "hello",
            "reasoning": { "effort": "none", "summary": "auto" }
        }),
        "gemini-3.1-flash-lite",
    )
    .unwrap();
    assert_eq!(
        flash["generationConfig"]["thinkingConfig"]["thinkingLevel"],
        "minimal"
    );

    let pro = responses_to_google_generate_content(
        serde_json::json!({
            "model": "gemini-3.1-pro-preview",
            "input": "hello",
            "reasoning": { "effort": "xhigh" }
        }),
        "gemini-3.1-pro-preview",
    )
    .unwrap();
    assert_eq!(
        pro["generationConfig"]["thinkingConfig"]["thinkingLevel"],
        "high"
    );

    let legacy_budget = responses_to_google_generate_content(
        serde_json::json!({
            "model": "gemini-2.5-flash",
            "input": "hello",
            "reasoning": { "effort": "low" }
        }),
        "gemini-2.5-flash",
    )
    .unwrap();
    assert_eq!(
        legacy_budget["generationConfig"]["thinkingConfig"]["thinkingBudget"],
        1024
    );
}

#[test]
fn google_responses_stream_true_text_input_maps_to_generate_content() {
    let request = serde_json::json!({
        "model": "gemini-3.1-flash-lite",
        "input": "hello",
        "stream": true
    });
    assert!(is_google_responses_stream_request(&request));

    let google = responses_to_google_generate_content(request, "gemini-3.1-flash-lite").unwrap();
    assert_eq!(google["contents"][0]["role"], "user");
    assert_eq!(google["contents"][0]["parts"][0]["text"], "hello");
    assert!(google.get("stream").is_none());
}

#[test]
fn google_generate_content_response_maps_to_responses() {
    let responses = google_generate_content_to_responses(
        serde_json::json!({
            "candidates": [{
                "content": {
                    "role": "model",
                    "parts": [{ "text": "ok" }]
                },
                "finishReason": "STOP"
            }],
            "usageMetadata": {
                "promptTokenCount": 3,
                "candidatesTokenCount": 2,
                "totalTokenCount": 5
            }
        }),
        "gemini-3.1-flash-lite",
    )
    .unwrap();

    assert_eq!(responses["object"], "response");
    assert_eq!(responses["model"], "gemini-3.1-flash-lite");
    assert_eq!(responses["status"], "completed");
    assert_eq!(responses["output"][0]["content"][0]["text"], "ok");
    assert_eq!(responses["usage"]["input_tokens"], 3);
    assert_eq!(responses["usage"]["output_tokens"], 2);
    assert_eq!(responses["usage"]["total_tokens"], 5);
}

#[test]
fn google_generate_content_sse_maps_to_responses_sse() {
    let sse = google_generate_content_sse_to_responses_sse_text(
        r#"data: {"candidates":[{"content":{"role":"model","parts":[{"text":"ok"}]},"finishReason":"STOP"}],"usageMetadata":{"promptTokenCount":3,"candidatesTokenCount":2,"totalTokenCount":5}}"#,
        "gemini-3.1-flash-lite",
    )
    .unwrap();

    assert!(sse.contains("event: response.created"));
    assert!(sse.contains("event: response.output_item.added"));
    assert!(sse.contains("event: response.output_text.delta"));
    assert!(sse.contains("event: response.output_item.done"));
    assert!(sse.contains("event: response.completed"));
    assert!(sse.contains("\"delta\":\"ok\""));
    assert!(sse.contains("\"model\":\"gemini-3.1-flash-lite\""));
    assert!(sse.contains("\"input_tokens\":3"));
    assert!(!sse.contains("\"candidates\""));
}

#[test]
fn google_generate_content_response_without_text_is_upstream_error() {
    assert!(matches!(
        google_generate_content_to_responses(
            serde_json::json!({
                "candidates": [{
                    "content": { "role": "model", "parts": [] },
                    "finishReason": "STOP"
                }]
            }),
            "gemini-3.1-flash-lite",
        )
        .unwrap_err(),
        AppError::Upstream(_)
    ));
}
