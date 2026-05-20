use std::{env, fs, sync::Mutex};

use base64::Engine;
use tempfile::tempdir;
use unified_model_proxy_v2::{
    auth::{
        bedrock::{resolve_bedrock_auth, BedrockAuth},
        codex::refresh_codex_auth_with_endpoint,
        codex::{load_codex_auth, parse_codex_auth, CODEX_ORIGINATOR},
        google::api_key as google_api_key,
        windsurf::api_key as windsurf_api_key,
    },
    AppError, AppState,
};
use wiremock::{
    matchers::{method, path},
    Mock, MockServer, ResponseTemplate,
};

static ENV_LOCK: Mutex<()> = Mutex::new(());

#[test]
fn codex_parses_ump_shape_with_account_id_from_id_token() {
    let id_token = id_token_with_account("acct_from_token");
    let auth = parse_codex_auth(
        &serde_json::json!({
            "tokens": {
                "access_token": "access_ump",
                "refresh_token": "refresh_ump",
                "id_token": id_token
            }
        })
        .to_string(),
    )
    .unwrap();

    assert_eq!(CODEX_ORIGINATOR, "codex_cli_rs");
    assert_eq!(auth.access_token, "access_ump");
    assert_eq!(auth.refresh_token.as_deref(), Some("refresh_ump"));
    assert_eq!(auth.account_id.as_deref(), Some("acct_from_token"));
}

#[test]
fn codex_parses_shorthand_shape_and_prefers_explicit_account_id() {
    let auth = parse_codex_auth(
        &serde_json::json!({
            "access": "access_open",
            "refresh": "refresh_open",
            "id": id_token_with_account("acct_from_token"),
            "accountId": "acct_explicit"
        })
        .to_string(),
    )
    .unwrap();

    assert_eq!(auth.access_token, "access_open");
    assert_eq!(auth.refresh_token.as_deref(), Some("refresh_open"));
    assert_eq!(
        auth.id_token.as_deref(),
        Some(id_token_with_account("acct_from_token").as_str())
    );
    assert_eq!(auth.account_id.as_deref(), Some("acct_explicit"));
}

#[test]
fn codex_load_uses_test_home_override_and_fails_closed_when_missing() {
    let _guard = ENV_LOCK.lock().unwrap();
    let codex_home = tempdir().unwrap();
    let auth_home = tempdir().unwrap();
    with_env(
        "UMP_V2_CODEX_HOME",
        Some(codex_home.path().as_os_str()),
        || {
            with_env(
                "UMP_V2_AUTH_HOME",
                Some(auth_home.path().as_os_str()),
                || {
                    let state = AppState::from_env();
                    assert_eq!(state.codex_home, codex_home.path());
                    assert_eq!(state.auth_home, auth_home.path());

                    let missing = load_codex_auth(&state).unwrap_err();
                    assert!(matches!(
                        missing,
                        AppError::MissingCredential("~/.codex/auth.json")
                    ));

                    fs::write(
                codex_home.path().join("auth.json"),
                r#"{"access_token":"access_raw","refresh_token":"refresh_raw","account_id":"acct_raw"}"#,
            )
            .unwrap();
                    let loaded = load_codex_auth(&state).unwrap();
                    assert_eq!(loaded.access_token, "access_raw");
                    assert_eq!(loaded.account_id.as_deref(), Some("acct_raw"));
                },
            )
        },
    );
}

#[test]
fn home_and_codex_home_resolve_to_temp_roots_without_raw_dot_access() {
    let _guard = ENV_LOCK.lock().unwrap();
    let home = tempdir().unwrap();
    let codex_home = tempdir().unwrap();

    fs::write(
        codex_home.path().join("auth.json"),
        r#"{"access_token":"must-not-read-codex-home"}"#,
    )
    .unwrap();

    with_env("HOME", Some(home.path().as_os_str()), || {
        with_env("CODEX_HOME", Some(codex_home.path().as_os_str()), || {
            with_env("UMP_V2_CODEX_HOME", None, || {
                with_env("UMP_V2_AUTH_HOME", None, || {
                    let state = AppState::from_env();

                    assert!(state.codex_home.starts_with(home.path()));
                    assert!(state.auth_home.starts_with(home.path()));
                    assert_eq!(state.codex_home, home.path().join(".codex"));
                    assert_eq!(state.auth_home, home.path().join(".ump"));

                    let missing = load_codex_auth(&state).unwrap_err();
                    assert!(matches!(
                        missing,
                        AppError::MissingCredential("~/.codex/auth.json")
                    ));
                })
            })
        })
    });
}

#[test]
fn ump_v2_home_overrides_resolve_to_temp_roots_without_raw_dot_access() {
    let _guard = ENV_LOCK.lock().unwrap();
    let home = tempdir().unwrap();
    let codex_home = tempdir().unwrap();
    let auth_home = tempdir().unwrap();

    fs::create_dir_all(home.path().join(".codex")).unwrap();
    fs::create_dir_all(home.path().join(".ump")).unwrap();
    fs::write(
        home.path().join(".codex/auth.json"),
        r#"{"access_token":"must-not-read-home-codex"}"#,
    )
    .unwrap();
    fs::write(
        home.path().join(".ump/auth.json"),
        r#"{"codex":{"account_id":"must-not-read-home-ump"}}"#,
    )
    .unwrap();

    with_env("HOME", Some(home.path().as_os_str()), || {
        with_env(
            "CODEX_HOME",
            Some(home.path().join("ignored-codex-home").as_os_str()),
            || {
                with_env(
                    "UMP_V2_CODEX_HOME",
                    Some(codex_home.path().as_os_str()),
                    || {
                        with_env(
                            "UMP_V2_AUTH_HOME",
                            Some(auth_home.path().as_os_str()),
                            || {
                                let state = AppState::from_env();

                                assert_eq!(state.codex_home, codex_home.path());
                                assert_eq!(state.auth_home, auth_home.path());

                                let missing = load_codex_auth(&state).unwrap_err();
                                assert!(matches!(
                                    missing,
                                    AppError::MissingCredential("~/.codex/auth.json")
                                ));
                            },
                        )
                    },
                )
            },
        )
    });
}

#[tokio::test]
async fn codex_refresh_writes_codex_auth_and_diagnostic_mirror() {
    let codex_home = tempdir().unwrap();
    let auth_home = tempdir().unwrap();
    fs::write(
        codex_home.path().join("auth.json"),
        serde_json::json!({
            "access_token": "access-old",
            "refresh_token": "refresh-old",
            "account_id": "acct-old"
        })
        .to_string(),
    )
    .unwrap();

    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/oauth/token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "access_token": "access-new",
            "refresh_token": "refresh-new",
            "id_token": id_token_with_account("acct-new")
        })))
        .mount(&server)
        .await;

    let state = AppState::for_tests(
        codex_home.path().to_path_buf(),
        auth_home.path().to_path_buf(),
    );
    let refreshed = refresh_codex_auth_with_endpoint(
        &state.specter,
        &state,
        &format!("{}/oauth/token", server.uri()),
    )
    .await
    .unwrap();

    assert_eq!(refreshed.access_token, "access-new");
    assert_eq!(refreshed.refresh_token.as_deref(), Some("refresh-new"));
    assert_eq!(refreshed.account_id.as_deref(), Some("acct-new"));

    let codex_auth = load_codex_auth(&state).unwrap();
    assert_eq!(codex_auth.access_token, "access-new");
    assert_eq!(codex_auth.refresh_token.as_deref(), Some("refresh-new"));

    let mirror: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(auth_home.path().join("auth.json")).unwrap())
            .unwrap();
    assert_eq!(mirror["codex"]["account_id"], "acct-new");
    assert_eq!(mirror["codex"]["has_access_token"], true);
    assert_eq!(mirror["codex"]["has_refresh_token"], true);
    assert_eq!(mirror["codex"]["originator"], CODEX_ORIGINATOR);
    assert!(mirror["codex"].get("access_token").is_none());
}

#[test]
fn bedrock_resolves_env_then_auth_home_bearer_then_profile_and_fails_closed() {
    let _guard = ENV_LOCK.lock().unwrap();
    let codex_home = tempdir().unwrap();
    let auth_home = tempdir().unwrap();

    with_env(
        "UMP_V2_CODEX_HOME",
        Some(codex_home.path().as_os_str()),
        || {
            with_env(
                "UMP_V2_AUTH_HOME",
                Some(auth_home.path().as_os_str()),
                || {
                    with_env("AWS_BEARER_TOKEN_BEDROCK", None, || {
                        with_env(
                            "AWS_BEARER_TOKEN_BEDROCK",
                            Some("env_bearer".as_ref()),
                            || {
                                let state = AppState::from_env();
                                assert_eq!(
                                    resolve_bedrock_auth(&state).unwrap(),
                                    BedrockAuth::Bearer {
                                        token: "env_bearer".to_string(),
                                        source: "bearer_env"
                                    }
                                );
                            },
                        );

                        let state = AppState::from_env();
                        assert!(matches!(
                            resolve_bedrock_auth(&state).unwrap_err(),
                            AppError::MissingCredential("Bedrock bearer/profile")
                        ));

                        fs::write(
                            auth_home.path().join("auth.json"),
                            r#"{"bedrock":{"bearer":"file_bearer","profile":"ignored_profile"}}"#,
                        )
                        .unwrap();
                        assert_eq!(
                            resolve_bedrock_auth(&state).unwrap(),
                            BedrockAuth::Bearer {
                                token: "file_bearer".to_string(),
                                source: "bearer_file"
                            }
                        );

                        with_env(
                            "AWS_BEARER_TOKEN_BEDROCK",
                            Some("env_bearer".as_ref()),
                            || {
                                assert_eq!(
                                    resolve_bedrock_auth(&state).unwrap(),
                                    BedrockAuth::Bearer {
                                        token: "file_bearer".to_string(),
                                        source: "bearer_file"
                                    }
                                );
                            },
                        );

                        fs::write(
                            auth_home.path().join("auth.json"),
                            r#"{"bedrock":{"profile":"dev-profile"}}"#,
                        )
                        .unwrap();
                        assert_eq!(
                            resolve_bedrock_auth(&state).unwrap(),
                            BedrockAuth::Profile {
                                name: "dev-profile".to_string()
                            }
                        );
                    });
                },
            )
        },
    );
}

#[test]
fn google_resolves_auth_home_api_key_then_env_and_fails_closed_without_either() {
    let _guard = ENV_LOCK.lock().unwrap();
    let codex_home = tempdir().unwrap();
    let auth_home = tempdir().unwrap();

    with_env(
        "UMP_V2_CODEX_HOME",
        Some(codex_home.path().as_os_str()),
        || {
            with_env(
                "UMP_V2_AUTH_HOME",
                Some(auth_home.path().as_os_str()),
                || {
                    with_env("GOOGLE_API_KEY", None, || {
                        let state = AppState::from_env();
                        assert!(matches!(
                            google_api_key(&state).unwrap_err(),
                            AppError::MissingCredential("GOOGLE_API_KEY")
                        ));
                    });

                    fs::write(
                        auth_home.path().join("auth.json"),
                        r#"{"gemini":{"api_key":"file_google_key"}}"#,
                    )
                    .unwrap();

                    with_env("GOOGLE_API_KEY", None, || {
                        let state = AppState::from_env();
                        assert_eq!(google_api_key(&state).unwrap(), "file_google_key");
                    });

                    with_env("GOOGLE_API_KEY", Some("env_google_key".as_ref()), || {
                        let state = AppState::from_env();
                        assert_eq!(google_api_key(&state).unwrap(), "file_google_key");
                    });

                    fs::write(auth_home.path().join("auth.json"), r#"{"google":{}}"#).unwrap();

                    with_env("GOOGLE_API_KEY", Some("  ".as_ref()), || {
                        let state = AppState::from_env();
                        assert!(matches!(
                            google_api_key(&state).unwrap_err(),
                            AppError::MissingCredential("GOOGLE_API_KEY")
                        ));
                    });

                    with_env("GOOGLE_API_KEY", Some("env_google_key".as_ref()), || {
                        let state = AppState::from_env();
                        assert_eq!(google_api_key(&state).unwrap(), "env_google_key");
                    });
                },
            )
        },
    );
}

#[test]
fn windsurf_resolves_auth_home_api_key_legacy_file_then_env_and_fails_closed() {
    let _guard = ENV_LOCK.lock().unwrap();
    let codex_home = tempdir().unwrap();
    let auth_home = tempdir().unwrap();

    with_env(
        "UMP_V2_CODEX_HOME",
        Some(codex_home.path().as_os_str()),
        || {
            with_env(
                "UMP_V2_AUTH_HOME",
                Some(auth_home.path().as_os_str()),
                || {
                    with_env("WINDSURF_API_KEY", None, || {
                        let state = AppState::from_env();
                        assert!(matches!(
                            windsurf_api_key(&state).unwrap_err(),
                            AppError::MissingCredential("WINDSURF_API_KEY")
                        ));
                    });

                    fs::write(
                        auth_home.path().join("auth.json"),
                        r#"{"windsurf":{"api_key":"file_windsurf_key"}}"#,
                    )
                    .unwrap();

                    with_env(
                        "WINDSURF_API_KEY",
                        Some("env_windsurf_key".as_ref()),
                        || {
                            let state = AppState::from_env();
                            assert_eq!(windsurf_api_key(&state).unwrap(), "file_windsurf_key");
                        },
                    );

                    fs::write(auth_home.path().join("auth.json"), r#"{"windsurf":{}}"#).unwrap();
                    fs::create_dir_all(auth_home.path().join("windsurf")).unwrap();
                    fs::write(
                        auth_home.path().join("windsurf/auth.json"),
                        r#"{"apiKey":"legacy_windsurf_key"}"#,
                    )
                    .unwrap();

                    with_env("WINDSURF_API_KEY", None, || {
                        let state = AppState::from_env();
                        assert_eq!(windsurf_api_key(&state).unwrap(), "legacy_windsurf_key");
                    });

                    fs::remove_file(auth_home.path().join("windsurf/auth.json")).unwrap();
                    with_env(
                        "WINDSURF_API_KEY",
                        Some("env_windsurf_key".as_ref()),
                        || {
                            let state = AppState::from_env();
                            assert_eq!(windsurf_api_key(&state).unwrap(), "env_windsurf_key");
                        },
                    );
                },
            )
        },
    );
}

fn id_token_with_account(account_id: &str) -> String {
    let header = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(r#"{"alg":"none"}"#);
    let payload = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .encode(serde_json::json!({ "account_id": account_id }).to_string());
    format!("{header}.{payload}.")
}

fn with_env<F>(key: &str, value: Option<&std::ffi::OsStr>, f: F)
where
    F: FnOnce(),
{
    let previous = env::var_os(key);
    match value {
        Some(value) => env::set_var(key, value),
        None => env::remove_var(key),
    }
    f();
    match previous {
        Some(value) => env::set_var(key, value),
        None => env::remove_var(key),
    }
}
