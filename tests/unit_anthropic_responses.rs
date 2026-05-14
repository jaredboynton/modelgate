use unified_model_proxy_v2::{
    adapter::anthropic_responses::{
        anthropic_message_to_responses_json, anthropic_message_to_responses_json_with_context,
        anthropic_messages_to_responses, anthropic_sse_to_responses_sse_text,
        anthropic_sse_to_responses_sse_text_with_model_and_context, chat_completions_to_responses,
        responses_json_to_anthropic_message, responses_sse_to_anthropic_sse_text,
        responses_to_anthropic_messages, responses_to_anthropic_messages_with_context,
        AnthropicSseStreamTranslator, ToolContext,
    },
    failure_capture::redact_failure_value,
};

#[test]
fn unit_anthropic_responses_text_request_maps_to_responses_input() {
    let body = serde_json::json!({
        "model": "openai:gpt-5.5",
        "system": [
            { "type": "text", "text": "Use terse answers." },
            { "type": "text", "text": "No markdown." }
        ],
        "max_tokens": 128,
        "temperature": 0.2,
        "metadata": { "user_id": "user-123" },
        "messages": [{ "role": "user", "content": "hello" }]
    });

    let responses = anthropic_messages_to_responses(body).unwrap();

    assert_eq!(responses["model"], "openai:gpt-5.5");
    assert_eq!(
        responses["instructions"],
        "Use terse answers.\n\nNo markdown."
    );
    assert_eq!(responses["max_output_tokens"], 128);
    assert_eq!(responses["temperature"], 0.2);
    assert_eq!(responses["user"], "user-123");
    assert_eq!(responses["input"][0]["type"], "message");
    assert_eq!(responses["input"][0]["role"], "user");
    assert_eq!(responses["input"][0]["content"][0]["type"], "input_text");
    assert_eq!(responses["input"][0]["content"][0]["text"], "hello");
}

#[test]
fn unit_anthropic_responses_base64_image_maps_to_data_url_and_failure_redaction_hides_it() {
    let body = serde_json::json!({
        "model": "openai:gpt-5.5",
        "messages": [{
            "role": "user",
            "content": [{
                "type": "image",
                "source": {
                    "type": "base64",
                    "media_type": "image/png",
                    "data": "base64-secret"
                }
            }]
        }]
    });

    let mut responses = anthropic_messages_to_responses(body).unwrap();
    assert_eq!(
        responses["input"][0]["content"][0]["image_url"],
        "data:image/png;base64,base64-secret"
    );

    redact_failure_value(&mut responses);
    assert_eq!(
        responses["input"][0]["content"][0]["image_url"],
        "[redacted]"
    );
}

#[test]
fn unit_anthropic_responses_tools_tool_results_tool_choice_and_thinking_map_to_responses() {
    let body = serde_json::json!({
        "model": "openai:gpt-5.5",
        "thinking": { "type": "enabled", "budget_tokens": 1000 },
        "tools": [{
            "name": "lookup",
            "description": "Fetch a thing",
            "input_schema": {
                "type": "object",
                "properties": { "query": { "type": "string" } }
            }
        }],
        "tool_choice": { "type": "any" },
        "messages": [
            {
                "role": "assistant",
                "content": [{
                    "type": "tool_use",
                    "id": "toolu_1",
                    "name": "lookup",
                    "input": { "query": "amp" }
                }]
            },
            {
                "role": "user",
                "content": [{
                    "type": "tool_result",
                    "tool_use_id": "toolu_1",
                    "content": [
                        { "type": "text", "text": "result " },
                        { "type": "text", "text": "text" }
                    ]
                }]
            }
        ]
    });

    let responses = anthropic_messages_to_responses(body).unwrap();

    assert_eq!(responses["reasoning"]["effort"], "low");
    assert_eq!(responses["reasoning"]["summary"], "auto");
    assert_eq!(responses["tools"][0]["type"], "function");
    assert_eq!(responses["tools"][0]["name"], "lookup");
    assert_eq!(
        responses["tools"][0]["parameters"]["properties"]["query"]["type"],
        "string"
    );
    assert_eq!(responses["tool_choice"], "required");
    assert_eq!(responses["input"][0]["type"], "function_call");
    assert_eq!(responses["input"][0]["call_id"], "toolu_1");
    assert_eq!(responses["input"][0]["name"], "lookup");
    assert_eq!(responses["input"][0]["arguments"], "{\"query\":\"amp\"}");
    assert_eq!(responses["input"][1]["type"], "function_call_output");
    assert_eq!(responses["input"][1]["call_id"], "toolu_1");
    assert_eq!(responses["input"][1]["output"], "result text");
}

#[test]
fn unit_anthropic_responses_tool_choice_variants_map_to_responses_shapes() {
    let base = |choice: serde_json::Value| {
        anthropic_messages_to_responses(serde_json::json!({
            "model": "openai:gpt-5.5",
            "tool_choice": choice,
            "messages": [{ "role": "user", "content": "hello" }]
        }))
        .unwrap()
    };

    assert_eq!(
        base(serde_json::json!({ "type": "auto" }))["tool_choice"],
        "auto"
    );
    assert_eq!(
        base(serde_json::json!({ "type": "none" }))["tool_choice"],
        "none"
    );
    assert_eq!(
        base(serde_json::json!({ "type": "tool", "name": "lookup" }))["tool_choice"],
        serde_json::json!({ "type": "function", "name": "lookup" })
    );
}

#[test]
fn unit_anthropic_responses_unsupported_hosted_tool_is_bad_request() {
    let err = anthropic_messages_to_responses(serde_json::json!({
        "model": "openai:gpt-5.5",
        "tools": [{ "type": "web_search", "name": "web_search" }],
        "messages": [{ "role": "user", "content": "hello" }]
    }))
    .unwrap_err();

    assert_eq!(err.error_type(), "invalid_request");
    assert!(err
        .to_string()
        .contains("unsupported Anthropic hosted tool"));
}

#[test]
fn unit_anthropic_responses_chat_completions_use_canonical_responses_items() {
    let body = serde_json::json!({
        "model": "openai:gpt-5.5",
        "reasoning_effort": "high",
        "messages": [
            { "role": "system", "content": "Be direct." },
            { "role": "user", "content": "hello" },
            {
                "role": "assistant",
                "content": "calling",
                "tool_calls": [{
                    "id": "call_1",
                    "type": "function",
                    "function": { "name": "lookup", "arguments": "{\"q\":\"amp\"}" }
                }]
            },
            { "role": "tool", "tool_call_id": "call_1", "content": "done" }
        ],
        "tools": [{
            "type": "function",
            "function": {
                "name": "lookup",
                "parameters": { "type": "object", "properties": {} }
            }
        }]
    });

    let responses = chat_completions_to_responses(body).unwrap();

    assert_eq!(responses["instructions"], "Be direct.");
    assert_eq!(responses["service_tier"], "priority");
    assert_eq!(responses["reasoning"]["effort"], "high");
    assert_eq!(responses["input"][0]["content"][0]["type"], "input_text");
    assert_eq!(responses["input"][1]["content"][0]["type"], "output_text");
    assert_eq!(responses["input"][2]["type"], "function_call");
    assert_eq!(responses["input"][2]["arguments"], "{\"q\":\"amp\"}");
    assert_eq!(responses["input"][3]["type"], "function_call_output");
    assert_eq!(responses["tools"][0]["type"], "function");
    assert_eq!(responses["tools"][0]["name"], "lookup");
}

#[test]
fn unit_anthropic_responses_json_text_tool_usage_and_model_map_to_anthropic_message() {
    let response = serde_json::json!({
        "id": "resp_1",
        "status": "completed",
        "output": [
            {
                "type": "message",
                "content": [{ "type": "output_text", "text": "hello" }]
            },
            {
                "type": "function_call",
                "call_id": "call_1",
                "name": "lookup",
                "arguments": "{\"query\":\"amp\"}"
            }
        ],
        "usage": {
            "input_tokens": 10,
            "output_tokens": 5,
            "input_tokens_details": { "cached_tokens": 3 }
        }
    });

    let message = responses_json_to_anthropic_message(response, "openai:gpt-5.5").unwrap();

    assert_eq!(message["id"], "msg_resp_1");
    assert_eq!(message["model"], "openai:gpt-5.5");
    assert_eq!(message["stop_reason"], "tool_use");
    assert_eq!(
        message["content"][0],
        serde_json::json!({ "type": "text", "text": "hello" })
    );
    assert_eq!(message["content"][1]["type"], "tool_use");
    assert_eq!(message["content"][1]["id"], "call_1");
    assert_eq!(message["content"][1]["input"]["query"], "amp");
    assert_eq!(message["usage"]["input_tokens"], 10);
    assert_eq!(message["usage"]["output_tokens"], 5);
    assert_eq!(message["usage"]["cache_read_input_tokens"], 3);
}

#[test]
fn unit_anthropic_responses_json_incomplete_maps_to_max_tokens() {
    let response = serde_json::json!({
        "id": "resp_2",
        "status": "incomplete",
        "incomplete_details": { "reason": "max_output_tokens" },
        "output": []
    });

    let message = responses_json_to_anthropic_message(response, "openai:gpt-5.5").unwrap();

    assert_eq!(message["stop_reason"], "max_tokens");
    assert_eq!(message["content"].as_array().unwrap().len(), 0);
}

#[test]
fn unit_anthropic_responses_sse_text_stream_maps_to_anthropic_sse() {
    let input = concat!(
        "event: response.created\n",
        "data: {\"response\":{\"id\":\"resp_1\"}}\n",
        "\n",
        "event: response.output_item.added\n",
        "data: {\"item\":{\"id\":\"msg_item\",\"type\":\"message\"}}\n",
        "\n",
        "event: response.output_text.delta\n",
        "data: {\"item_id\":\"msg_item\",\"delta\":\"hello\"}\n",
        "\n",
        "event: response.output_item.done\n",
        "data: {\"item\":{\"id\":\"msg_item\",\"type\":\"message\",\"content\":[{\"type\":\"output_text\",\"text\":\"hello\"}]}}\n",
        "\n",
        "event: response.completed\n",
        "data: {\"response\":{\"id\":\"resp_1\",\"status\":\"completed\",\"output\":[],\"usage\":{\"input_tokens\":1,\"output_tokens\":2}}}\n",
        "\n",
    );

    let anthropic = responses_sse_to_anthropic_sse_text(input, "openai:gpt-5.5").unwrap();

    assert!(anthropic.contains("event: message_start"));
    assert!(anthropic.contains("\"model\":\"openai:gpt-5.5\""));
    assert!(anthropic.contains("event: content_block_start"));
    assert!(anthropic.contains("\"type\":\"text_delta\""));
    assert!(anthropic.contains("\"text\":\"hello\""));
    assert!(anthropic.contains("\"stop_reason\":\"end_turn\""));
    assert!(anthropic.contains("\"input_tokens\":1"));
    assert!(anthropic.contains("event: message_stop"));
}

#[test]
fn unit_anthropic_responses_sse_tool_stream_maps_argument_deltas() {
    let input = concat!(
        "event: response.output_item.added\n",
        "data: {\"item\":{\"id\":\"fc_1\",\"type\":\"function_call\",\"call_id\":\"call_1\",\"name\":\"lookup\"}}\n",
        "\n",
        "event: response.function_call_arguments.delta\n",
        "data: {\"item_id\":\"fc_1\",\"delta\":\"{\\\"q\\\":\"}\n",
        "\n",
        "event: response.function_call_arguments.delta\n",
        "data: {\"item_id\":\"fc_1\",\"delta\":\"\\\"amp\\\"}\"}\n",
        "\n",
        "event: response.output_item.done\n",
        "data: {\"item\":{\"id\":\"fc_1\",\"type\":\"function_call\",\"call_id\":\"call_1\",\"name\":\"lookup\",\"arguments\":\"{\\\"q\\\":\\\"amp\\\"}\"}}\n",
        "\n",
        "event: response.completed\n",
        "data: {\"response\":{\"id\":\"resp_1\",\"status\":\"completed\",\"output\":[]}}\n",
        "\n",
    );

    let anthropic = responses_sse_to_anthropic_sse_text(input, "openai:gpt-5.5").unwrap();

    assert!(anthropic.contains("\"type\":\"tool_use\""));
    assert!(anthropic.contains("\"id\":\"call_1\""));
    assert!(anthropic.contains("\"type\":\"input_json_delta\""));
    assert!(anthropic.contains("\"stop_reason\":\"tool_use\""));
}

#[test]
fn unit_responses_to_anthropic_messages_string_input_maps_to_user_message() {
    let body = serde_json::json!({
        "model": "anthropic:claude-sonnet-4-5",
        "input": "hello"
    });

    let anthropic = responses_to_anthropic_messages(body).unwrap();

    assert_eq!(anthropic["model"], "anthropic:claude-sonnet-4-5");
    assert_eq!(anthropic["messages"][0]["role"], "user");
    assert_eq!(anthropic["messages"][0]["content"], "hello");
}

#[test]
fn unit_responses_to_anthropic_messages_array_maps_instructions_tokens_tools_and_tool_outputs() {
    let body = serde_json::json!({
        "model": "anthropic:claude-sonnet-4-5",
        "instructions": "Use terse answers.",
        "max_output_tokens": 256,
        "input": [
            {
                "type": "message",
                "role": "user",
                "content": [{ "type": "input_text", "text": "lookup amp" }]
            },
            {
                "type": "function_call",
                "call_id": "call_1",
                "name": "lookup",
                "arguments": "{\"query\":\"amp\"}"
            },
            {
                "type": "function_call_output",
                "call_id": "call_1",
                "output": "result text"
            }
        ],
        "tools": [{
            "type": "function",
            "name": "lookup",
            "description": "Fetch a thing",
            "parameters": {
                "type": "object",
                "properties": { "query": { "type": "string" } }
            }
        }]
    });

    let anthropic = responses_to_anthropic_messages(body).unwrap();

    assert_eq!(anthropic["system"], "Use terse answers.");
    assert_eq!(anthropic["max_tokens"], 256);
    assert_eq!(anthropic["tools"][0]["name"], "lookup");
    assert_eq!(anthropic["tools"][0]["description"], "Fetch a thing");
    assert_eq!(
        anthropic["tools"][0]["input_schema"]["properties"]["query"]["type"],
        "string"
    );
    assert_eq!(anthropic["messages"][0]["role"], "user");
    assert_eq!(anthropic["messages"][0]["content"][0]["type"], "text");
    assert_eq!(anthropic["messages"][0]["content"][0]["text"], "lookup amp");
    assert_eq!(anthropic["messages"][1]["role"], "assistant");
    assert_eq!(anthropic["messages"][1]["content"][0]["type"], "tool_use");
    assert_eq!(anthropic["messages"][1]["content"][0]["id"], "call_1");
    assert_eq!(anthropic["messages"][1]["content"][0]["name"], "lookup");
    assert_eq!(
        anthropic["messages"][1]["content"][0]["input"],
        serde_json::json!({ "query": "amp" })
    );
    assert_eq!(anthropic["messages"][2]["role"], "user");
    assert_eq!(
        anthropic["messages"][2]["content"][0]["type"],
        "tool_result"
    );
    assert_eq!(
        anthropic["messages"][2]["content"][0]["tool_use_id"],
        "call_1"
    );
    assert_eq!(
        anthropic["messages"][2]["content"][0]["content"],
        "result text"
    );
}

#[test]
fn unit_responses_to_anthropic_messages_accepts_parallel_custom_tools_and_groups_results() {
    let body = serde_json::json!({
        "model": "claude-opus-4-7",
        "parallel_tool_calls": true,
        "input": [
            {
                "type": "message",
                "role": "user",
                "content": [{ "type": "input_text", "text": "use both tools" }]
            },
            {
                "type": "function_call",
                "call_id": "call_lookup",
                "name": "lookup",
                "arguments": "{\"query\":\"amp\"}"
            },
            {
                "type": "custom_tool_call",
                "call_id": "call_patch",
                "name": "apply_patch",
                "input": "*** Begin Patch\n*** End Patch\n"
            },
            {
                "type": "function_call_output",
                "call_id": "call_lookup",
                "output": "found"
            },
            {
                "type": "custom_tool_call_output",
                "call_id": "call_patch",
                "output": "patched"
            }
        ],
        "tools": [
            {
                "type": "function",
                "name": "lookup",
                "parameters": { "type": "object", "properties": {} }
            },
            { "type": "custom", "name": "apply_patch" }
        ]
    });

    let (anthropic, context) = responses_to_anthropic_messages_with_context(body).unwrap();

    assert_eq!(anthropic["tools"][0]["name"], "lookup");
    assert_eq!(anthropic["tools"][1]["name"], "apply_patch");
    assert_eq!(anthropic["messages"][1]["role"], "assistant");
    assert_eq!(
        anthropic["messages"][1]["content"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    assert_eq!(anthropic["messages"][1]["content"][0]["name"], "lookup");
    assert_eq!(
        anthropic["messages"][1]["content"][1]["name"],
        "apply_patch"
    );
    assert_eq!(
        anthropic["messages"][1]["content"][1]["input"],
        serde_json::json!({ "input": "*** Begin Patch\n*** End Patch\n" })
    );
    assert_eq!(anthropic["messages"][2]["role"], "user");
    assert_eq!(
        anthropic["messages"][2]["content"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    assert_eq!(
        anthropic["messages"][2]["content"][0]["tool_use_id"],
        "call_lookup"
    );
    assert_eq!(
        anthropic["messages"][2]["content"][1]["tool_use_id"],
        "call_patch"
    );

    let message = serde_json::json!({
        "id": "msg_1",
        "type": "message",
        "role": "assistant",
        "model": "claude-opus-4-7",
        "stop_reason": "tool_use",
        "content": [{
            "type": "tool_use",
            "id": "call_patch",
            "name": "apply_patch",
            "input": { "input": "*** Begin Patch\n*** End Patch\n" }
        }]
    });
    let responses = anthropic_message_to_responses_json_with_context(message, &context).unwrap();
    assert_eq!(responses["output"][0]["type"], "custom_tool_call");
    assert_eq!(responses["output"][0]["call_id"], "call_patch");
    assert_eq!(
        responses["output"][0]["input"],
        "*** Begin Patch\n*** End Patch\n"
    );
}

#[test]
fn unit_responses_to_anthropic_messages_maps_reasoning_effort_boundaries() {
    let with_effort = |effort: &str| {
        responses_to_anthropic_messages(serde_json::json!({
            "model": "anthropic/claude-opus-4-7",
            "input": "hello",
            "reasoning": { "effort": effort }
        }))
        .unwrap()
    };

    assert_eq!(with_effort("low")["thinking"]["type"], "adaptive");
    assert_eq!(with_effort("low")["output_config"]["effort"], "low");
    assert_eq!(with_effort("medium")["output_config"]["effort"], "medium");
    assert_eq!(with_effort("high")["output_config"]["effort"], "high");
    assert_eq!(with_effort("xhigh")["output_config"]["effort"], "xhigh");
    assert_eq!(with_effort("max")["output_config"]["effort"], "max");
    assert_eq!(with_effort("minimal")["output_config"]["effort"], "low");
    assert!(with_effort("none").get("thinking").is_none());
}

#[test]
fn unit_responses_to_anthropic_messages_downgrades_xhigh_for_non_xhigh_adaptive_models() {
    let messages = responses_to_anthropic_messages(serde_json::json!({
        "model": "anthropic/claude-sonnet-4-6",
        "input": "hello",
        "reasoning": { "effort": "xhigh" }
    }))
    .unwrap();

    assert_eq!(messages["output_config"]["effort"], "high");
}

#[test]
fn unit_responses_to_anthropic_messages_rejects_budget_tokens_for_adaptive_models() {
    let err = responses_to_anthropic_messages(serde_json::json!({
        "model": "anthropic/claude-opus-4-7",
        "input": "hello",
        "reasoning": { "budget_tokens": 4096 }
    }))
    .unwrap_err();

    assert!(err.to_string().contains("budget_tokens"));
}

#[test]
fn unit_responses_to_anthropic_messages_maps_manual_thinking_for_non_adaptive_models() {
    let with_effort = |effort: &str| {
        responses_to_anthropic_messages(serde_json::json!({
            "model": "anthropic/claude-haiku-4-5",
            "max_output_tokens": 4096,
            "input": "hello",
            "reasoning": { "effort": effort }
        }))
        .unwrap()
    };

    assert_eq!(with_effort("low")["thinking"]["budget_tokens"], 1_024);
    assert_eq!(with_effort("medium")["thinking"]["budget_tokens"], 2_048);
    assert_eq!(with_effort("high")["thinking"]["budget_tokens"], 4_096);
    assert_eq!(with_effort("xhigh")["thinking"]["budget_tokens"], 8_192);
    assert_eq!(with_effort("max")["thinking"]["budget_tokens"], 16_384);
}

#[test]
fn unit_responses_to_anthropic_messages_rejects_sampling_when_thinking_enabled() {
    let err = responses_to_anthropic_messages(serde_json::json!({
        "model": "anthropic/claude-opus-4-7",
        "temperature": 0.2,
        "input": "hello",
        "reasoning": { "effort": "low" }
    }))
    .unwrap_err();

    assert!(err.to_string().contains("temperature"));
}

#[test]
fn unit_responses_to_anthropic_messages_rejects_forced_tool_choice_when_thinking_enabled() {
    let err = responses_to_anthropic_messages(serde_json::json!({
        "model": "anthropic/claude-opus-4-7",
        "input": "hello",
        "reasoning": { "effort": "low" },
        "tool_choice": "required"
    }))
    .unwrap_err();

    assert!(err.to_string().contains("tool_choice"));
}

#[test]
fn unit_responses_to_anthropic_messages_rejects_unsupported_responses_features() {
    let unsupported = [
        (
            "previous_response_id",
            serde_json::json!({ "previous_response_id": "resp_prev" }),
        ),
        (
            "conversation",
            serde_json::json!({ "conversation": { "id": "conv_1" } }),
        ),
        ("store_true", serde_json::json!({ "store": true })),
        (
            "input_file",
            serde_json::json!({
                "input": [{
                    "type": "message",
                    "role": "user",
                    "content": [{ "type": "input_file", "file_id": "file_1" }]
                }]
            }),
        ),
    ];

    for (case, patch) in unsupported {
        let mut body = serde_json::json!({
            "model": "anthropic:claude-sonnet-4-5",
            "input": "hello"
        });
        body.as_object_mut()
            .unwrap()
            .extend(patch.as_object().unwrap().clone());

        let err = responses_to_anthropic_messages(body).unwrap_err();
        assert_eq!(err.error_type(), "invalid_request", "{case}");
    }
}

#[test]
fn unit_anthropic_message_to_responses_json_maps_text_tool_usage_and_model() {
    let message = serde_json::json!({
        "id": "msg_1",
        "type": "message",
        "role": "assistant",
        "model": "claude-sonnet-4-5",
        "stop_reason": "tool_use",
        "content": [
            { "type": "text", "text": "hello" },
            {
                "type": "tool_use",
                "id": "toolu_1",
                "name": "lookup",
                "input": { "query": "amp" }
            }
        ],
        "usage": {
            "input_tokens": 10,
            "output_tokens": 5,
            "cache_read_input_tokens": 3
        }
    });

    let responses = anthropic_message_to_responses_json(message).unwrap();

    assert_eq!(responses["id"], "resp_msg_1");
    assert_eq!(responses["model"], "claude-sonnet-4-5");
    assert_eq!(responses["status"], "completed");
    assert_eq!(responses["output"][0]["type"], "message");
    assert_eq!(responses["output"][0]["role"], "assistant");
    assert_eq!(responses["output"][0]["content"][0]["type"], "output_text");
    assert_eq!(responses["output"][0]["content"][0]["text"], "hello");
    assert_eq!(responses["output"][1]["type"], "function_call");
    assert_eq!(responses["output"][1]["call_id"], "toolu_1");
    assert_eq!(responses["output"][1]["name"], "lookup");
    assert_eq!(responses["output"][1]["arguments"], "{\"query\":\"amp\"}");
    assert_eq!(responses["usage"]["input_tokens"], 10);
    assert_eq!(responses["usage"]["output_tokens"], 5);
    assert_eq!(
        responses["usage"]["input_tokens_details"]["cached_tokens"],
        3
    );
}

#[test]
fn unit_anthropic_sse_to_responses_sse_text_maps_text_and_tool_stream() {
    let input = concat!(
        "event: message_start\n",
        "data: {\"type\":\"message_start\",\"message\":{\"id\":\"msg_1\",\"type\":\"message\",\"role\":\"assistant\",\"model\":\"claude-sonnet-4-5\",\"content\":[],\"stop_reason\":null,\"usage\":{\"input_tokens\":1,\"output_tokens\":0}}}\n",
        "\n",
        "event: content_block_start\n",
        "data: {\"type\":\"content_block_start\",\"index\":0,\"content_block\":{\"type\":\"text\",\"text\":\"\"}}\n",
        "\n",
        "event: content_block_delta\n",
        "data: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"text_delta\",\"text\":\"hello\"}}\n",
        "\n",
        "event: content_block_stop\n",
        "data: {\"type\":\"content_block_stop\",\"index\":0}\n",
        "\n",
        "event: content_block_start\n",
        "data: {\"type\":\"content_block_start\",\"index\":1,\"content_block\":{\"type\":\"tool_use\",\"id\":\"toolu_1\",\"name\":\"lookup\",\"input\":{}}}\n",
        "\n",
        "event: content_block_delta\n",
        "data: {\"type\":\"content_block_delta\",\"index\":1,\"delta\":{\"type\":\"input_json_delta\",\"partial_json\":\"{\\\"query\\\":\"}}\n",
        "\n",
        "event: content_block_delta\n",
        "data: {\"type\":\"content_block_delta\",\"index\":1,\"delta\":{\"type\":\"input_json_delta\",\"partial_json\":\"\\\"amp\\\"}\"}}\n",
        "\n",
        "event: content_block_stop\n",
        "data: {\"type\":\"content_block_stop\",\"index\":1}\n",
        "\n",
        "event: message_delta\n",
        "data: {\"type\":\"message_delta\",\"delta\":{\"stop_reason\":\"tool_use\"},\"usage\":{\"output_tokens\":5}}\n",
        "\n",
        "event: message_stop\n",
        "data: {\"type\":\"message_stop\"}\n",
        "\n",
    );

    let responses = anthropic_sse_to_responses_sse_text(input).unwrap();

    assert!(responses.contains("event: response.created"));
    assert!(responses.contains("\"model\":\"claude-sonnet-4-5\""));
    assert!(responses.contains("event: response.output_item.added"));
    assert!(responses.contains("event: response.output_text.delta"));
    assert!(responses.contains("\"delta\":\"hello\""));
    assert!(responses.contains("\"type\":\"function_call\""));
    assert!(responses.contains("\"call_id\":\"toolu_1\""));
    assert!(responses.contains("event: response.function_call_arguments.delta"));
    assert!(responses.contains("\"delta\":\"{\\\"query\\\":\""));
    assert!(responses.contains("\"delta\":\"\\\"amp\\\"}\""));
    assert!(responses.contains("event: response.completed"));
    assert!(responses.contains("\"input_tokens\":1"));
    assert!(responses.contains("\"output_tokens\":5"));
}

#[test]
fn unit_anthropic_sse_to_responses_sse_text_uses_custom_tool_context() {
    let input = concat!(
        "event: message_start\n",
        "data: {\"type\":\"message_start\",\"message\":{\"id\":\"msg_1\",\"type\":\"message\",\"role\":\"assistant\",\"model\":\"claude-opus-4-7\",\"content\":[],\"stop_reason\":null,\"usage\":{\"input_tokens\":1,\"output_tokens\":0}}}\n",
        "\n",
        "event: content_block_start\n",
        "data: {\"type\":\"content_block_start\",\"index\":0,\"content_block\":{\"type\":\"tool_use\",\"id\":\"toolu_1\",\"name\":\"apply_patch\",\"input\":{}}}\n",
        "\n",
        "event: content_block_delta\n",
        "data: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"input_json_delta\",\"partial_json\":\"{\\\"input\\\":\\\"patch\\\"}\"}}\n",
        "\n",
        "event: content_block_stop\n",
        "data: {\"type\":\"content_block_stop\",\"index\":0}\n",
        "\n",
        "event: message_delta\n",
        "data: {\"type\":\"message_delta\",\"delta\":{\"stop_reason\":\"tool_use\"},\"usage\":{\"output_tokens\":5}}\n",
        "\n",
        "event: message_stop\n",
        "data: {\"type\":\"message_stop\"}\n",
        "\n",
    );

    let responses = anthropic_sse_to_responses_sse_text_with_model_and_context(
        input,
        "claude-opus-4-7",
        ToolContext::default().with_custom_tool("apply_patch"),
    )
    .unwrap();

    assert!(responses.contains("\"type\":\"custom_tool_call\""));
    assert!(responses.contains("event: response.custom_tool_call_input.delta"));
    assert!(responses.contains("\"input\":\"patch\""));
}

#[test]
fn unit_anthropic_sse_stream_translator_emits_complete_events_incrementally() {
    let mut translator = AnthropicSseStreamTranslator::with_model("anthropic/claude-opus-4-7");

    let first = translator
        .push_bytes(bytes::Bytes::from_static(
            b"event: message_start\ndata: {\"type\":\"message_start\",\"message\":{\"id\":\"msg_1\",\"type\":\"message\",\"role\":\"assistant\",\"model\":\"claude-opus-4-7\",\"content\":[],\"stop_reason\":null,\"usage\":{\"input_tokens\":1,\"output_tokens\":0}}}\n\n",
        ))
        .unwrap();
    assert!(!first.is_empty());
    let first = std::str::from_utf8(&first).unwrap();
    assert!(first.contains("event: response.created"));
    assert!(first.contains("\"model\":\"anthropic/claude-opus-4-7\""));

    let partial = translator
        .push_bytes(bytes::Bytes::from_static(
            b"event: content_block_start\ndata: {\"type\":\"content_block_start\",\"index\":0,\"content_block\":{\"type\":\"text\",\"text\":\"\"}}\n",
        ))
        .unwrap();
    assert!(partial.is_empty());

    let completed = translator
        .push_bytes(bytes::Bytes::from_static(b"\n"))
        .unwrap();
    assert!(!completed.is_empty());
    let completed = std::str::from_utf8(&completed).unwrap();
    assert!(completed.contains("event: response.output_item.added"));
}

#[test]
fn unit_anthropic_sse_stream_translator_ignores_thinking_blocks() {
    let mut translator = AnthropicSseStreamTranslator::with_model("claude-haiku-4-5");

    let frames = translator
        .push_bytes(bytes::Bytes::from_static(
            concat!(
                "event: message_start\n",
                "data: {\"type\":\"message_start\",\"message\":{\"id\":\"msg_1\",\"type\":\"message\",\"role\":\"assistant\",\"model\":\"claude-haiku-4-5\",\"content\":[],\"usage\":{\"input_tokens\":1,\"output_tokens\":0}}}\n",
                "\n",
                "event: content_block_start\n",
                "data: {\"type\":\"content_block_start\",\"index\":0,\"content_block\":{\"type\":\"thinking\",\"thinking\":\"\"}}\n",
                "\n",
                "event: content_block_delta\n",
                "data: {\"type\":\"content_block_delta\",\"index\":0,\"delta\":{\"type\":\"thinking_delta\",\"thinking\":\"scratch\"}}\n",
                "\n",
                "event: content_block_stop\n",
                "data: {\"type\":\"content_block_stop\",\"index\":0}\n",
                "\n",
                "event: content_block_start\n",
                "data: {\"type\":\"content_block_start\",\"index\":1,\"content_block\":{\"type\":\"text\",\"text\":\"\"}}\n",
                "\n",
                "event: content_block_delta\n",
                "data: {\"type\":\"content_block_delta\",\"index\":1,\"delta\":{\"type\":\"text_delta\",\"text\":\"OK\"}}\n",
                "\n",
                "event: content_block_stop\n",
                "data: {\"type\":\"content_block_stop\",\"index\":1}\n",
                "\n",
                "event: message_stop\n",
                "data: {\"type\":\"message_stop\"}\n",
                "\n"
            )
            .as_bytes(),
        ))
        .unwrap();
    let frames = std::str::from_utf8(&frames).unwrap();

    assert!(frames.contains("event: response.created"));
    assert!(frames.contains("\"delta\":\"OK\""));
    assert!(frames.contains("event: response.completed"));
    assert!(!frames.contains("unsupported Anthropic SSE content block"));
    assert!(!frames.contains("scratch"));
}

#[test]
fn unit_max_alias_strips_suffix_and_forces_max_effort_on_responses_request() {
    let body = serde_json::json!({
        "model": "anthropic/claude-opus-4-7-max",
        "input": [{
            "role": "user",
            "content": [{ "type": "input_text", "text": "hi" }]
        }]
    });
    let messages = responses_to_anthropic_messages(body).unwrap();
    assert_eq!(messages["model"], "anthropic/claude-opus-4-7");
    assert_eq!(messages["thinking"]["type"], "adaptive");
    assert_eq!(messages["output_config"]["effort"], "max");
}

#[test]
fn unit_max_alias_forces_max_when_codex_sent_xhigh() {
    let body = serde_json::json!({
        "model": "anthropic/claude-opus-4-7-max",
        "reasoning": { "effort": "xhigh", "summary": "auto" },
        "input": [{
            "role": "user",
            "content": [{ "type": "input_text", "text": "hi" }]
        }]
    });
    let messages = responses_to_anthropic_messages(body).unwrap();
    assert_eq!(messages["output_config"]["effort"], "max");
}

#[test]
fn unit_sonnet_max_alias_forces_max_when_codex_sent_high() {
    let body = serde_json::json!({
        "model": "anthropic/claude-sonnet-4-6-max",
        "reasoning": { "effort": "high", "summary": "auto" },
        "input": [{
            "role": "user",
            "content": [{ "type": "input_text", "text": "hi" }]
        }]
    });
    let messages = responses_to_anthropic_messages(body).unwrap();
    assert_eq!(messages["model"], "anthropic/claude-sonnet-4-6");
    assert_eq!(messages["thinking"]["type"], "adaptive");
    assert_eq!(messages["output_config"]["effort"], "max");
}

#[test]
fn unit_hosted_responses_tools_are_ignored_for_anthropic_adapter() {
    let body = serde_json::json!({
        "model": "anthropic/claude-opus-4-7",
        "tools": [
            { "type": "local_shell" },
            { "type": "image_generation" },
            { "type": "function", "name": "echo", "parameters": { "type": "object" } }
        ],
        "input": [{
            "role": "user",
            "content": [{ "type": "input_text", "text": "ls" }]
        }]
    });
    let messages = responses_to_anthropic_messages(body).unwrap();
    let tools = messages["tools"].as_array().unwrap();
    assert_eq!(tools.len(), 1);
    assert_eq!(tools[0]["name"], "echo");
}

#[test]
fn unit_custom_and_freeform_tools_map_to_function_with_input_arg() {
    let body = serde_json::json!({
        "model": "anthropic/claude-opus-4-7",
        "tools": [
            { "type": "freeform", "name": "apply_patch", "description": "Apply diff" },
            { "type": "custom", "name": "draft_note" }
        ],
        "input": [{
            "role": "user",
            "content": [{ "type": "input_text", "text": "go" }]
        }]
    });
    let messages = responses_to_anthropic_messages(body).unwrap();
    assert_eq!(messages["tools"][0]["name"], "apply_patch");
    assert_eq!(
        messages["tools"][0]["input_schema"]["properties"]["input"]["type"],
        "string"
    );
    assert_eq!(messages["tools"][1]["name"], "draft_note");
}

#[test]
fn unit_web_search_tool_is_ignored_for_anthropic_responses_adapter() {
    let body = serde_json::json!({
        "model": "anthropic/claude-opus-4-7",
        "tools": [
            { "type": "web_search" },
            { "type": "function", "name": "echo", "parameters": { "type": "object" } }
        ],
        "input": [{
            "role": "user",
            "content": [{ "type": "input_text", "text": "go" }]
        }]
    });

    let messages = responses_to_anthropic_messages(body).unwrap();
    let tools = messages["tools"].as_array().unwrap();
    assert_eq!(tools.len(), 1);
    assert_eq!(tools[0]["name"], "echo");
}

#[test]
fn unit_unknown_tool_types_are_rejected() {
    let body = serde_json::json!({
        "model": "anthropic/claude-opus-4-7",
        "tools": [
            { "type": "totally_unknown" },
            { "type": "function", "name": "echo", "parameters": { "type": "object" } }
        ],
        "input": [{
            "role": "user",
            "content": [{ "type": "input_text", "text": "go" }]
        }]
    });
    let err = responses_to_anthropic_messages(body).unwrap_err();
    assert!(err.to_string().contains("totally_unknown"));
}
