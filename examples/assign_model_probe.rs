//! Probe the two-step Windsurf authorization flow for protected models
//! (swe-grep, swe-grep-mini, etc.):
//!
//!   1. POST /exa.api_server_pb.ApiServerService/AssignModel
//!      -> AssignModelResponse { assignment: { assignment_jwt, ... } }
//!   2. POST /exa.api_server_pb.ApiServerService/GetChatMessage
//!      with field 26 (model_assignment_jwt) populated.
//!
//! Mirrors the metadata layout used by the production proxy
//! (src/upstream/windsurf.rs::build_metadata) and adds the AssignModel-only
//! fields documented in docs/windsurf/proto/windsurf_assign.proto.
//!
//! Usage:
//!   cargo run --example assign_model_probe -- <model> [host]
//!   cargo run --example assign_model_probe -- swe-grep-mini
//!   cargo run --example assign_model_probe -- swe-grep self-serve

use std::env;
use std::fs;
use std::process::Command;

use unified_model_proxy_v2::upstream::windsurf::{
    connect_envelope, encode_message, encode_string, encode_varint_field,
};
use uuid::Uuid;

const ASSIGN_PATH: &str = "/exa.api_server_pb.ApiServerService/AssignModel";
const CHAT_PATH: &str = "/exa.api_server_pb.ApiServerService/GetChatMessage";
const CLIENT_VERSION: &str = "1.13.104";

fn main() {
    let mut args = env::args().skip(1);
    let model = args.next().unwrap_or_else(|| "swe-grep-mini".to_string());
    let host_alias = args.next().unwrap_or_else(|| "codeium".to_string());

    let host = match host_alias.as_str() {
        "codeium" => "https://server.codeium.com",
        "self-serve" => "https://server.self-serve.windsurf.com",
        other => other,
    };

    let api_key = read_api_key().expect("WINDSURF_API_KEY env or ~/.ump/auth.json missing");

    let cascade_id = Uuid::new_v4().to_string();
    let prompt_id = Uuid::new_v4().to_string();
    let prompt_text = "List the top-level files in this repository.";

    eprintln!("== AssignModel probe ==");
    eprintln!("model:       {model}");
    eprintln!("host:        {host}");
    eprintln!("cascade_id:  {cascade_id}");
    eprintln!("prompt_id:   {prompt_id}");
    eprintln!(
        "api_key:     {}...",
        &api_key.chars().take(8).collect::<String>()
    );
    eprintln!();

    let assign_payload = build_assign_model_request(
        &api_key,
        CLIENT_VERSION,
        &model,
        &cascade_id,
        &prompt_id,
        prompt_text,
    );
    // AssignModel is a unary RPC; the accept-post header on this path lists
    // application/proto and application/grpc-web+proto but NOT application/connect+proto.
    // Use Connect unary mode (raw proto, no length prefix) by sending application/proto
    // with the bare protobuf bytes.
    let assign_url = format!("{host}{ASSIGN_PATH}");
    let assign_resp = post_with_content_type(&assign_url, &assign_payload, "application/proto");
    println!("---- AssignModel response ----");
    print_response(&assign_resp);

    let body = response_body(&assign_resp);
    let mut maybe_jwt: Option<String> = None;

    // For application/proto unary, body is raw protobuf with no Connect framing.
    // For application/connect+proto, it would be length-prefixed frames.
    // Try raw parse first; if that fails, fall back to frame parsing.
    let direct_assignment = find_message_field(&body, 1);
    if let Some(assignment) = direct_assignment {
        if let Some(jwt) = find_string_field(assignment, 1) {
            eprintln!("\n[direct] extracted assignment_jwt ({} chars)", jwt.len());
            if let Some(model_uid) = find_string_field(assignment, 2) {
                eprintln!("[direct] model_uid: {model_uid}");
            }
            maybe_jwt = Some(jwt);
        }
    }

    if maybe_jwt.is_none() {
        let frames = parse_connect_frames(&body);
        for (flags, payload) in &frames {
            if *flags == 0 {
                if let Some(assignment) = find_message_field(payload, 1) {
                    if let Some(jwt) = find_string_field(assignment, 1) {
                        eprintln!("\n[frame] extracted assignment_jwt ({} chars)", jwt.len());
                        if let Some(model_uid) = find_string_field(assignment, 2) {
                            eprintln!("[frame] model_uid: {model_uid}");
                        }
                        maybe_jwt = Some(jwt);
                    }
                }
            } else if *flags == 2 {
                eprintln!("\ntrailer frame: {}", String::from_utf8_lossy(payload));
            }
        }
    }

    let Some(jwt) = maybe_jwt else {
        eprintln!("\nno assignment_jwt found, aborting before GetChatMessage");
        std::process::exit(1);
    };

    eprintln!("\n== GetChatMessage with model_assignment_jwt (field 26) ==");
    let chat_payload = build_get_chat_message_with_jwt(
        &api_key,
        CLIENT_VERSION,
        &model,
        &cascade_id,
        &prompt_id,
        prompt_text,
        &jwt,
    );
    let chat_body = connect_envelope(&chat_payload);
    let chat_url = format!("{host}{CHAT_PATH}");
    let chat_resp = post(&chat_url, &chat_body);
    println!("---- GetChatMessage response ----");
    print_response(&chat_resp);

    let chat_body_bytes = response_body(&chat_resp);
    let chat_frames = parse_connect_frames(&chat_body_bytes);
    eprintln!("\n== GetChatMessage frames: {} ==", chat_frames.len());
    for (i, (flags, payload)) in chat_frames.iter().enumerate() {
        eprintln!("[{i}] flags={flags} len={}", payload.len());
        let mut texts = Vec::new();
        collect_string_field_3(payload, &mut texts);
        for t in &texts {
            let preview: String = t.chars().take(160).collect();
            eprintln!("    field-3 string: {preview}");
        }
        if *flags == 2 {
            eprintln!("    trailer: {}", String::from_utf8_lossy(payload));
        }
    }
}

fn build_assign_model_request(
    api_key: &str,
    version: &str,
    model: &str,
    cascade_id: &str,
    prompt_id: &str,
    prompt_text: &str,
) -> Vec<u8> {
    let metadata = build_metadata(api_key, version);
    let chat_message_prompt = build_chat_message_prompt(prompt_id, prompt_text);

    let mut out = Vec::with_capacity(256);
    out.extend(encode_message(1, &metadata)); // metadata
    out.extend(encode_string(2, model)); // model_router_uid
    out.extend(encode_string(3, cascade_id)); // cascade_id
    out.extend(encode_message(5, &chat_message_prompt)); // chat_message_prompt
    out
}

fn build_chat_message_prompt(prompt_id: &str, prompt_text: &str) -> Vec<u8> {
    let mut out = Vec::with_capacity(prompt_text.len() + 64);
    out.extend(encode_string(1, prompt_id)); // id
    out.extend(encode_varint_field(2, 1)); // source = user
    out.extend(encode_string(3, prompt_text)); // prompt
    out
}

fn build_metadata(api_key: &str, version: &str) -> Vec<u8> {
    let mut out = Vec::with_capacity(api_key.len() + version.len() * 2 + 96);
    out.extend(encode_string(1, "windsurf")); // ide_name
    out.extend(encode_string(2, version)); // extension_version
    out.extend(encode_string(3, api_key)); // api_key
    out.extend(encode_string(4, "en-US")); // locale
    out.extend(encode_string(7, version)); // ide_version
    let request_id: u64 = (std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0))
        % u64::MAX;
    out.extend(encode_varint_field(9, request_id)); // request_id (uint64)
    out.extend(encode_string(10, &Uuid::new_v4().to_string())); // session_id
    out.extend(encode_string(12, "windsurf")); // extension_name
    out.extend(encode_varint_field(15, 1)); // auth_source = AUTH_SOURCE_WIND_SURF_IDE
    out.extend(encode_string(28, "windsurf")); // ide_type
    out
}

/// Mirror of build_get_chat_message_request but with field 26 (model_assignment_jwt)
/// included. Also keeps cascade_id + prompt_id stable so AssignModel and
/// GetChatMessage agree on the session.
fn build_get_chat_message_with_jwt(
    api_key: &str,
    version: &str,
    model: &str,
    cascade_id: &str,
    prompt_id: &str,
    prompt_text: &str,
    jwt: &str,
) -> Vec<u8> {
    let metadata = build_metadata(api_key, version);

    // ChatMessagePrompt-style entry: id (1), source (2)=user, prompt (3)
    let mut prompt_msg = Vec::with_capacity(prompt_text.len() + 64);
    prompt_msg.extend(encode_string(1, prompt_id));
    prompt_msg.extend(encode_varint_field(2, 1));
    prompt_msg.extend(encode_string(3, prompt_text));

    let mut out = Vec::with_capacity(512);
    out.extend(encode_message(1, &metadata)); // metadata
    out.extend(encode_message(3, &prompt_msg)); // first prompt entry (mirrors prod field 3)
    out.extend(encode_varint_field(7, 5)); // existing varint field used in prod
    out.extend(encode_string(21, model)); // upstream_model
                                          // Section 3.4: also include cascade_id + assignment JWT.
                                          // Field number for cascade_id in GetChatMessageRequest is not in the proto we have,
                                          // but the assignment JWT is the gating element. Try field 26 alone first.
    out.extend(encode_string(26, jwt)); // model_assignment_jwt
    out
}

// ---------- HTTP / framing helpers ----------

fn post(url: &str, body: &[u8]) -> Vec<u8> {
    post_with_content_type(url, body, "application/connect+proto")
}

fn post_with_content_type(url: &str, body: &[u8], content_type: &str) -> Vec<u8> {
    let path = format!("/tmp/probe_{}.bin", Uuid::new_v4());
    fs::write(&path, body).expect("write probe body");
    let accept = content_type.to_string();
    let output = Command::new("curl")
        .args([
            "-sS",
            "-i",
            "-X",
            "POST",
            "-H",
            &format!("content-type: {content_type}"),
            "-H",
            "connect-protocol-version: 1",
            "-H",
            &format!("accept: {accept}"),
            "--data-binary",
            &format!("@{path}"),
            url,
        ])
        .output()
        .expect("curl invocation failed");
    let _ = fs::remove_file(&path);
    output.stdout
}

fn print_response(stdout: &[u8]) {
    let split = find_header_body_split(stdout);
    let (head, body) = stdout.split_at(split);
    println!("{}", String::from_utf8_lossy(head));
    println!("--- body ({} bytes) ---", body.len());
    let printable_ratio = body
        .iter()
        .filter(|b| b.is_ascii_graphic() || b.is_ascii_whitespace())
        .count() as f32
        / body.len().max(1) as f32;
    if printable_ratio > 0.85 {
        println!("{}", String::from_utf8_lossy(body));
    } else {
        let preview: Vec<String> = body.iter().take(256).map(|b| format!("{b:02x}")).collect();
        println!("hex (first 256): {}", preview.join(" "));
        if let Ok(s) = std::str::from_utf8(body) {
            println!("(utf8 lossy: {})", s.chars().take(400).collect::<String>());
        } else {
            println!(
                "(utf8 lossy: {})",
                String::from_utf8_lossy(body)
                    .chars()
                    .take(400)
                    .collect::<String>()
            );
        }
    }
}

fn response_body(stdout: &[u8]) -> Vec<u8> {
    let split = find_header_body_split(stdout);
    stdout[split..].to_vec()
}

fn find_header_body_split(buf: &[u8]) -> usize {
    let mut last = 0usize;
    let mut i = 0usize;
    while i + 3 < buf.len() {
        if &buf[i..i + 4] == b"\r\n\r\n" {
            last = i + 4;
        }
        i += 1;
    }
    if last == 0 {
        buf.len()
    } else {
        last
    }
}

fn parse_connect_frames(buf: &[u8]) -> Vec<(u8, Vec<u8>)> {
    let mut out = Vec::new();
    let mut offset = 0;
    while offset + 5 <= buf.len() {
        let flags = buf[offset];
        let length = u32::from_be_bytes([
            buf[offset + 1],
            buf[offset + 2],
            buf[offset + 3],
            buf[offset + 4],
        ]) as usize;
        let start = offset + 5;
        let end = start + length;
        if end > buf.len() {
            break;
        }
        out.push((flags, buf[start..end].to_vec()));
        offset = end;
    }
    out
}

// ---------- minimal protobuf walker ----------

fn read_varint(buf: &[u8], mut offset: usize) -> Option<(u64, usize)> {
    let mut result: u64 = 0;
    let mut shift = 0u32;
    loop {
        if offset >= buf.len() {
            return None;
        }
        let byte = buf[offset];
        offset += 1;
        result |= ((byte & 0x7f) as u64) << shift;
        if byte & 0x80 == 0 {
            return Some((result, offset));
        }
        shift += 7;
        if shift >= 64 {
            return None;
        }
    }
}

fn find_string_field(buf: &[u8], target_field: u32) -> Option<String> {
    walk_for_field(buf, target_field, 2).and_then(|bytes| String::from_utf8(bytes).ok())
}

fn find_message_field(buf: &[u8], target_field: u32) -> Option<&[u8]> {
    walk_for_field_borrow(buf, target_field, 2)
}

fn walk_for_field(buf: &[u8], target_field: u32, target_wire: u8) -> Option<Vec<u8>> {
    walk_for_field_borrow(buf, target_field, target_wire).map(|s| s.to_vec())
}

fn walk_for_field_borrow(buf: &[u8], target_field: u32, target_wire: u8) -> Option<&[u8]> {
    let mut offset = 0;
    while offset < buf.len() {
        let (key, next) = read_varint(buf, offset)?;
        offset = next;
        let field = (key >> 3) as u32;
        let wire = (key & 7) as u8;
        match wire {
            0 => {
                let (_, n) = read_varint(buf, offset)?;
                offset = n;
            }
            1 => {
                if offset + 8 > buf.len() {
                    return None;
                }
                offset += 8;
            }
            2 => {
                let (len, n) = read_varint(buf, offset)?;
                offset = n;
                let end = offset.checked_add(len as usize)?;
                if end > buf.len() {
                    return None;
                }
                if field == target_field && wire == target_wire {
                    return Some(&buf[offset..end]);
                }
                offset = end;
            }
            5 => {
                if offset + 4 > buf.len() {
                    return None;
                }
                offset += 4;
            }
            _ => return None,
        }
    }
    None
}

fn collect_string_field_3(buf: &[u8], out: &mut Vec<String>) {
    let mut offset = 0;
    while offset < buf.len() {
        let Some((key, next)) = read_varint(buf, offset) else {
            break;
        };
        offset = next;
        let field = (key >> 3) as u32;
        let wire = (key & 7) as u8;
        match wire {
            0 => {
                let Some((_, n)) = read_varint(buf, offset) else {
                    break;
                };
                offset = n;
            }
            1 => {
                if offset + 8 > buf.len() {
                    break;
                }
                offset += 8;
            }
            2 => {
                let Some((len, n)) = read_varint(buf, offset) else {
                    break;
                };
                offset = n;
                let Some(end) = offset.checked_add(len as usize) else {
                    break;
                };
                if end > buf.len() {
                    break;
                }
                if field == 3 {
                    if let Ok(s) = std::str::from_utf8(&buf[offset..end]) {
                        out.push(s.to_string());
                    }
                }
                offset = end;
            }
            5 => {
                if offset + 4 > buf.len() {
                    break;
                }
                offset += 4;
            }
            _ => break,
        }
    }
}

// ---------- credential resolution (same as swe_grep_probe) ----------

fn read_api_key() -> Option<String> {
    if let Ok(env) = env::var("WINDSURF_API_KEY") {
        let trimmed = env.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }
    let home = env::var("HOME").ok()?;
    for path in [
        format!("{home}/.ump/auth.json"),
        format!("{home}/.windsurf/auth.json"),
    ] {
        let Ok(text) = fs::read_to_string(&path) else {
            continue;
        };
        let Ok(value) = serde_json::from_str::<serde_json::Value>(&text) else {
            continue;
        };
        for ptr in [
            "/windsurf/api_key",
            "/windsurf/apiKey",
            "/api_key",
            "/apiKey",
        ] {
            if let Some(s) = value.pointer(ptr).and_then(|v| v.as_str()) {
                if !s.is_empty() {
                    return Some(s.to_string());
                }
            }
        }
    }
    None
}
