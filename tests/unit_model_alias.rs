use unified_model_proxy_v2::{
    model_alias::{resolve_model, Provider, KNOWN_MODELS},
    route::chat::{route_for_chat_model, ChatRoute},
    route::messages::{route_for_messages_model, MessagesRoute},
    route::responses::{route_for_responses_model, ResponsesRoute},
};

#[test]
fn maps_known_aliases() {
    let alias = resolve_model("openai:gpt-5.5").unwrap();
    assert_eq!(alias.provider, Provider::Codex);
    assert_eq!(alias.upstream_model, "gpt-5.5");

    let alias = resolve_model("vertexai/gemini-3-pro-image").unwrap();
    assert_eq!(alias.provider, Provider::Google);
    assert_eq!(alias.upstream_model, "gemini-3-pro-image-preview");

    let alias = resolve_model("anthropic/claude-haiku-4-5-20251001").unwrap();
    assert_eq!(alias.provider, Provider::Bedrock);
    assert_eq!(alias.upstream_model, "anthropic.claude-haiku-4-5");

    let alias = resolve_model("claude-haiku-4-5-20251001").unwrap();
    assert_eq!(alias.provider, Provider::Bedrock);
    assert_eq!(alias.upstream_model, "anthropic.claude-haiku-4-5");

    let alias = resolve_model("claude-sonnet-4-6").unwrap();
    assert_eq!(alias.provider, Provider::Bedrock);
    assert_eq!(alias.upstream_model, "us.anthropic.claude-sonnet-4-6");

    let alias = resolve_model("claude-sonnet-4-6-max").unwrap();
    assert_eq!(alias.provider, Provider::Bedrock);
    assert_eq!(alias.upstream_model, "us.anthropic.claude-sonnet-4-6");

    let alias = resolve_model("claude-opus-4-6").unwrap();
    assert_eq!(alias.provider, Provider::Bedrock);
    assert_eq!(alias.upstream_model, "us.anthropic.claude-opus-4-6-v1");

    let alias = resolve_model("claude-opus-4-7").unwrap();
    assert_eq!(alias.provider, Provider::Bedrock);
    assert_eq!(alias.upstream_model, "anthropic.claude-opus-4-7");

    let alias = resolve_model("gemini-3.1-flash-lite").unwrap();
    assert_eq!(alias.provider, Provider::Google);
    assert_eq!(alias.upstream_model, "gemini-3.1-flash-lite");

    let alias = resolve_model("gemini-3.1-pro-preview").unwrap();
    assert_eq!(alias.provider, Provider::Google);
    assert_eq!(alias.upstream_model, "gemini-3.1-pro-preview");

    let alias = resolve_model("vertexai/gemini-3.1-flash-lite").unwrap();
    assert_eq!(alias.provider, Provider::Google);
    assert_eq!(alias.upstream_model, "gemini-3.1-flash-lite");

    let alias = resolve_model("gpt-5.5").unwrap();
    assert_eq!(alias.provider, Provider::Codex);
    assert_eq!(alias.upstream_model, "gpt-5.5");
}

#[test]
fn dated_suffix_is_rejected_when_row_flag_is_false_or_row_missing() {
    // Codex row has accepts_dated_snapshots: false.
    assert!(resolve_model("gpt-5.5-20260101").is_none());

    // Anthropic opus-4-7 row has accepts_dated_snapshots: false until a real
    // dated snapshot ships.
    assert!(resolve_model("claude-opus-4-7-20260101").is_none());

    // No `claude-haiku-4-6` base row exists; fallback can only match
    // existing rows.
    assert!(resolve_model("claude-haiku-4-6-20260201").is_none());

    // 9-char input — length guard in strip_dated_suffix returns early
    // because bytes.len() <= 9.
    assert!(resolve_model("-12345678").is_none());

    // 10-char input, 1-char prefix `a`. Length guard accepts but
    // lookup_exact("a") misses, so resolve returns None.
    assert!(resolve_model("a-12345678").is_none());
}

#[test]
fn unknown_model_is_rejected() {
    assert!(resolve_model("nope/nope").is_none());
    assert!(resolve_model("anthropic/claude-sonnet-4-7").is_none());
    assert!(resolve_model("claude-sonnet-4-7").is_none());
}

#[test]
fn known_model_list_is_stable_and_resolvable() {
    assert!(KNOWN_MODELS
        .iter()
        .any(|model| model.id == "anthropic/claude-sonnet-4-6"));
    assert!(KNOWN_MODELS
        .iter()
        .any(|model| model.id == "claude-sonnet-4-6-max"));
    assert!(KNOWN_MODELS
        .iter()
        .any(|model| model.id == "openai:gpt-5.5"));
    assert!(KNOWN_MODELS.iter().any(|model| model.id == "gpt-image-2"));

    for model in KNOWN_MODELS {
        assert_eq!(
            resolve_model(model.id).unwrap().upstream_model,
            model.upstream_model
        );
    }
}

#[test]
fn every_claude_alias_routes_to_bedrock() {
    for model in KNOWN_MODELS {
        if model.id.contains("claude") || model.id.starts_with("anthropic/") {
            let alias = resolve_model(model.id).unwrap();
            assert_eq!(alias.provider, Provider::Bedrock, "{}", model.id);
            assert!(
                alias.upstream_model.starts_with("anthropic.claude-")
                    || alias.upstream_model.starts_with("us.anthropic.claude-"),
                "{} -> {}",
                model.id,
                alias.upstream_model
            );
        }
    }
}

#[test]
fn chat_route_helper_selects_gpt_vs_anthropic() {
    let gpt = serde_json::json!({ "model": "openai:gpt-5.5", "messages": [] });
    assert_eq!(
        route_for_chat_model(&gpt).unwrap(),
        ChatRoute::CodexResponses
    );

    let anthropic = serde_json::json!({
        "model": "claude-sonnet-4-6",
        "messages": []
    });
    assert_eq!(
        route_for_chat_model(&anthropic).unwrap(),
        ChatRoute::BedrockMessages
    );

    let google = serde_json::json!({ "model": "gemini-3-flash-preview", "messages": [] });
    assert!(route_for_chat_model(&google).is_err());
}

#[test]
fn messages_route_helper_routes_anthropic_shape_to_bedrock() {
    let bare = serde_json::json!({
        "model": "claude-sonnet-4-6",
        "messages": []
    });
    assert_eq!(
        route_for_messages_model(&bare).unwrap(),
        MessagesRoute::BedrockMessages
    );

    let prefixed = serde_json::json!({
        "model": "anthropic/claude-opus-4-6",
        "messages": []
    });
    assert_eq!(
        route_for_messages_model(&prefixed).unwrap(),
        MessagesRoute::BedrockMessages
    );
}

#[test]
fn responses_route_helper_selects_supported_provider_routes() {
    let gpt = serde_json::json!({ "model": "openai:gpt-5.5", "input": "hello" });
    assert_eq!(
        route_for_responses_model(&gpt).unwrap(),
        ResponsesRoute::CodexResponses
    );

    let sonnet = serde_json::json!({ "model": "claude-sonnet-4-6", "input": "hello" });
    assert_eq!(
        route_for_responses_model(&sonnet).unwrap(),
        ResponsesRoute::BedrockMessages
    );

    let prefixed_sonnet =
        serde_json::json!({ "model": "anthropic/claude-sonnet-4-6", "input": "hello" });
    assert_eq!(
        route_for_responses_model(&prefixed_sonnet).unwrap(),
        ResponsesRoute::BedrockMessages
    );

    let sonnet_max = serde_json::json!({ "model": "claude-sonnet-4-6-max", "input": "hello" });
    assert_eq!(
        route_for_responses_model(&sonnet_max).unwrap(),
        ResponsesRoute::BedrockMessages
    );

    let opus = serde_json::json!({ "model": "claude-opus-4-7", "input": "hello" });
    assert_eq!(
        route_for_responses_model(&opus).unwrap(),
        ResponsesRoute::BedrockMessages
    );

    let prefixed_opus =
        serde_json::json!({ "model": "anthropic/claude-opus-4-7", "input": "hello" });
    assert_eq!(
        route_for_responses_model(&prefixed_opus).unwrap(),
        ResponsesRoute::BedrockMessages
    );

    let gemini = serde_json::json!({ "model": "gemini-3.1-flash-lite", "input": "hello" });
    assert_eq!(
        route_for_responses_model(&gemini).unwrap(),
        ResponsesRoute::GoogleGenerateContent {
            upstream_model: "gemini-3.1-flash-lite".to_string()
        }
    );

    let unknown = serde_json::json!({ "model": "claude-sonnet-4-7", "input": "hello" });
    assert!(route_for_responses_model(&unknown).is_err());
}
