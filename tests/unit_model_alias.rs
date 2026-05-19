use unified_model_proxy_v2::{
    error::AppResult,
    model_alias::{resolve_model, Provider, ResolvedModel, TargetFormat, KNOWN_MODELS},
    route::chat::{route_for_chat_model, route_for_chat_model_with_resolver, ChatRoute},
    route::messages::{
        route_for_messages_model, route_for_messages_model_with_resolver, MessagesRoute,
    },
    route::responses::{
        route_for_responses_model, route_for_responses_model_with_resolver, ResponsesRoute,
    },
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

    // Composer rows added in Phase 1: Cursor provider, upstream id == public id.
    let alias = resolve_model("composer-1.5").unwrap();
    assert_eq!(alias.provider, Provider::Cursor);
    assert_eq!(alias.upstream_model, "composer-1.5");

    let alias = resolve_model("composer-2").unwrap();
    assert_eq!(alias.provider, Provider::Cursor);
    assert_eq!(alias.upstream_model, "composer-2");

    let alias = resolve_model("composer-2-fast").unwrap();
    assert_eq!(alias.provider, Provider::Cursor);
    assert_eq!(alias.upstream_model, "composer-2-fast");

    let alias = resolve_model("swe-1.6").unwrap();
    assert_eq!(alias.provider, Provider::Windsurf);
    assert_eq!(alias.upstream_model, "swe-1-6");

    let alias = resolve_model("swe-1.6-fast").unwrap();
    assert_eq!(alias.provider, Provider::Windsurf);
    assert_eq!(alias.upstream_model, "swe-1-6-fast");

    let alias = resolve_model("windsurf/swe-1.5-fast").unwrap();
    assert_eq!(alias.provider, Provider::Windsurf);
    assert_eq!(alias.upstream_model, "swe-1-5");

    let alias = resolve_model("adaptive").unwrap();
    assert_eq!(alias.provider, Provider::Windsurf);
    assert_eq!(alias.upstream_model, "adaptive");
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
fn provider_defaults_select_expected_target_formats() {
    assert_eq!(
        Provider::Bedrock.default_target_format(),
        Some(TargetFormat::AnthropicMessages)
    );
    assert_eq!(
        Provider::Codex.default_target_format(),
        Some(TargetFormat::Responses)
    );
    assert_eq!(
        Provider::Google.default_target_format(),
        Some(TargetFormat::GoogleGenerateContent)
    );
    assert_eq!(
        Provider::Cursor.default_target_format(),
        Some(TargetFormat::CursorAgent)
    );
    assert_eq!(
        Provider::Windsurf.default_target_format(),
        Some(TargetFormat::WindsurfChat)
    );
    assert_eq!(Provider::Unsupported.default_target_format(), None);
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
fn chat_route_planner_allows_only_codex_gpt_and_bedrock_claude() {
    for (provider, upstream_model, expected) in [
        (Provider::Codex, "gpt-5.5", Ok(ChatRoute::CodexResponses)),
        (
            Provider::Bedrock,
            "us.anthropic.claude-sonnet-4-6",
            Ok(ChatRoute::BedrockMessages),
        ),
        (
            Provider::Google,
            "gemini-3.1-flash-lite",
            Err("model_not_supported"),
        ),
        (
            Provider::Cursor,
            "composer-2-fast",
            Ok(ChatRoute::CursorAgent {
                upstream_model: "composer-2-fast".to_string(),
            }),
        ),
        (
            Provider::Windsurf,
            "swe-1-6",
            Ok(ChatRoute::WindsurfChat {
                upstream_model: "swe-1-6".to_string(),
            }),
        ),
        (
            Provider::Unsupported,
            "gpt-image-2",
            Err("model_not_supported"),
        ),
    ] {
        let body = serde_json::json!({ "model": "planner-fixture", "messages": [] });
        let route =
            route_for_chat_model_with_resolver(&body, |_| Ok(resolved(provider, upstream_model)));

        assert_route_result(route, expected, provider, upstream_model);
    }
}

#[test]
fn messages_route_planner_allows_only_bedrock_and_codex_gpt() {
    for (provider, upstream_model, expected) in [
        (
            Provider::Bedrock,
            "us.anthropic.claude-sonnet-4-6",
            Ok(MessagesRoute::BedrockMessages),
        ),
        (
            Provider::Codex,
            "gpt-5.5",
            Ok(MessagesRoute::CodexResponses),
        ),
        (
            Provider::Google,
            "gemini-3.1-flash-lite",
            Err("model_not_supported"),
        ),
        (
            Provider::Cursor,
            "composer-2",
            Ok(MessagesRoute::CursorAgent {
                upstream_model: "composer-2".to_string(),
            }),
        ),
        (Provider::Windsurf, "swe-1-6", Err("model_not_supported")),
        (
            Provider::Unsupported,
            "gpt-image-2",
            Err("model_not_supported"),
        ),
    ] {
        let body = serde_json::json!({ "model": "planner-fixture", "messages": [] });
        let route = route_for_messages_model_with_resolver(&body, |_| {
            Ok(resolved(provider, upstream_model))
        });

        assert_route_result(route, expected, provider, upstream_model);
    }
}

#[test]
fn responses_route_planner_allows_codex_bedrock_and_google_only() {
    for (provider, upstream_model, expected) in [
        (
            Provider::Codex,
            "gpt-5.5",
            Ok(ResponsesRoute::CodexResponses),
        ),
        (
            Provider::Bedrock,
            "us.anthropic.claude-sonnet-4-6",
            Ok(ResponsesRoute::BedrockMessages),
        ),
        (
            Provider::Google,
            "gemini-3.1-flash-lite",
            Ok(ResponsesRoute::GoogleGenerateContent {
                upstream_model: "gemini-3.1-flash-lite".to_string(),
            }),
        ),
        (
            Provider::Cursor,
            "composer-1.5",
            Ok(ResponsesRoute::CursorAgent {
                upstream_model: "composer-1.5".to_string(),
            }),
        ),
        (
            Provider::Windsurf,
            "swe-1-6",
            Ok(ResponsesRoute::WindsurfChat {
                upstream_model: "swe-1-6".to_string(),
            }),
        ),
        (
            Provider::Unsupported,
            "gpt-image-2",
            Err("model_not_supported"),
        ),
    ] {
        let body = serde_json::json!({ "model": "planner-fixture", "input": "hello" });
        let route = route_for_responses_model_with_resolver(&body, |_| {
            Ok(resolved(provider, upstream_model))
        });

        assert_route_result(route, expected, provider, upstream_model);
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

    let swe = serde_json::json!({ "model": "swe-1.6", "input": "hello" });
    assert_eq!(
        route_for_responses_model(&swe).unwrap(),
        ResponsesRoute::WindsurfChat {
            upstream_model: "swe-1-6".to_string()
        }
    );
}

#[test]
fn known_model_catalog_exposes_provider_aware_remote_compaction_policy() {
    for model in KNOWN_MODELS {
        let value = serde_json::to_value(model).unwrap();
        let policy = value
            .get("remote_compaction_policy")
            .and_then(|value| value.as_str())
            .unwrap_or_else(|| panic!("{} missing remote_compaction_policy", model.id));
        let expected = match model.provider {
            Provider::Codex => "native",
            Provider::Bedrock | Provider::Google | Provider::Cursor | Provider::Windsurf => "local",
            Provider::Unsupported => "off",
        };

        assert_eq!(policy, expected, "{}", model.id);
    }
}

fn resolved(provider: Provider, upstream_model: &str) -> ResolvedModel {
    ResolvedModel {
        provider,
        upstream_model: upstream_model.to_string(),
    }
}

fn assert_route_result<T>(
    actual: AppResult<T>,
    expected: Result<T, &str>,
    provider: Provider,
    upstream_model: &str,
) where
    T: std::fmt::Debug + Eq,
{
    match expected {
        Ok(expected) => assert_eq!(actual.unwrap(), expected, "{provider:?} {upstream_model}"),
        Err(expected_code) => {
            let error = actual.unwrap_err();
            assert_eq!(
                error.code(),
                Some(expected_code),
                "{provider:?} {upstream_model}: {error}"
            );
        }
    }
}
