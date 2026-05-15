//! Provider-aware compaction inspection and UMP-owned pack seams.

mod limits;
mod pack;
mod policy;
mod render;

pub use crate::error::CompactionHttpError;
pub use limits::CompactionLimits;
pub use pack::{
    decode_deterministic_ump_pack, decode_ump_pack, deterministic_ump_pack,
    encode_ump_pack_from_env, is_ump_compaction_marker, load_pack_keys_from_env,
    CompactVisibleContext, CompactionPackContext, DecodedUmpPack, UMP_COMPACTION_SCHEMA,
};
pub use policy::{CompactionPolicy, RemoteCompactionPolicy};
pub use render::render_ump_pack_for_target;

use axum::http::{HeaderMap, StatusCode};
use serde_json::Value;

use crate::{
    model_alias::{Provider, ResolvedTarget},
    AppResult,
};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum CompactionItemKind {
    Compaction,
    ContextCompaction,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct CompactionCarrier {
    pub index: usize,
    pub kind: CompactionItemKind,
    pub encrypted_content: Option<String>,
    pub is_ump_pack: bool,
}

impl CompactionCarrier {
    fn from_item(index: usize, item: &Value) -> Option<Self> {
        let object = item.as_object()?;
        let kind = match object.get("type").and_then(Value::as_str)? {
            "compaction" => CompactionItemKind::Compaction,
            "context_compaction" => CompactionItemKind::ContextCompaction,
            _ => return None,
        };
        let encrypted_content = object
            .get("encrypted_content")
            .and_then(Value::as_str)
            .map(ToOwned::to_owned);
        let is_ump_pack = encrypted_content
            .as_deref()
            .is_some_and(is_ump_compaction_marker);
        Some(Self {
            index,
            kind,
            encrypted_content,
            is_ump_pack,
        })
    }
}

pub fn find_compaction_carriers(input: &Value) -> Vec<CompactionCarrier> {
    let Value::Array(items) = input else {
        return Vec::new();
    };
    items
        .iter()
        .enumerate()
        .filter_map(|(index, item)| CompactionCarrier::from_item(index, item))
        .collect()
}

pub fn validate_compaction_carriers(
    input: &Value,
    target: &ResolvedTarget,
    limits: CompactionLimits,
) -> AppResult<Vec<CompactionCarrier>> {
    let carriers = find_compaction_carriers(input);
    limits.check_carrier_count(carriers.len())?;
    for carrier in &carriers {
        if let Some(encrypted_content) = carrier.encrypted_content.as_deref() {
            limits.check_encrypted_content(encrypted_content)?;
        }
        if !carrier.is_ump_pack && target.provider != Provider::Codex {
            return Err(CompactionHttpError::unsupported_item_for_target(target).into());
        }
    }
    Ok(carriers)
}

pub fn prepare_responses_input_for_target(
    request: &mut Value,
    target: &ResolvedTarget,
    limits: CompactionLimits,
    pack_context: Option<&CompactionPackContext>,
) -> AppResult<Vec<CompactionCarrier>> {
    let Some(input) = request.get("input") else {
        return Ok(Vec::new());
    };
    let carriers = validate_compaction_carriers(input, target, limits)?;
    if carriers.is_empty() {
        return Ok(carriers);
    }
    if target.provider == Provider::Codex && !carriers.iter().any(|carrier| carrier.is_ump_pack) {
        return Ok(carriers);
    }

    let Value::Array(items) = input else {
        return Ok(carriers);
    };
    let mut prepared = Vec::with_capacity(items.len());
    for (index, item) in items.iter().enumerate() {
        let Some(carrier) = carriers.iter().find(|carrier| carrier.index == index) else {
            prepared.push(item.clone());
            continue;
        };
        if carrier.is_ump_pack {
            let encrypted_content = carrier.encrypted_content.as_deref().ok_or_else(|| {
                CompactionHttpError::invalid_pack(
                    "UMP compaction item is missing encrypted_content",
                )
            })?;
            let pack_context = pack_context.ok_or_else(|| {
                CompactionHttpError::new(
                    StatusCode::BAD_REQUEST,
                    "compaction_binding_required",
                    "invalid_request",
                    "remote compaction requires a session binding",
                )
            })?;
            let pack = decode_ump_pack(encrypted_content, limits, pack_context)?;
            prepared.extend(render_ump_pack_for_target(&pack, target));
        } else {
            prepared.push(item.clone());
        }
    }
    request["input"] = Value::Array(prepared);
    Ok(carriers)
}

pub fn pack_context_from_headers(
    headers: &HeaderMap,
    route_binding: impl Into<String>,
    target: &ResolvedTarget,
) -> AppResult<CompactionPackContext> {
    let session_binding = session_binding_from_headers(headers)?;
    pack_context_for_session(session_binding, route_binding, target)
}

pub fn pack_context_for_session(
    session_binding: impl Into<String>,
    route_binding: impl Into<String>,
    target: &ResolvedTarget,
) -> AppResult<CompactionPackContext> {
    let instance_id = std::env::var("UMP_COMPACTION_INSTANCE_ID").map_err(|_| {
        CompactionHttpError::new(
            StatusCode::SERVICE_UNAVAILABLE,
            "proxy_compaction_unavailable",
            "server_error",
            "UMP_COMPACTION_INSTANCE_ID is required for proxy-visible compaction",
        )
    })?;
    Ok(CompactionPackContext {
        auth_subject: format!("local-no-auth:{instance_id}"),
        session_binding: session_binding.into(),
        route_binding: route_binding.into(),
        target_provider: provider_name(target.provider).to_string(),
        target_format: target.target_format.as_str().to_string(),
        target_model: target.upstream_model.clone(),
    })
}

fn session_binding_from_headers(headers: &HeaderMap) -> AppResult<String> {
    let session_id = header_string(headers, "session-id");
    let thread_id = header_string(headers, "thread-id");
    if let (Some(session_id), Some(thread_id)) = (session_id, thread_id) {
        if !session_id.trim().is_empty() && !thread_id.trim().is_empty() {
            return Ok(format!("session-id:{session_id}:thread-id:{thread_id}"));
        }
    }
    if let Some(session) =
        header_string(headers, "x-ump-compaction-session").filter(|value| !value.trim().is_empty())
    {
        return Ok(format!("x-ump-compaction-session:{session}"));
    }
    Err(CompactionHttpError::new(
        StatusCode::BAD_REQUEST,
        "compaction_binding_required",
        "invalid_request",
        "remote compaction requires session-id/thread-id or x-ump-compaction-session",
    )
    .into())
}

fn header_string(headers: &HeaderMap, name: &'static str) -> Option<String> {
    headers
        .get(name)
        .and_then(|value| value.to_str().ok())
        .map(str::to_owned)
}

fn provider_name(provider: Provider) -> &'static str {
    match provider {
        Provider::Bedrock => "bedrock",
        Provider::Codex => "codex",
        Provider::Google => "google",
        Provider::Cursor => "cursor",
        Provider::Unsupported => "unsupported",
    }
}

#[cfg(test)]
mod tests {
    use std::{ffi::OsString, sync::Mutex};

    use serde_json::json;

    use super::*;
    use crate::model_alias::TargetFormat;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    struct EnvRestore {
        key: &'static str,
        previous: Option<OsString>,
    }

    impl EnvRestore {
        fn set(key: &'static str, value: &str) -> Self {
            let previous = std::env::var_os(key);
            std::env::set_var(key, value);
            Self { key, previous }
        }
    }

    impl Drop for EnvRestore {
        fn drop(&mut self) {
            match &self.previous {
                Some(value) => std::env::set_var(self.key, value),
                None => std::env::remove_var(self.key),
            }
        }
    }

    fn bedrock_target() -> ResolvedTarget {
        ResolvedTarget {
            provider: Provider::Bedrock,
            upstream_model: "anthropic.claude-opus-4-1".into(),
            target_format: TargetFormat::AnthropicMessages,
        }
    }

    fn codex_target() -> ResolvedTarget {
        ResolvedTarget {
            provider: Provider::Codex,
            upstream_model: "gpt-5.5".into(),
            target_format: TargetFormat::Responses,
        }
    }

    #[test]
    fn detects_compaction_carriers() {
        let input = json!([
            {"role": "user", "content": "hi"},
            {"type": "context_compaction"},
            {"type": "compaction", "encrypted_content": "ump.compaction.v1.a.b.c"}
        ]);

        let carriers = find_compaction_carriers(&input);

        assert_eq!(carriers.len(), 2);
        assert_eq!(carriers[0].kind, CompactionItemKind::ContextCompaction);
        assert!(carriers[1].is_ump_pack);
    }

    #[test]
    fn non_codex_native_compaction_fails_before_adapter() {
        let input = json!([
            {"type": "compaction", "encrypted_content": "opaque-provider-native"}
        ]);

        let error =
            validate_compaction_carriers(&input, &bedrock_target(), CompactionLimits::default())
                .unwrap_err();

        assert_eq!(error.status(), axum::http::StatusCode::BAD_REQUEST);
        assert_eq!(error.code(), Some("unsupported_compaction_item_for_target"));
    }

    #[test]
    fn codex_native_compaction_passes_through() {
        let mut request = json!({
            "model": "gpt-5.5",
            "input": [{"type": "compaction", "encrypted_content": "opaque"}]
        });

        let context = test_context(&codex_target());
        prepare_responses_input_for_target(
            &mut request,
            &codex_target(),
            CompactionLimits::default(),
            Some(&context),
        )
        .unwrap();

        assert_eq!(request["input"][0]["type"], "compaction");
    }

    #[test]
    fn ump_pack_expands_for_non_codex_target() {
        let _guard = ENV_LOCK.lock().unwrap();
        let _key_env = EnvRestore::set(
            "UMP_COMPACTION_KEYS_JSON",
            r#"{"current":"fixture","keys":{"fixture":"AAECAwQFBgcICQoLDA0ODxAREhMUFRYXGBkaGxwdHh8"}}"#,
        );
        let target = bedrock_target();
        let context = test_context(&target);
        let pack = deterministic_ump_pack(
            CompactVisibleContext {
                task_objective: Some("keep working".into()),
                durable_constraints: vec!["do not leak secrets".into()],
                summary: Some("prior visible summary".into()),
                context_degraded: true,
            },
            &context,
        )
        .unwrap();
        let mut request = json!({
            "model": "claude",
            "input": [{"type": "compaction", "encrypted_content": pack}]
        });

        prepare_responses_input_for_target(
            &mut request,
            &target,
            CompactionLimits::default(),
            Some(&context),
        )
        .unwrap();

        assert_eq!(request["input"][0]["type"], "message");
        assert_eq!(request["input"][0]["role"], "system");
        assert!(request["input"][0]["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("context_degraded"));
    }

    #[test]
    fn too_many_carriers_fails() {
        let input = json!([
            {"type": "context_compaction"},
            {"type": "compaction", "encrypted_content": "opaque"}
        ]);

        let error =
            validate_compaction_carriers(&input, &codex_target(), CompactionLimits::default())
                .unwrap_err();

        assert_eq!(error.code(), Some("too_many_compaction_items"));
    }

    fn test_context(target: &ResolvedTarget) -> CompactionPackContext {
        CompactionPackContext {
            auth_subject: "local-no-auth:test".into(),
            session_binding: "session:test".into(),
            route_binding: "POST /v1/responses".into(),
            target_provider: match target.provider {
                Provider::Bedrock => "bedrock",
                Provider::Codex => "codex",
                Provider::Google => "google",
                Provider::Cursor => "cursor",
                Provider::Unsupported => "unsupported",
            }
            .into(),
            target_format: target.target_format.as_str().into(),
            target_model: target.upstream_model.clone(),
        }
    }
}
