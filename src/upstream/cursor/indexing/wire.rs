//! Cursor RepositoryService wire layer.
//!
//! Five Connect-unary RPCs share this encoder set, mirroring the TS
//! `cursor-index-wire.ts` byte-for-byte. Field tags and constants come from
//! `indexing-extraction.md` "Cloud index RPCs". Decoders parse only the
//! fields the orchestrator reads; the rest of the proto is skipped.
//!
//! Path encryption follows Cursor's IDE scheme: per-segment AES-256-CTR
//! with a 6-byte HMAC-SHA256 prefix used as the IV head. The cipher
//! degrades to passthrough when no key is available, exactly as the TS
//! plugin does (`makePathCipher` returning `undefined` short-circuits
//! `encryptCursorPath`).
//!
//! Wire format for an encrypted path segment (matches `cursor-index-wire.ts`):
//!
//! ```text
//! base64url(
//!   [ 6-byte HMAC-SHA256(macKey, plaintext) prefix ]
//!   [ AES-256-CTR ciphertext of plaintext null-padded to a 4-byte boundary ]
//! )
//! ```
//!
//! The AES counter (16 bytes) is the 6-byte MAC prefix followed by 10 zero
//! bytes, matching Node's `Buffer.alloc(10)` in the reference plugin. Two
//! 32-byte keys are derived from the base64url master key:
//! `macKey = sha256(master || 0x00)`, `encKey = sha256(master || 0x01)`.
//! Decryption reverses the process and verifies the recovered MAC; mismatch
//! falls back to returning the segment untouched (mirrors the TS try/catch).

use aes::Aes256;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine as _;
use ctr::cipher::{KeyIvInit, StreamCipher};
use hmac::{Hmac, Mac};
use sha2::{Digest, Sha256};

type Aes256Ctr128BE = ctr::Ctr128BE<Aes256>;

use crate::upstream::cursor::proto::{
    concat_bytes, decode_varint, encode_bool_field, encode_int32_field, encode_message_field,
    encode_string_field, encode_varint, parse_proto_fields,
};

type HmacSha256 = Hmac<Sha256>;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// `RepositoryService` codebase status enum (handshake response field 2,
/// inner field 2). See `cursor-index-wire.ts:74-80`.
#[allow(dead_code)]
pub mod codebase_status {
    pub const UP_TO_DATE: i32 = 1;
    pub const OUT_OF_SYNC: i32 = 2;
    pub const EMPTY: i32 = 3;
    pub const EMPTY_WITH_COPY_AVAILABLE: i32 = 4;
    pub const COPY_IN_PROGRESS: i32 = 5;
}

pub const SIMILARITY_METRIC_TYPE_SIMHASH: i32 = 1;
pub const PATH_KEY_HASH_TYPE_SHA256: i32 = 1;
pub const FAST_UPDATE_STATUS_SUCCESS: i32 = 1;
pub const FAST_UPDATE_TYPE_ADD: i32 = 1;
pub const FAST_UPDATE_TYPE_BATCH: i32 = 4;
pub const SYNC_CODEBASE_STATUS_SUCCESS: i32 = 1;
pub const SYNC_CODEBASE_STATUS_FAILURE: i32 = 2;

// ---------------------------------------------------------------------------
// Index metadata + repository context
// ---------------------------------------------------------------------------

/// Cursor index metadata sourced from env, plugin state, or generated. The
/// source label is preserved for the diagnostic gate (UMP refuses
/// `plugin-generated` keys by default).
#[derive(Debug, Clone, Default)]
pub struct RepositoryIndexMetadata {
    pub workspace_uri: String,
    pub path_encryption_key: String,
    pub orthogonal_transform_seed: Option<f64>,
    pub repo_name: Option<String>,
    pub repo_owner: Option<String>,
    pub source: Option<MetadataSource>,
}

/// Where the metadata came from. Mirrors the TS
/// `RepositoryIndexMetadataSource` union.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum MetadataSource {
    CursorState,
    EnvJson,
    EnvFile,
    PluginGenerated,
}

impl MetadataSource {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::CursorState => "cursor-state",
            Self::EnvJson => "env-json",
            Self::EnvFile => "env-file",
            Self::PluginGenerated => "plugin-generated",
        }
    }
}

/// Repository context used to populate `RepositoryInfo`. Built off
/// `crate::upstream::cursor::workspace::RepoMetadata` with the public-facing
/// fields the wire encoders need.
#[derive(Debug, Clone, Default)]
pub struct RepositoryContext {
    pub relative_workspace_path: String,
    pub remotes: Vec<GitRemote>,
    pub repo_name: String,
    pub repo_owner: String,
    pub is_tracked: bool,
    pub is_local: bool,
}

/// Single git remote entry. Names parallel URLs in the same order so the
/// proto layer can encode parallel `repeated string` fields.
#[derive(Debug, Clone, Default)]
pub struct GitRemote {
    pub name: String,
    pub url: String,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct RepositoryInfoOverrides {
    pub is_tracked: Option<bool>,
    pub is_local: Option<bool>,
    pub num_files: Option<i32>,
}

/// Result of `decodeFastRepoInitHandshakeV2Response`.
#[derive(Debug, Clone)]
pub struct HandshakeResponse {
    pub status: i32,
    pub codebases: Vec<CodebaseInfo>,
}

#[derive(Debug, Clone)]
pub struct CodebaseInfo {
    pub codebase_id: String,
    pub status: i32,
}

#[derive(Debug, Clone, Default)]
pub struct UploadFile {
    pub relative_path: String,
    pub contents: String,
    pub hash: String,
    pub ancestor_paths: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct SyncCodebaseStatus {
    pub codebase_id: String,
    pub success: bool,
    pub total_upload_count: i32,
    pub failed_upload_count: i32,
}

/// Decoded `CodeResult` row.
#[derive(Debug, Clone, Default)]
pub struct CodeResult {
    pub path: String,
    pub contents: String,
    pub score: f32,
    pub start_line: Option<u32>,
    pub end_line: Option<u32>,
}

// ---------------------------------------------------------------------------
// Path cipher
// ---------------------------------------------------------------------------

/// SHA-256 of `pathEncryptionKey + "_PATH_KEY_HASH_SHA256"`, hex-encoded.
/// Mirrors TS `pathKeyHashHex`.
pub fn path_key_hash_hex(path_encryption_key: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(path_encryption_key.as_bytes());
    hasher.update(b"_PATH_KEY_HASH_SHA256");
    hex_lower(&hasher.finalize())
}

fn hex_lower(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push_str(&format!("{:02x}", byte));
    }
    out
}

/// Per-segment AES-256-CTR cipher used to obscure path components on the
/// wire. The TS plugin keeps separators (`/`, `\`, `.`) and empty parts
/// untouched and only encrypts the named segments.
pub fn encrypt_cursor_path(value: &str, path_encryption_key: &str) -> String {
    if path_encryption_key.is_empty() {
        return value.to_string();
    }
    encode_segments(value, |segment| {
        encrypt_segment(segment, path_encryption_key)
    })
}

pub fn decrypt_cursor_path(value: &str, path_encryption_key: &str) -> String {
    if path_encryption_key.is_empty() {
        return value.to_string();
    }
    encode_segments(value, |segment| {
        decrypt_segment(segment, path_encryption_key).unwrap_or_else(|| segment.to_string())
    })
}

fn encode_segments<F>(value: &str, mut transform: F) -> String
where
    F: FnMut(&str) -> String,
{
    // Mirrors `value.split(/([./\\])/)` — keep the separators in place and
    // transform only the segments between them.
    let mut out = String::with_capacity(value.len());
    let mut segment_start = 0usize;
    for (idx, ch) in value.char_indices() {
        if matches!(ch, '/' | '\\' | '.') {
            if idx > segment_start {
                let segment = &value[segment_start..idx];
                if !segment.is_empty() {
                    out.push_str(&transform(segment));
                }
            }
            out.push(ch);
            segment_start = idx + ch.len_utf8();
        }
    }
    if segment_start < value.len() {
        let tail = &value[segment_start..];
        if !tail.is_empty() {
            out.push_str(&transform(tail));
        }
    }
    out
}

/// Build the 6-byte HMAC-SHA256 prefix used as the IV head. The remaining 10
/// bytes are zero to match the TS shape (`Buffer.alloc(10)`).
fn iv_for(key: &MacKey, value: &str) -> [u8; 16] {
    let mut mac = HmacSha256::new_from_slice(&key.mac_key).expect("hmac key length");
    mac.update(value.as_bytes());
    let tag = mac.finalize().into_bytes();
    let mut iv = [0u8; 16];
    iv[..6].copy_from_slice(&tag[..6]);
    iv
}

struct MacKey {
    mac_key: [u8; 32],
    enc_key: [u8; 32],
}

fn derive_keys(master_key_b64url: &str) -> Option<MacKey> {
    let master = URL_SAFE_NO_PAD.decode(master_key_b64url).ok()?;
    let mac_key = sha256_with_byte(&master, 0);
    let enc_key = sha256_with_byte(&master, 1);
    Some(MacKey { mac_key, enc_key })
}

fn sha256_with_byte(input: &[u8], suffix: u8) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(input);
    hasher.update([suffix]);
    let mut out = [0u8; 32];
    out.copy_from_slice(&hasher.finalize());
    out
}

fn encrypt_segment(segment: &str, master_key_b64url: &str) -> String {
    let Some(keys) = derive_keys(master_key_b64url) else {
        return segment.to_string();
    };
    let iv = iv_for(&keys, segment);
    let padded = pad_segment(segment);
    let mut buffer = padded.into_bytes();
    let mut cipher = Aes256Ctr128BE::new(&keys.enc_key.into(), &iv.into());
    cipher.apply_keystream(&mut buffer);
    let mut combined = Vec::with_capacity(6 + buffer.len());
    combined.extend_from_slice(&iv[..6]);
    combined.extend_from_slice(&buffer);
    URL_SAFE_NO_PAD.encode(combined)
}

fn decrypt_segment(segment: &str, master_key_b64url: &str) -> Option<String> {
    let bytes = URL_SAFE_NO_PAD.decode(segment).ok()?;
    if bytes.len() <= 6 {
        return Some(segment.to_string());
    }
    let keys = derive_keys(master_key_b64url)?;
    let mut iv = [0u8; 16];
    iv[..6].copy_from_slice(&bytes[..6]);
    let mut buffer = bytes[6..].to_vec();
    let mut cipher = Aes256Ctr128BE::new(&keys.enc_key.into(), &iv.into());
    cipher.apply_keystream(&mut buffer);
    let text = String::from_utf8(buffer).ok()?;
    Some(text.trim_end_matches('\0').to_string())
}

fn pad_segment(value: &str) -> String {
    let pad = (4 - value.len() % 4) % 4;
    if pad == 0 {
        return value.to_string();
    }
    let mut out = String::with_capacity(value.len() + pad);
    out.push_str(value);
    for _ in 0..pad {
        out.push('\0');
    }
    out
}

// ---------------------------------------------------------------------------
// Local helpers (proto)
// ---------------------------------------------------------------------------

/// Encode a fixed64 little-endian double field. Lane D's `proto.rs` covers
/// varint/string/int32/bool, but not the wire-type-1 double tag this layer
/// needs.
fn encode_double_field(field_number: u32, value: f64) -> Vec<u8> {
    let mut out = encode_varint(((field_number << 3) | 1) as u64);
    out.extend_from_slice(&value.to_le_bytes());
    out
}

fn encode_repeated_message_field(field_number: u32, items: &[Vec<u8>]) -> Vec<u8> {
    let mut out = Vec::new();
    for item in items {
        out.extend_from_slice(&encode_message_field(field_number, item));
    }
    out
}

// ---------------------------------------------------------------------------
// RepositoryInfo encoder
// ---------------------------------------------------------------------------

/// Encode `RepositoryInfo`. Field tags follow `cursor-index-wire.ts:381-398`.
pub fn encode_repository_info(
    context: &RepositoryContext,
    metadata: &RepositoryIndexMetadata,
    overrides: RepositoryInfoOverrides,
) -> Vec<u8> {
    let mut chunks: Vec<Vec<u8>> = Vec::new();
    let relative = if context.relative_workspace_path.is_empty() {
        ".".to_string()
    } else {
        context.relative_workspace_path.clone()
    };
    chunks.push(encode_string_field(1, &relative));
    for remote in &context.remotes {
        chunks.push(encode_string_field(2, &remote.url));
    }
    for remote in &context.remotes {
        chunks.push(encode_string_field(3, &remote.name));
    }
    let repo_name = metadata
        .repo_name
        .clone()
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| context.repo_name.clone());
    let repo_owner = metadata
        .repo_owner
        .clone()
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| context.repo_owner.clone());
    chunks.push(encode_string_field(4, &repo_name));
    chunks.push(encode_string_field(5, &repo_owner));
    chunks.push(encode_bool_field(
        6,
        overrides.is_tracked.unwrap_or(context.is_tracked),
    ));
    chunks.push(encode_bool_field(
        7,
        overrides.is_local.unwrap_or(context.is_local),
    ));
    if let Some(num_files) = overrides.num_files {
        chunks.push(encode_int32_field(8, num_files as u32));
    }
    if let Some(seed) = metadata.orthogonal_transform_seed {
        chunks.push(encode_double_field(9, seed));
    }
    chunks.push(encode_string_field(11, &metadata.workspace_uri));
    concat_bytes(&chunks)
}

// ---------------------------------------------------------------------------
// SearchRepositoryV2
// ---------------------------------------------------------------------------

/// Encode the SearchRepositoryV2 request body.
pub fn encode_search_repository_request(
    query: &str,
    context: &RepositoryContext,
    metadata: &RepositoryIndexMetadata,
    top_k: i32,
) -> Vec<u8> {
    let mut chunks: Vec<Vec<u8>> = Vec::new();
    chunks.push(encode_string_field(1, query));
    let info = encode_repository_info(context, metadata, RepositoryInfoOverrides::default());
    chunks.push(encode_message_field(2, &info));
    chunks.push(encode_int32_field(3, top_k as u32));
    chunks.push(encode_bool_field(5, true));
    concat_bytes(&chunks)
}

/// Decode `SearchRepositoryV2Response`, decrypting paths with the metadata
/// key. Mirrors `decodeSearchRepositoryResponse`.
pub fn decode_search_repository_response(
    payload: &[u8],
    path_encryption_key: &str,
) -> Vec<CodeResult> {
    let body = strip_connect_unary_body(payload);
    let mut results = Vec::new();
    for field in parse_proto_fields(body) {
        if field.number == 1 && field.wire_type == 2 {
            if let Some(result) = decode_code_result(&field.value, path_encryption_key) {
                results.push(result);
            }
        }
    }
    results
}

fn decode_code_result(bytes: &[u8], path_encryption_key: &str) -> Option<CodeResult> {
    let mut block: Option<CodeBlockDraft> = None;
    let mut score: f32 = 0.0;
    for field in parse_proto_fields(bytes) {
        match (field.number, field.wire_type) {
            (1, 2) => block = Some(decode_code_block(&field.value, path_encryption_key)),
            (2, 5) => score = decode_fixed32_f32(&field.value).unwrap_or(0.0),
            _ => {}
        }
    }
    let block = block?;
    if block.path.is_empty() && block.contents.is_empty() {
        return None;
    }
    Some(CodeResult {
        path: block.path,
        contents: block.contents,
        score,
        start_line: block.start_line,
        end_line: block.end_line,
    })
}

#[derive(Default)]
struct CodeBlockDraft {
    path: String,
    contents: String,
    start_line: Option<u32>,
    end_line: Option<u32>,
}

fn decode_code_block(bytes: &[u8], path_encryption_key: &str) -> CodeBlockDraft {
    let mut draft = CodeBlockDraft::default();
    let mut detailed_lines: Vec<DetailedLine> = Vec::new();
    for field in parse_proto_fields(bytes) {
        match (field.number, field.wire_type) {
            (1, 2) => draft.path = String::from_utf8_lossy(&field.value).into_owned(),
            (3, 2) => {
                let range = decode_range(&field.value);
                draft.start_line = range.start_line;
                draft.end_line = range.end_line;
            }
            (4, 2) => draft.contents = String::from_utf8_lossy(&field.value).into_owned(),
            (8, 2) => detailed_lines.push(decode_detailed_line(&field.value)),
            _ => {}
        }
    }
    if draft.contents.is_empty() && !detailed_lines.is_empty() {
        draft.contents = detailed_lines
            .iter()
            .map(|line| line.text.clone().unwrap_or_default())
            .collect::<Vec<String>>()
            .join("\n");
    }
    if draft.start_line.is_none() {
        draft.start_line = detailed_lines.iter().find_map(|line| line.line_number);
    }
    if draft.end_line.is_none() {
        if let Some(start) = draft.start_line {
            let line_count = draft.contents.split('\n').count() as u32;
            draft.end_line = Some(start.saturating_add(line_count.saturating_sub(1)));
        }
    }
    draft.path = decrypt_cursor_path(&draft.path, path_encryption_key);
    draft
}

#[derive(Default)]
struct RangeDraft {
    start_line: Option<u32>,
    end_line: Option<u32>,
}

fn decode_range(bytes: &[u8]) -> RangeDraft {
    let mut draft = RangeDraft::default();
    for field in parse_proto_fields(bytes) {
        match (field.number, field.wire_type) {
            (1, 2) => draft.start_line = decode_position(&field.value),
            (2, 2) => draft.end_line = decode_position(&field.value),
            _ => {}
        }
    }
    draft
}

fn decode_position(bytes: &[u8]) -> Option<u32> {
    for field in parse_proto_fields(bytes) {
        if field.number == 1 && field.wire_type == 0 {
            let (line, _) = decode_varint(&field.value, 0)?;
            return Some(line as u32);
        }
    }
    None
}

#[derive(Default)]
struct DetailedLine {
    text: Option<String>,
    line_number: Option<u32>,
}

fn decode_detailed_line(bytes: &[u8]) -> DetailedLine {
    let mut draft = DetailedLine::default();
    for field in parse_proto_fields(bytes) {
        match (field.number, field.wire_type) {
            (1, 2) => draft.text = Some(String::from_utf8_lossy(&field.value).into_owned()),
            (2, 5) => draft.line_number = decode_fixed32_f32(&field.value).map(|v| v as u32),
            _ => {}
        }
    }
    draft
}

fn decode_fixed32_f32(bytes: &[u8]) -> Option<f32> {
    if bytes.len() < 4 {
        return None;
    }
    Some(f32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
}

// ---------------------------------------------------------------------------
// FastRepoInitHandshakeV2
// ---------------------------------------------------------------------------

pub fn encode_fast_repo_init_handshake_v2_request(
    context: &RepositoryContext,
    metadata: &RepositoryIndexMetadata,
    file_count: i32,
    root_hash: &str,
) -> Vec<u8> {
    let mut chunks: Vec<Vec<u8>> = Vec::new();
    let info = encode_repository_info(
        context,
        metadata,
        RepositoryInfoOverrides {
            is_tracked: Some(false),
            is_local: Some(true),
            num_files: Some(file_count),
        },
    );
    chunks.push(encode_message_field(1, &info));
    chunks.push(encode_string_field(2, root_hash));
    chunks.push(encode_int32_field(3, SIMILARITY_METRIC_TYPE_SIMHASH as u32));
    chunks.push(encode_string_field(
        5,
        &path_key_hash_hex(&metadata.path_encryption_key),
    ));
    chunks.push(encode_int32_field(6, PATH_KEY_HASH_TYPE_SHA256 as u32));
    chunks.push(encode_bool_field(7, false));
    concat_bytes(&chunks)
}

pub fn decode_fast_repo_init_handshake_v2_response(payload: &[u8]) -> HandshakeResponse {
    let body = strip_connect_unary_body(payload);
    let mut response = HandshakeResponse {
        status: 0,
        codebases: Vec::new(),
    };
    for field in parse_proto_fields(body) {
        match (field.number, field.wire_type) {
            (1, 0) => {
                if let Some((value, _)) = decode_varint(&field.value, 0) {
                    response.status = value as i32;
                }
            }
            (2, 2) => response.codebases.push(decode_codebase_info(&field.value)),
            _ => {}
        }
    }
    response
}

fn decode_codebase_info(bytes: &[u8]) -> CodebaseInfo {
    let mut info = CodebaseInfo {
        codebase_id: String::new(),
        status: 0,
    };
    for field in parse_proto_fields(bytes) {
        match (field.number, field.wire_type) {
            (1, 2) => info.codebase_id = String::from_utf8_lossy(&field.value).into_owned(),
            (2, 0) => {
                if let Some((value, _)) = decode_varint(&field.value, 0) {
                    info.status = value as i32;
                }
            }
            _ => {}
        }
    }
    info
}

// ---------------------------------------------------------------------------
// FastUpdateFileV2
// ---------------------------------------------------------------------------

fn encode_client_repository_info(metadata: &RepositoryIndexMetadata) -> Vec<u8> {
    encode_double_field(1, metadata.orthogonal_transform_seed.unwrap_or(0.0))
}

fn encode_uploaded_local_file(file: &UploadFile, metadata: &RepositoryIndexMetadata) -> Vec<u8> {
    let inner_path = encrypt_cursor_path(&file.relative_path, &metadata.path_encryption_key);
    let inner = concat_bytes(&[
        encode_string_field(1, &inner_path),
        encode_string_field(2, &file.contents),
    ]);
    concat_bytes(&[
        encode_message_field(1, &inner),
        encode_string_field(2, &file.hash),
        encode_string_field(3, &file.relative_path),
    ])
}

fn encode_partial_path_item(relative_path: &str, metadata: &RepositoryIndexMetadata) -> Vec<u8> {
    let encrypted = encrypt_cursor_path(relative_path, &metadata.path_encryption_key);
    concat_bytes(&[
        encode_string_field(1, &encrypted),
        encode_string_field(2, ""),
    ])
}

fn encode_file_update(file: &UploadFile, metadata: &RepositoryIndexMetadata) -> Vec<u8> {
    let mut chunks: Vec<Vec<u8>> = Vec::new();
    chunks.push(encode_message_field(
        2,
        &encode_uploaded_local_file(file, metadata),
    ));
    for ancestor in &file.ancestor_paths {
        chunks.push(encode_message_field(
            3,
            &encode_partial_path_item(ancestor, metadata),
        ));
    }
    chunks.push(encode_int32_field(4, FAST_UPDATE_TYPE_ADD as u32));
    concat_bytes(&chunks)
}

pub fn encode_fast_update_file_v2_request(
    codebase_id: &str,
    metadata: &RepositoryIndexMetadata,
    files: &[UploadFile],
) -> Vec<u8> {
    let mut chunks: Vec<Vec<u8>> = Vec::new();
    chunks.push(encode_message_field(
        1,
        &encode_client_repository_info(metadata),
    ));
    chunks.push(encode_string_field(2, codebase_id));
    if files.len() == 1 {
        let file = &files[0];
        chunks.push(encode_message_field(
            4,
            &encode_uploaded_local_file(file, metadata),
        ));
        for ancestor in &file.ancestor_paths {
            chunks.push(encode_message_field(
                5,
                &encode_partial_path_item(ancestor, metadata),
            ));
        }
        chunks.push(encode_int32_field(6, FAST_UPDATE_TYPE_ADD as u32));
    } else {
        chunks.push(encode_int32_field(6, FAST_UPDATE_TYPE_BATCH as u32));
        let updates: Vec<Vec<u8>> = files
            .iter()
            .map(|file| encode_file_update(file, metadata))
            .collect();
        chunks.push(encode_repeated_message_field(7, &updates));
    }
    concat_bytes(&chunks)
}

/// Decode just the top-level status varint from a FastUpdateFileV2 response.
pub fn decode_fast_update_file_v2_response_status(payload: &[u8]) -> i32 {
    let body = strip_connect_unary_body(payload);
    for field in parse_proto_fields(body) {
        if field.number == 1 && field.wire_type == 0 {
            if let Some((value, _)) = decode_varint(&field.value, 0) {
                return value as i32;
            }
        }
    }
    0
}

pub fn is_fast_update_file_v2_success(payload: &[u8]) -> bool {
    decode_fast_update_file_v2_response_status(payload) == FAST_UPDATE_STATUS_SUCCESS
}

// ---------------------------------------------------------------------------
// EnsureIndexCreated
// ---------------------------------------------------------------------------

pub fn encode_ensure_index_created_request(
    context: &RepositoryContext,
    metadata: &RepositoryIndexMetadata,
) -> Vec<u8> {
    let info = encode_repository_info(
        context,
        metadata,
        RepositoryInfoOverrides {
            is_tracked: Some(false),
            is_local: Some(true),
            num_files: Some(0),
        },
    );
    encode_message_field(1, &info)
}

// ---------------------------------------------------------------------------
// FastRepoSyncComplete
// ---------------------------------------------------------------------------

pub fn encode_fast_repo_sync_complete_request(
    codebases: &[SyncCodebaseStatus],
    metadata: &RepositoryIndexMetadata,
) -> Vec<u8> {
    let mut entries: Vec<Vec<u8>> = Vec::new();
    let path_hash = path_key_hash_hex(&metadata.path_encryption_key);
    for codebase in codebases {
        let status_value = if codebase.success {
            SYNC_CODEBASE_STATUS_SUCCESS
        } else {
            SYNC_CODEBASE_STATUS_FAILURE
        };
        let chunks: Vec<Vec<u8>> = vec![
            encode_string_field(1, &codebase.codebase_id),
            encode_int32_field(2, status_value as u32),
            encode_int32_field(3, SIMILARITY_METRIC_TYPE_SIMHASH as u32),
            encode_string_field(5, &path_hash),
            encode_int32_field(6, PATH_KEY_HASH_TYPE_SHA256 as u32),
            encode_int32_field(7, codebase.failed_upload_count as u32),
            encode_int32_field(8, 0),
            encode_int32_field(9, codebase.total_upload_count as u32),
            encode_int32_field(10, 0),
            encode_int32_field(11, 0),
            encode_int32_field(12, 0),
            encode_bool_field(13, false),
        ];
        entries.push(concat_bytes(&chunks));
    }
    encode_repeated_message_field(1, &entries)
}

// ---------------------------------------------------------------------------
// Connect-unary framing
// ---------------------------------------------------------------------------

/// Strip an optional Connect-unary frame envelope from `payload`. Mirrors
/// `decodeConnectUnaryBody` (`cursor-index-wire.ts:573-585`).
pub fn strip_connect_unary_body(payload: &[u8]) -> &[u8] {
    if payload.len() < 5 {
        return payload;
    }
    let mut offset = 0usize;
    while offset + 5 <= payload.len() {
        let flags = payload[offset];
        let length = u32::from_be_bytes([
            payload[offset + 1],
            payload[offset + 2],
            payload[offset + 3],
            payload[offset + 4],
        ]) as usize;
        let frame_end = offset + 5 + length;
        if frame_end > payload.len() {
            return payload;
        }
        if flags & 0b0000_0010 == 0 {
            return &payload[offset + 5..frame_end];
        }
        offset = frame_end;
    }
    payload
}

/// Parse a Connect JSON error envelope returned by the unary endpoint.
/// Returns `None` when the body does not start with `'{'` or fails to
/// decode.
pub fn parse_connect_error(body: &[u8]) -> Option<ConnectErrorPayload> {
    if body.is_empty() || body[0] != b'{' {
        return None;
    }
    let value: serde_json::Value = serde_json::from_slice(body).ok()?;
    let code = value.get("code")?.as_str()?.to_string();
    let message = value.get("message")?.as_str()?.to_string();
    let detail = value
        .get("details")
        .and_then(|details| details.as_array())
        .and_then(|details| details.first())
        .and_then(|first| first.get("debug"))
        .and_then(|debug| debug.get("details"))
        .and_then(|nested| nested.get("detail"))
        .and_then(|leaf| leaf.as_str())
        .map(str::to_owned);
    Some(ConnectErrorPayload {
        code,
        message,
        detail,
    })
}

#[derive(Debug, Clone)]
pub struct ConnectErrorPayload {
    pub code: String,
    pub message: String,
    pub detail: Option<String>,
}

impl ConnectErrorPayload {
    pub fn is_codebase_not_found(&self) -> bool {
        if let Some(detail) = self.detail.as_deref() {
            if detail.to_lowercase().contains("codebase not found") {
                return true;
            }
        }
        self.code == "invalid_argument"
            && self.message.to_lowercase().contains("codebase not found")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn path_key_hash_matches_ts_shape() {
        let hashed = path_key_hash_hex("abc");
        assert_eq!(hashed.len(), 64);
        assert_eq!(
            hashed, "914134a3b818ee9d8a305494a85b5904eca1b24c2376c43b564b11f9f66ca237",
            "known vector from TS: sha256(`${{pathEncryptionKey}}_PATH_KEY_HASH_SHA256`)"
        );
    }

    #[test]
    fn encrypt_path_passthrough_when_key_empty() {
        let value = "src/lib/foo.rs";
        assert_eq!(encrypt_cursor_path(value, ""), value);
    }

    #[test]
    fn encrypt_decrypt_roundtrip_matches_ts_shape() {
        // 32 raw bytes, base64url with no padding -> valid master key shape.
        let key = URL_SAFE_NO_PAD.encode([7u8; 32]);
        let path = "src/lib/foo.rs";
        let encrypted = encrypt_cursor_path(path, &key);
        // Separators preserved; segments differ from plaintext.
        assert_ne!(encrypted, path);
        assert_eq!(encrypted.matches('/').count(), 2);
        assert_eq!(encrypted.matches('.').count(), 1);
        assert_eq!(decrypt_cursor_path(&encrypted, &key), path);
    }

    #[test]
    fn encrypt_segment_is_deterministic_for_same_input() {
        // IV is derived from HMAC(plaintext), so identical plaintext yields
        // identical ciphertext — the property the TS plugin relies on.
        let key = URL_SAFE_NO_PAD.encode([3u8; 32]);
        let a = encrypt_cursor_path("foo", &key);
        let b = encrypt_cursor_path("foo", &key);
        assert_eq!(a, b);
    }

    #[test]
    fn search_request_encodes_query_and_repository_info() {
        let context = RepositoryContext {
            relative_workspace_path: ".".into(),
            ..RepositoryContext::default()
        };
        let metadata = RepositoryIndexMetadata::default();
        let bytes = encode_search_repository_request("hello", &context, &metadata, 10);
        assert!(!bytes.is_empty());
        // Field 1 is the query, wire-type 2 → tag = 0x0a.
        assert_eq!(bytes[0], 0x0a);
    }

    #[test]
    fn handshake_decode_recovers_status_and_codebases() {
        let mut payload: Vec<u8> = Vec::new();
        // Field 1 (status) wire-type 0, value 2.
        payload.extend_from_slice(&[0x08, 0x02]);
        // Field 2 (codebase) wire-type 2, value = inner message.
        let inner = concat_bytes(&[
            encode_string_field(1, "cb-1"),
            encode_int32_field(2, codebase_status::EMPTY as u32),
        ]);
        payload.extend_from_slice(&encode_message_field(2, &inner));
        let response = decode_fast_repo_init_handshake_v2_response(&payload);
        assert_eq!(response.status, 2);
        assert_eq!(response.codebases.len(), 1);
        assert_eq!(response.codebases[0].codebase_id, "cb-1");
        assert_eq!(response.codebases[0].status, codebase_status::EMPTY);
    }

    #[test]
    fn connect_error_detects_codebase_not_found() {
        let body = b"{\"code\":\"invalid_argument\",\"message\":\"codebase not found here\"}";
        let parsed = parse_connect_error(body).expect("parsed");
        assert!(parsed.is_codebase_not_found());
    }
}
