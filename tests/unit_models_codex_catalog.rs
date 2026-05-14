use unified_model_proxy_v2::{
    route::models::{codex_catalog_to_openai_models, codex_models_endpoint},
    AppError,
};

fn ids(body: &serde_json::Value) -> Vec<&str> {
    body["data"]
        .as_array()
        .unwrap()
        .iter()
        .map(|model| model["id"].as_str().unwrap())
        .collect()
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
            "gpt-5.5",
            "gpt-5.4",
            "gpt-5.4-mini",
            "gpt-5.3-codex",
            "gpt-5.2"
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
            "gpt-5.5",
            "gpt-5.4",
            "gpt-5.4-mini",
            "gpt-5.3-codex",
            "gpt-5.2",
            "codex-auto-review"
        ]
    );
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
        vec!["gpt-5.4", "gpt-5.4-mini", "gpt-5.3-codex", "gpt-5.2"]
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
