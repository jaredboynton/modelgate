use std::{
    collections::BTreeMap,
    env,
    time::{SystemTime, UNIX_EPOCH},
};

use aes_gcm_siv::{
    aead::{Aead, KeyInit, Payload},
    Aes256GcmSiv, Nonce,
};
use axum::http::StatusCode;
use base64::{
    engine::general_purpose::{STANDARD, STANDARD_NO_PAD, URL_SAFE_NO_PAD},
    Engine as _,
};
use hkdf::Hkdf;
use hmac::{Hmac, Mac};
use rand_core::{OsRng, RngCore};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::Sha256;

use crate::{compaction::CompactionLimits, error::CompactionHttpError, AppResult};

pub const UMP_COMPACTION_SCHEMA: &str = "ump.compaction.v1";
const UMP_COMPACTION_PREFIX: &str = "ump.compaction.v1.";
const DEFAULT_KEYS_ENV: &str = "UMP_COMPACTION_KEYS_JSON";
const TEST_KID: &str = "fixture";
const TEST_ROOT_KEY: [u8; 32] = [
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25,
    26, 27, 28, 29, 30, 31,
];
const TEST_NONCE: [u8; 12] = *b"test-nonce-1";

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Clone, Deserialize, Eq, PartialEq, Serialize)]
pub struct CompactVisibleContext {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_objective: Option<String>,
    #[serde(default)]
    pub durable_constraints: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(default)]
    pub context_degraded: bool,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct DecodedUmpPack {
    pub protected_header: Value,
    pub visible: CompactVisibleContext,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct CompactionPackContext {
    pub auth_subject: String,
    pub session_binding: String,
    pub route_binding: String,
    pub target_provider: String,
    pub target_format: String,
    pub target_model: String,
}

#[derive(Debug, Deserialize)]
struct RawKeySet {
    current: String,
    keys: BTreeMap<String, String>,
}

#[derive(Debug, Clone)]
struct PackKeySet {
    current: String,
    keys: BTreeMap<String, [u8; 32]>,
}

pub fn is_ump_compaction_marker(encrypted_content: &str) -> bool {
    encrypted_content.starts_with(UMP_COMPACTION_PREFIX)
}

pub fn load_pack_keys_from_env() -> AppResult<()> {
    let _ = pack_keyset_from_env()?;
    Ok(())
}

pub fn deterministic_ump_pack(
    visible: CompactVisibleContext,
    context: &CompactionPackContext,
) -> AppResult<String> {
    let keyset = PackKeySet {
        current: TEST_KID.to_string(),
        keys: BTreeMap::from([(TEST_KID.to_string(), TEST_ROOT_KEY)]),
    };
    encode_ump_pack_with_keyset(visible, context, &keyset, Some(TEST_NONCE))
}

pub fn encode_ump_pack_from_env(
    visible: CompactVisibleContext,
    context: &CompactionPackContext,
) -> AppResult<String> {
    let keyset = pack_keyset_from_env()?;
    encode_ump_pack_with_keyset(visible, context, &keyset, None)
}

pub fn decode_ump_pack(
    encrypted_content: &str,
    limits: CompactionLimits,
    context: &CompactionPackContext,
) -> AppResult<DecodedUmpPack> {
    decode_ump_pack_with_keyset(encrypted_content, limits, context, &pack_keyset_from_env()?)
}

pub fn decode_deterministic_ump_pack(
    encrypted_content: &str,
    limits: CompactionLimits,
    context: &CompactionPackContext,
) -> AppResult<DecodedUmpPack> {
    let keyset = PackKeySet {
        current: TEST_KID.to_string(),
        keys: BTreeMap::from([(TEST_KID.to_string(), TEST_ROOT_KEY)]),
    };
    decode_ump_pack_with_keyset(encrypted_content, limits, context, &keyset)
}

fn encode_ump_pack_with_keyset(
    visible: CompactVisibleContext,
    context: &CompactionPackContext,
    keyset: &PackKeySet,
    nonce_override: Option<[u8; 12]>,
) -> AppResult<String> {
    validate_pack_context(context)?;
    let root_key = keyset.keys.get(&keyset.current).ok_or_else(|| {
        CompactionHttpError::new(
            StatusCode::SERVICE_UNAVAILABLE,
            "proxy_compaction_unavailable",
            "server_error",
            "current UMP compaction key is not configured",
        )
    })?;
    let pack_id = format!("ucp_{}", uuid::Uuid::new_v4().simple());
    let commitment_key = derive_key(
        root_key,
        pack_id.as_bytes(),
        b"ump.compaction.v1.commitment",
    )?;
    let header = json!({
        "alg": "A256GCM-SIV",
        "schema": UMP_COMPACTION_SCHEMA,
        "kid": keyset.current,
        "created_at": unix_seconds_string(),
        "pack_id": pack_id,
        "policy": "proxy_visible_summary",
        "auth_binding": {
            "auth_subject": commitment(&commitment_key, &context.auth_subject)?,
            "session_binding": commitment(&commitment_key, &context.session_binding)?,
            "route_binding": commitment(&commitment_key, &context.route_binding)?,
        },
        "render_profile_version": 1,
        "target_compatibility": [{
            "provider": context.target_provider,
            "format": context.target_format,
            "model": context.target_model,
        }]
    });
    let header_bytes = serde_json::to_vec(&header)?;
    let payload = serde_json::to_vec(&json!({
        "schema_version": UMP_COMPACTION_SCHEMA,
        "portable_visible": visible,
    }))?;
    let encryption_key = derive_key(
        root_key,
        pack_id.as_bytes(),
        b"ump.compaction.v1.encryption",
    )?;
    let cipher = Aes256GcmSiv::new_from_slice(&encryption_key)
        .map_err(|_| CompactionHttpError::invalid_pack("failed to initialize compaction cipher"))?;
    let nonce_bytes = nonce_override.unwrap_or_else(random_nonce);
    let ciphertext = cipher
        .encrypt(
            Nonce::from_slice(&nonce_bytes),
            Payload {
                msg: &payload,
                aad: &header_bytes,
            },
        )
        .map_err(|_| CompactionHttpError::invalid_pack("failed to encrypt UMP pack"))?;
    Ok(format!(
        "{UMP_COMPACTION_PREFIX}{}.{}.{}",
        URL_SAFE_NO_PAD.encode(header_bytes),
        URL_SAFE_NO_PAD.encode(nonce_bytes),
        URL_SAFE_NO_PAD.encode(ciphertext)
    ))
}

fn decode_ump_pack_with_keyset(
    encrypted_content: &str,
    limits: CompactionLimits,
    context: &CompactionPackContext,
    keyset: &PackKeySet,
) -> AppResult<DecodedUmpPack> {
    validate_pack_context(context)?;
    if !is_ump_compaction_marker(encrypted_content) {
        return Err(CompactionHttpError::invalid_pack(
            "encrypted_content is not a UMP compaction pack",
        )
        .into());
    }
    limits.check_encrypted_content(encrypted_content)?;
    let suffix = encrypted_content
        .strip_prefix(UMP_COMPACTION_PREFIX)
        .expect("prefix checked");
    let mut parts = suffix.split('.');
    let header = parts.next();
    let nonce = parts.next();
    let ciphertext = parts.next();
    if header.is_none() || nonce.is_none() || ciphertext.is_none() || parts.next().is_some() {
        return Err(CompactionHttpError::invalid_pack(
            "UMP compaction pack must contain protected header, nonce, and ciphertext",
        )
        .into());
    }
    let header_bytes = decode_base64url(header.unwrap())?;
    let nonce_bytes = decode_base64url(nonce.unwrap())?;
    let ciphertext = decode_base64url(ciphertext.unwrap())?;
    if nonce_bytes.len() != 12 {
        return Err(CompactionHttpError::invalid_pack("UMP pack nonce must be 12 bytes").into());
    }
    let protected_header: Value = serde_json::from_slice(&header_bytes).map_err(|error| {
        CompactionHttpError::invalid_pack(format!("invalid UMP protected header: {error}"))
    })?;
    validate_header_shape(&protected_header)?;
    validate_target_compatibility(&protected_header, context)?;
    let kid = protected_header
        .get("kid")
        .and_then(Value::as_str)
        .ok_or_else(|| CompactionHttpError::invalid_pack("UMP pack missing kid"))?;
    let root_key = keyset.keys.get(kid).ok_or_else(|| {
        CompactionHttpError::new(
            StatusCode::BAD_REQUEST,
            "stale_ump_compaction_key",
            "invalid_request",
            "UMP compaction pack uses an unknown key id",
        )
    })?;
    let pack_id = protected_header
        .get("pack_id")
        .and_then(Value::as_str)
        .ok_or_else(|| CompactionHttpError::invalid_pack("UMP pack missing pack_id"))?;
    let commitment_key = derive_key(
        root_key,
        pack_id.as_bytes(),
        b"ump.compaction.v1.commitment",
    )?;
    validate_commitment(
        &protected_header,
        "auth_subject",
        &context.auth_subject,
        &commitment_key,
    )?;
    validate_commitment(
        &protected_header,
        "session_binding",
        &context.session_binding,
        &commitment_key,
    )?;
    validate_commitment(
        &protected_header,
        "route_binding",
        &context.route_binding,
        &commitment_key,
    )?;
    let encryption_key = derive_key(
        root_key,
        pack_id.as_bytes(),
        b"ump.compaction.v1.encryption",
    )?;
    let cipher = Aes256GcmSiv::new_from_slice(&encryption_key)
        .map_err(|_| CompactionHttpError::invalid_pack("failed to initialize compaction cipher"))?;
    let payload_bytes = cipher
        .decrypt(
            Nonce::from_slice(&nonce_bytes),
            Payload {
                msg: &ciphertext,
                aad: &header_bytes,
            },
        )
        .map_err(|_| CompactionHttpError::invalid_pack("UMP pack authentication failed"))?;
    limits.check_decrypted_pack(&payload_bytes)?;
    let payload: Value = serde_json::from_slice(&payload_bytes).map_err(|error| {
        CompactionHttpError::invalid_pack(format!("invalid UMP pack payload: {error}"))
    })?;
    if payload.get("schema_version").and_then(Value::as_str) != Some(UMP_COMPACTION_SCHEMA) {
        return Err(CompactionHttpError::unsupported_schema().into());
    }
    let visible = serde_json::from_value(
        payload
            .get("portable_visible")
            .cloned()
            .ok_or_else(|| CompactionHttpError::invalid_pack("missing portable_visible"))?,
    )
    .map_err(|error| {
        CompactionHttpError::invalid_pack(format!("invalid visible context: {error}"))
    })?;
    Ok(DecodedUmpPack {
        protected_header,
        visible,
    })
}

fn pack_keyset_from_env() -> AppResult<PackKeySet> {
    let raw = env::var(DEFAULT_KEYS_ENV).map_err(|_| {
        CompactionHttpError::new(
            StatusCode::SERVICE_UNAVAILABLE,
            "proxy_compaction_unavailable",
            "server_error",
            "UMP compaction keys are not configured",
        )
    })?;
    parse_pack_keyset(&raw)
}

fn parse_pack_keyset(raw: &str) -> AppResult<PackKeySet> {
    let raw: RawKeySet = serde_json::from_str(raw).map_err(|error| {
        CompactionHttpError::new(
            StatusCode::SERVICE_UNAVAILABLE,
            "proxy_compaction_unavailable",
            "server_error",
            format!("invalid UMP compaction keyset: {error}"),
        )
    })?;
    let mut keys = BTreeMap::new();
    for (kid, encoded) in raw.keys {
        let decoded = decode_base64_any(&encoded)?;
        let key: [u8; 32] = decoded.try_into().map_err(|_| {
            CompactionHttpError::new(
                StatusCode::SERVICE_UNAVAILABLE,
                "proxy_compaction_unavailable",
                "server_error",
                format!("UMP compaction key {kid} must decode to 32 bytes"),
            )
        })?;
        if keys.insert(kid.clone(), key).is_some() {
            return Err(CompactionHttpError::new(
                StatusCode::SERVICE_UNAVAILABLE,
                "proxy_compaction_unavailable",
                "server_error",
                format!("duplicate UMP compaction key id: {kid}"),
            )
            .into());
        }
    }
    if !keys.contains_key(&raw.current) {
        return Err(CompactionHttpError::new(
            StatusCode::SERVICE_UNAVAILABLE,
            "proxy_compaction_unavailable",
            "server_error",
            "current UMP compaction key is missing",
        )
        .into());
    }
    Ok(PackKeySet {
        current: raw.current,
        keys,
    })
}

fn validate_pack_context(context: &CompactionPackContext) -> AppResult<()> {
    if context.session_binding.trim().is_empty() {
        return Err(CompactionHttpError::new(
            StatusCode::BAD_REQUEST,
            "compaction_binding_required",
            "invalid_request",
            "remote compaction requires a non-empty session binding",
        )
        .into());
    }
    Ok(())
}

fn validate_header_shape(header: &Value) -> AppResult<()> {
    if header.get("schema").and_then(Value::as_str) != Some(UMP_COMPACTION_SCHEMA) {
        return Err(CompactionHttpError::unsupported_schema().into());
    }
    if header.get("alg").and_then(Value::as_str) != Some("A256GCM-SIV") {
        return Err(CompactionHttpError::invalid_pack("unsupported UMP pack algorithm").into());
    }
    if header.get("policy").and_then(Value::as_str) != Some("proxy_visible_summary") {
        return Err(CompactionHttpError::invalid_pack("unsupported UMP pack policy").into());
    }
    Ok(())
}

fn validate_target_compatibility(header: &Value, context: &CompactionPackContext) -> AppResult<()> {
    let Some(entries) = header.get("target_compatibility").and_then(Value::as_array) else {
        return Err(CompactionHttpError::invalid_pack("missing target_compatibility").into());
    };
    if entries.iter().any(|entry| {
        entry.get("provider").and_then(Value::as_str) == Some(context.target_provider.as_str())
            && entry.get("format").and_then(Value::as_str) == Some(context.target_format.as_str())
            && entry.get("model").and_then(Value::as_str) == Some(context.target_model.as_str())
    }) {
        return Ok(());
    }
    Err(CompactionHttpError::new(
        StatusCode::CONFLICT,
        "context_unavailable_for_target",
        "invalid_request",
        "UMP compaction pack is not compatible with the resolved target",
    )
    .into())
}

fn validate_commitment(
    header: &Value,
    name: &str,
    expected_value: &str,
    key: &[u8; 32],
) -> AppResult<()> {
    let expected = commitment(key, expected_value)?;
    let actual = header
        .pointer(&format!("/auth_binding/{name}"))
        .and_then(Value::as_str);
    if actual == Some(expected.as_str()) {
        return Ok(());
    }
    Err(CompactionHttpError::new(
        StatusCode::BAD_REQUEST,
        "compaction_replay_rejected",
        "invalid_request",
        "UMP compaction pack binding does not match this request",
    )
    .into())
}

fn derive_key(root_key: &[u8; 32], salt: &[u8], info: &[u8]) -> AppResult<[u8; 32]> {
    let hk = Hkdf::<Sha256>::new(Some(salt), root_key);
    let mut key = [0_u8; 32];
    hk.expand(info, &mut key)
        .map_err(|_| CompactionHttpError::invalid_pack("failed to derive UMP pack key"))?;
    Ok(key)
}

fn commitment(key: &[u8; 32], value: &str) -> AppResult<String> {
    let mut mac = <HmacSha256 as Mac>::new_from_slice(key)
        .map_err(|_| CompactionHttpError::invalid_pack("invalid commitment key"))?;
    mac.update(value.as_bytes());
    Ok(URL_SAFE_NO_PAD.encode(mac.finalize().into_bytes()))
}

fn random_nonce() -> [u8; 12] {
    let mut nonce = [0_u8; 12];
    OsRng.fill_bytes(&mut nonce);
    nonce
}

fn unix_seconds_string() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default()
        .to_string()
}

fn decode_base64url(encoded: &str) -> AppResult<Vec<u8>> {
    URL_SAFE_NO_PAD.decode(encoded).map_err(|error| {
        CompactionHttpError::invalid_pack(format!("invalid base64url: {error}")).into()
    })
}

fn decode_base64_any(encoded: &str) -> AppResult<Vec<u8>> {
    URL_SAFE_NO_PAD
        .decode(encoded)
        .or_else(|_| STANDARD_NO_PAD.decode(encoded))
        .or_else(|_| STANDARD.decode(encoded))
        .map_err(|error| {
            CompactionHttpError::new(
                StatusCode::SERVICE_UNAVAILABLE,
                "proxy_compaction_unavailable",
                "server_error",
                format!("invalid UMP compaction key encoding: {error}"),
            )
            .into()
        })
}

#[cfg(test)]
pub fn test_pack_context() -> CompactionPackContext {
    CompactionPackContext {
        auth_subject: "local-no-auth:test".into(),
        session_binding: "session:test".into(),
        route_binding: "POST /v1/responses".into(),
        target_provider: "bedrock".into(),
        target_format: "anthropic_messages".into(),
        target_model: "anthropic.claude-opus-4-1".into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic_pack_round_trips_visible_context() {
        let context = test_pack_context();
        let pack = deterministic_ump_pack(
            CompactVisibleContext {
                task_objective: Some("ship lane 1".into()),
                durable_constraints: vec!["small diff".into()],
                summary: Some("summary".into()),
                context_degraded: false,
            },
            &context,
        )
        .unwrap();

        let decoded =
            decode_deterministic_ump_pack(&pack, CompactionLimits::default(), &context).unwrap();

        assert_eq!(
            decoded.visible.task_objective.as_deref(),
            Some("ship lane 1")
        );
        assert_eq!(decoded.visible.durable_constraints, ["small diff"]);
        assert_eq!(decoded.protected_header["alg"], "A256GCM-SIV");
    }

    #[test]
    fn wrong_session_binding_rejects_replay() {
        let context = test_pack_context();
        let pack = deterministic_ump_pack(
            CompactVisibleContext {
                task_objective: Some("ship lane 1".into()),
                durable_constraints: Vec::new(),
                summary: None,
                context_degraded: false,
            },
            &context,
        )
        .unwrap();
        let mut wrong_context = context.clone();
        wrong_context.session_binding = "session:other".into();

        let error =
            decode_deterministic_ump_pack(&pack, CompactionLimits::default(), &wrong_context)
                .unwrap_err();

        assert_eq!(error.code(), Some("compaction_replay_rejected"));
    }

    #[test]
    fn malformed_marker_is_invalid_pack() {
        let error = decode_deterministic_ump_pack(
            "ump.compaction.v1.only-one-part",
            CompactionLimits::default(),
            &test_pack_context(),
        )
        .unwrap_err();

        assert_eq!(error.code(), Some("invalid_ump_compaction_pack"));
    }
}
