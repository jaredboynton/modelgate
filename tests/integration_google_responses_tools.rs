use serde_json::Value;
use unified_model_proxy_v2::adapter::google_responses::{
    google_generate_content_sse_to_responses_sse_text, google_generate_content_to_responses,
    responses_to_google_generate_content, responses_to_google_generate_content_with_context,
    GoogleResponsesSseTranslator,
};

#[test]
fn google_responses_tools_and_tool_choice_map_to_gemini_function_declarations() {
    let google = responses_to_google_generate_content(
        serde_json::json!({
            "model": "gemini-3.1-flash-lite",
            "input": "look up amp",
            "parallel_tool_calls": true,
            "tool_choice": { "type": "function", "name": "lookup" },
            "tools": [{
                "type": "function",
                "name": "lookup",
                "description": "Look up a term",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "query": { "type": "string" }
                    },
                    "required": ["query"]
                }
            }]
        }),
        "gemini-3.1-flash-lite",
    )
    .unwrap();

    assert_eq!(
        google["tools"][0]["functionDeclarations"][0]["name"],
        "lookup"
    );
    assert_eq!(
        google["tools"][0]["functionDeclarations"][0]["parameters"]["properties"]["query"]["type"],
        "string"
    );
    assert_eq!(google["toolConfig"]["functionCallingConfig"]["mode"], "ANY");
    assert_eq!(
        google["toolConfig"]["functionCallingConfig"]["allowedFunctionNames"][0],
        "lookup"
    );
}

#[test]
fn google_responses_function_call_and_output_items_map_to_gemini_parts() {
    let google = responses_to_google_generate_content(
        serde_json::json!({
            "model": "gemini-3.1-flash-lite",
            "input": [
                { "type": "message", "role": "user", "content": "look up amp" },
                {
                    "type": "function_call",
                    "call_id": "call_abc",
                    "name": "lookup",
                    "google_thought_signature": "sig_lookup",
                    "arguments": "{\"query\":\"amp\"}"
                },
                {
                    "type": "function_call_output",
                    "call_id": "call_abc",
                    "output": "{\"result\":\"ok\"}"
                }
            ]
        }),
        "gemini-3.1-flash-lite",
    )
    .unwrap();

    assert_eq!(google["contents"][1]["role"], "model");
    assert_eq!(
        google["contents"][1]["parts"][0]["functionCall"]["id"],
        "call_abc"
    );
    assert_eq!(
        google["contents"][1]["parts"][0]["functionCall"]["args"]["query"],
        "amp"
    );
    assert_eq!(
        google["contents"][1]["parts"][0]["thoughtSignature"],
        "sig_lookup"
    );
    assert_eq!(google["contents"][2]["role"], "user");
    assert_eq!(
        google["contents"][2]["parts"][0]["functionResponse"]["id"],
        "call_abc"
    );
    assert_eq!(
        google["contents"][2]["parts"][0]["functionResponse"]["response"]["result"],
        "ok"
    );
}

#[test]
fn google_responses_custom_tool_call_and_output_items_map_to_gemini_parts() {
    let google = responses_to_google_generate_content(
        serde_json::json!({
            "model": "gemini-3.1-flash-lite",
            "input": [
                { "type": "message", "role": "user", "content": "run exec" },
                {
                    "type": "custom_tool_call",
                    "call_id": "call_exec",
                    "name": "exec",
                    "google_thought_signature": "sig_exec",
                    "input": "printf hello"
                },
                {
                    "type": "custom_tool_call_output",
                    "call_id": "call_exec",
                    "name": "exec",
                    "output": "{\"result\":\"hello\"}"
                }
            ]
        }),
        "gemini-3.1-flash-lite",
    )
    .unwrap();

    assert_eq!(
        google["contents"][1]["parts"][0]["functionCall"]["args"]["input"],
        "printf hello"
    );
    assert_eq!(
        google["contents"][1]["parts"][0]["thoughtSignature"],
        "sig_exec"
    );
    assert_eq!(
        google["contents"][2]["parts"][0]["functionResponse"]["name"],
        "exec"
    );
}

#[test]
fn google_custom_tool_response_maps_back_to_custom_tool_call() {
    let request = serde_json::json!({
        "model": "gemini-3.1-flash-lite",
        "input": "run exec",
        "tools": [{
            "type": "custom",
            "name": "exec",
            "description": "Execute code",
            "format": { "type": "grammar", "syntax": "lark", "definition": "start: /.+/" }
        }]
    });
    let (_google, context) =
        responses_to_google_generate_content_with_context(request, "gemini-3.1-flash-lite")
            .unwrap();
    let responses = unified_model_proxy_v2::adapter::google_responses::google_generate_content_to_responses_with_context(
        serde_json::json!({
            "candidates": [{
                "content": {
                    "role": "model",
                    "parts": [{
                        "functionCall": {
                            "id": "call_exec",
                            "name": "exec",
                            "args": { "input": "printf hello" }
                        },
                        "thoughtSignature": "sig_exec"
                    }]
                },
                "finishReason": "STOP"
            }]
        }),
        "gemini-3.1-flash-lite",
        &context,
    )
    .unwrap();

    assert_eq!(responses["output"][0]["type"], "custom_tool_call");
    assert_eq!(responses["output"][0]["call_id"], "call_exec");
    assert_eq!(responses["output"][0]["name"], "exec");
    assert_eq!(responses["output"][0]["input"], "printf hello");
    assert_eq!(
        responses["output"][0]["google_thought_signature"],
        "sig_exec"
    );
}

#[test]
fn google_function_call_response_maps_to_responses_output_item() {
    let responses = google_generate_content_to_responses(
        serde_json::json!({
            "candidates": [{
                "content": {
                    "role": "model",
                    "parts": [{
                        "functionCall": {
                            "id": "call_abc",
                            "name": "lookup",
                            "args": { "query": "amp" }
                        },
                        "thoughtSignature": "sig_lookup"
                    }]
                },
                "finishReason": "STOP"
            }]
        }),
        "gemini-3.1-flash-lite",
    )
    .unwrap();

    assert_eq!(responses["output"][0]["type"], "function_call");
    assert_eq!(responses["output"][0]["call_id"], "call_abc");
    assert_eq!(responses["output"][0]["name"], "lookup");
    assert_eq!(responses["output"][0]["arguments"], "{\"query\":\"amp\"}");
    assert_eq!(
        responses["output"][0]["google_thought_signature"],
        "sig_lookup"
    );
}

#[test]
fn google_stream_custom_tool_call_maps_to_responses_events() {
    let request = serde_json::json!({
        "model": "gemini-3.1-flash-lite",
        "input": "run exec",
        "tools": [{
            "type": "custom",
            "name": "exec",
            "description": "Execute code",
            "format": { "type": "grammar", "syntax": "lark", "definition": "start: /.+/" }
        }]
    });
    let (_google, context) =
        responses_to_google_generate_content_with_context(request, "gemini-3.1-flash-lite")
            .unwrap();
    let mut translator =
        GoogleResponsesSseTranslator::with_tool_context("gemini-3.1-flash-lite", context);
    let first = translator
        .push_bytes(bytes::Bytes::from_static(
            br#"data: {"candidates":[{"content":{"role":"model","parts":[{"functionCall":{"id":"call_exec","name":"exec","args":{"input":"printf hello"}},"thoughtSignature":"sig_exec"}]},"finishReason":"STOP"}]}"#,
        ))
        .unwrap();
    let second = translator.finish().unwrap();
    let sse = String::from_utf8([first.as_ref(), second.as_ref()].concat()).unwrap();
    let events = sse_events(&sse);

    assert!(events
        .iter()
        .any(|event| event["type"] == "response.output_item.added"
            && event["item"]["type"] == "custom_tool_call"));
    assert!(events.iter().any(
        |event| event["type"] == "response.custom_tool_call_input.delta"
            && event["delta"] == "printf hello"
    ));
    assert!(events
        .iter()
        .any(|event| event["type"] == "response.output_item.done"
            && event["item"]["input"] == "printf hello"
            && event["item"]["google_thought_signature"] == "sig_exec"));
}

#[test]
fn google_stream_function_call_maps_to_responses_events() {
    let sse = google_generate_content_sse_to_responses_sse_text(
        r#"data: {"candidates":[{"content":{"role":"model","parts":[{"functionCall":{"id":"call_abc","name":"lookup","args":{"query":"amp"}},"thoughtSignature":"sig_lookup"}]},"finishReason":"STOP"}]}"#,
        "gemini-3.1-flash-lite",
    )
    .unwrap();
    let events = sse_events(&sse);

    assert!(events
        .iter()
        .any(|event| event["type"] == "response.created"));
    assert!(events
        .iter()
        .any(|event| event["type"] == "response.output_item.added"
            && event["item"]["type"] == "function_call"
            && event["item"]["call_id"] == "call_abc"));
    assert!(events.iter().any(
        |event| event["type"] == "response.function_call_arguments.delta"
            && event["delta"] == "{\"query\":\"amp\"}"
    ));
    assert!(events
        .iter()
        .any(|event| event["type"] == "response.output_item.done"
            && event["item"]["arguments"] == "{\"query\":\"amp\"}"
            && event["item"]["google_thought_signature"] == "sig_lookup"));
    assert!(events
        .iter()
        .any(|event| event["type"] == "response.completed"
            && event["response"]["output"][0]["type"] == "function_call"));
}

fn sse_events(sse: &str) -> Vec<Value> {
    sse.split("\n\n")
        .filter_map(|frame| {
            frame
                .lines()
                .find_map(|line| line.strip_prefix("data: "))
                .map(|data| serde_json::from_str(data).unwrap())
        })
        .collect()
}
