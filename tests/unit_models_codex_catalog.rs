use std::{
    sync::atomic::{AtomicUsize, Ordering},
    time::Duration,
};

use unified_model_proxy_v2::{
    codex_catalog::{CodexCatalogCache, CodexCatalogConfig, DEFAULT_CODEX_CLIENT_VERSION},
    route::models::{codex_catalog_to_openai_models, codex_models_endpoint},
    AppError,
};

fn ids(body: &serde_json::Value) -> Vec<&str> {
    let mut ids = body["data"]
        .as_array()
        .unwrap()
        .iter()
        .map(|model| model["id"].as_str().unwrap())
        .collect::<Vec<_>>();
    ids.sort_unstable();
    ids
}

fn catalog(model_ids: &[&str]) -> serde_json::Value {
    serde_json::json!({
        "models": model_ids
            .iter()
            .map(|id| {
                serde_json::json!({
                    "slug": id,
                    "display_name": id,
                    "visibility": if *id == "codex-auto-review" { "hidden" } else { "list" },
                    "supported_in_api": true,
                })
            })
            .collect::<Vec<_>>()
    })
}

fn unsupported_catalog(model_ids: &[&str]) -> serde_json::Value {
    serde_json::json!({
        "models": model_ids
            .iter()
            .map(|id| {
                serde_json::json!({
                    "slug": id,
                    "display_name": id,
                    "visibility": "list",
                    "supported_in_api": false,
                })
            })
            .collect::<Vec<_>>()
    })
}

#[test]
fn current_codex_catalog_maps_visible_models_and_hides_auto_review_by_default() {
    let upstream = catalog(&[
        "gpt-5.5",
        "gpt-5.4",
        "gpt-5.4-mini",
        "gpt-5.3-codex",
        "gpt-5.2",
        "codex-auto-review",
    ]);

    let public = codex_catalog_to_openai_models(Some("26.506.31421"), &upstream, false).unwrap();
    assert_eq!(
        ids(&public),
        vec![
            "gpt-5.2",
            "gpt-5.3-codex",
            "gpt-5.4",
            "gpt-5.4-mini",
            "gpt-5.5"
        ]
    );
    assert_eq!(public["object"], "list");
    assert!(public["data"]
        .as_array()
        .unwrap()
        .iter()
        .all(|model| model["object"] == "model" && model["owned_by"] == "codex"));

    let internal = codex_catalog_to_openai_models(Some("26.506.31421"), &upstream, true).unwrap();
    assert_eq!(
        ids(&internal),
        vec![
            "codex-auto-review",
            "gpt-5.2",
            "gpt-5.3-codex",
            "gpt-5.4",
            "gpt-5.4-mini",
            "gpt-5.5"
        ]
    );
}

#[test]
fn codex_catalog_filters_hidden_and_unsupported_models_fail_closed() {
    let upstream = serde_json::json!({
        "models": [
            {
                "slug": "gpt-5.5",
                "display_name": "gpt-5.5",
                "visibility": "list",
                "supported_in_api": true
            },
            {
                "slug": "gpt-5.4",
                "display_name": "gpt-5.4",
                "visibility": "hidden",
                "supported_in_api": true
            },
            {
                "slug": "gpt-5.3-codex",
                "display_name": "gpt-5.3-codex",
                "visibility": "list",
                "supported_in_api": false
            },
            {
                "slug": "codex-auto-review",
                "display_name": "codex-auto-review",
                "visibility": "list",
                "supported_in_api": true
            }
        ]
    });

    let public = codex_catalog_to_openai_models(Some("26.506.31421"), &upstream, false).unwrap();
    assert_eq!(ids(&public), vec!["gpt-5.5"]);

    let internal = codex_catalog_to_openai_models(Some("26.506.31421"), &upstream, true).unwrap();
    assert_eq!(
        ids(&internal),
        vec!["codex-auto-review", "gpt-5.4", "gpt-5.5"]
    );
}

#[test]
fn codex_catalog_parses_current_codex_object_capability_shapes() {
    let upstream = serde_json::json!({
        "models": [{
            "slug": "gpt-5.5",
            "display_name": "gpt-5.5",
            "visibility": "list",
            "supported_in_api": true,
            "supported_reasoning_levels": [
                { "effort": "low", "description": "Low" },
                { "effort": "medium", "description": "Medium" },
                { "effort": "high", "description": "High" },
                { "effort": "xhigh", "description": "XHigh" }
            ],
            "service_tiers": [
                { "id": "auto", "name": "Auto", "description": "Default" },
                { "id": "priority", "name": "Priority", "description": "Fast lane" }
            ],
            "support_verbosity": true,
            "truncation_policy": { "mode": "tokens", "limit": 100000 },
            "input_modalities": ["text", "image"],
            "output_modalities": ["text"]
        }]
    });

    let catalog =
        unified_model_proxy_v2::codex_catalog::CodexCatalog::parse("26.506.31421", &upstream)
            .unwrap();
    let model = catalog.model("gpt-5.5").unwrap();

    assert_eq!(
        model.reasoning_levels,
        vec!["low", "medium", "high", "xhigh"]
    );
    assert_eq!(model.service_tiers, vec!["auto", "priority"]);
    assert_eq!(model.verbosity, vec!["low", "medium", "high"]);
    assert_eq!(model.truncation_policy, vec!["tokens"]);
}

#[test]
fn codex_catalog_rejects_requested_modalities_when_allowlists_are_absent() {
    let upstream = serde_json::json!({
        "models": [{
            "slug": "gpt-5.5",
            "display_name": "gpt-5.5",
            "visibility": "list",
            "supported_in_api": true
        }]
    });
    let catalog =
        unified_model_proxy_v2::codex_catalog::CodexCatalog::parse("26.506.31421", &upstream)
            .unwrap();

    for request in [
        unified_model_proxy_v2::codex_catalog::CodexCatalogRequest {
            model: "gpt-5.5",
            input_modalities: &["image"],
            ..Default::default()
        },
        unified_model_proxy_v2::codex_catalog::CodexCatalogRequest {
            model: "gpt-5.5",
            output_modalities: &["audio"],
            ..Default::default()
        },
    ] {
        assert!(catalog.validate_request(request).is_err());
    }
}

#[test]
fn codex_catalog_absent_or_empty_models_fail_closed() {
    for catalog in [
        serde_json::json!({}),
        serde_json::json!({ "models": null }),
        serde_json::json!({ "models": "not-an-array" }),
    ] {
        let error =
            codex_catalog_to_openai_models(Some("26.506.31421"), &catalog, false).unwrap_err();
        assert!(matches!(error, AppError::BadRequest(_)));
        assert!(error.to_string().contains("missing models"));
    }

    let public = codex_catalog_to_openai_models(
        Some("26.506.31421"),
        &unsupported_catalog(&["gpt-5.5", "gpt-5.4"]),
        false,
    )
    .unwrap();
    assert!(ids(&public).is_empty());
}

#[tokio::test]
async fn codex_catalog_cache_reuses_fresh_catalog_without_upstream() {
    let cache = CodexCatalogCache::new(CodexCatalogConfig {
        client_version: DEFAULT_CODEX_CLIENT_VERSION.to_string(),
        ttl: Duration::from_secs(60),
    });
    let calls = AtomicUsize::new(0);

    let first = cache
        .get_or_refresh_with(|_| async {
            calls.fetch_add(1, Ordering::SeqCst);
            Ok(catalog(&["gpt-5.5"]))
        })
        .await
        .unwrap();
    let second = cache
        .get_or_refresh_with(|_| async {
            calls.fetch_add(1, Ordering::SeqCst);
            Err(AppError::Upstream(
                "fresh cache should avoid upstream".into(),
            ))
        })
        .await
        .unwrap();

    assert_eq!(first, second);
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    assert!(second.model("gpt-5.5").is_some());
}

#[tokio::test]
async fn codex_catalog_cache_expired_entry_fails_closed_on_upstream_failure() {
    let cache = CodexCatalogCache::new(CodexCatalogConfig {
        client_version: DEFAULT_CODEX_CLIENT_VERSION.to_string(),
        ttl: Duration::ZERO,
    });
    cache.store_validated(&catalog(&["gpt-5.5"])).unwrap();

    let error = cache
        .get_or_refresh_with(|_| async { Err(AppError::Upstream("catalog unavailable".into())) })
        .await
        .unwrap_err();

    assert!(error.to_string().contains("catalog unavailable"));
}

#[tokio::test]
async fn codex_catalog_cache_refreshes_expired_entry_with_new_catalog() {
    let cache = CodexCatalogCache::new(CodexCatalogConfig {
        client_version: DEFAULT_CODEX_CLIENT_VERSION.to_string(),
        ttl: Duration::ZERO,
    });
    cache.store_validated(&catalog(&["gpt-5.5"])).unwrap();

    let refreshed = cache
        .get_or_refresh_with(|_| async { Ok(catalog(&["gpt-5.6"])) })
        .await
        .unwrap();

    assert!(refreshed.model("gpt-5.5").is_none());
    assert!(refreshed.model("gpt-5.6").is_some());
}

#[test]
fn old_codex_catalog_omits_gpt_5_5() {
    let upstream = catalog(&[
        "gpt-5.4",
        "gpt-5.4-mini",
        "gpt-5.3-codex",
        "gpt-5.2",
        "codex-auto-review",
    ]);

    let public = codex_catalog_to_openai_models(Some("0.99.0"), &upstream, false).unwrap();
    let ids = ids(&public);
    assert!(!ids.contains(&"gpt-5.5"));
    assert_eq!(
        ids,
        vec!["gpt-5.2", "gpt-5.3-codex", "gpt-5.4", "gpt-5.4-mini"]
    );
}

#[test]
fn codex_catalog_requires_client_version() {
    let upstream = catalog(&["gpt-5.5"]);

    let missing = codex_catalog_to_openai_models(None, &upstream, false).unwrap_err();
    assert!(matches!(missing, AppError::BadRequest(_)));
    assert!(missing.to_string().contains("client_version"));

    let blank = codex_catalog_to_openai_models(Some("   "), &upstream, false).unwrap_err();
    assert!(matches!(blank, AppError::BadRequest(_)));
    assert!(blank.to_string().contains("client_version"));

    let query_injection =
        codex_catalog_to_openai_models(Some("26.506.31421&foo=bar"), &upstream, false).unwrap_err();
    assert!(matches!(query_injection, AppError::BadRequest(_)));
}

#[test]
fn codex_models_endpoint_appends_required_client_version_query() {
    assert_eq!(
        codex_models_endpoint(
            "https://chatgpt.com/backend-api/codex/models",
            Some("26.506.31421")
        )
        .unwrap(),
        "https://chatgpt.com/backend-api/codex/models?client_version=26.506.31421"
    );
    assert_eq!(
        codex_models_endpoint("https://example.test/models?foo=bar", Some("0.99.0")).unwrap(),
        "https://example.test/models?foo=bar&client_version=0.99.0"
    );
    assert!(codex_models_endpoint("https://example.test/models", None).is_err());
}
