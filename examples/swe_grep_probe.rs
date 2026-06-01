//! Probe whether server.codeium.com (and server.self-serve.windsurf.com)
//! accept arbitrary `upstream_model` values via the GetChatMessage Connect RPC.
//!
//! Reuses the proxy's exact protobuf builder so the wire format is faithful
//! to what `src/upstream/windsurf.rs` produces in production.
//!
//! Usage:
//!   cargo run --example swe_grep_probe -- <model> [host]
//!   cargo run --example swe_grep_probe -- swe-grep
//!   cargo run --example swe_grep_probe -- swe-grep-mini self-serve
//!   cargo run --example swe_grep_probe -- swe-1-6-fast codeium
//!
//! Writes the Connect-framed body to /tmp/swe_grep_probe.bin and POSTs via
//! curl, printing status, response headers, and a hex/text view of the body.

use std::env;
use std::fs;
use std::process::Command;

use serde_json::json;
use unified_model_proxy_v2::upstream::windsurf::{
    build_get_chat_message_request, connect_envelope,
};

fn main() {
    let mut args = env::args().skip(1);
    let model = args.next().unwrap_or_else(|| "swe-grep".to_string());
    let host_alias = args.next().unwrap_or_else(|| "codeium".to_string());

    let host = match host_alias.as_str() {
        "codeium" => "https://server.codeium.com",
        "self-serve" => "https://server.self-serve.windsurf.com",
        other => other,
    };

    let api_key = read_api_key().expect("WINDSURF_API_KEY env or ~/.ump/auth.json missing");

    let request = json!({
        "messages": [
            {
                "role": "user",
                "content": "List the top-level files in this repository."
            }
        ]
    });

    let payload = build_get_chat_message_request(&request, &api_key, "1.13.104", &model)
        .expect("encode payload");
    let frame = connect_envelope(&payload);
    let body_path = "/tmp/swe_grep_probe.bin";
    fs::write(body_path, &frame).expect("write probe body");

    let url = format!("{host}/exa.api_server_pb.ApiServerService/GetChatMessage");
    eprintln!("== probe ==");
    eprintln!("model:          {model}");
    eprintln!("host:           {host}");
    eprintln!(
        "body bytes:     {} (frame) / {} (payload)",
        frame.len(),
        payload.len()
    );
    eprintln!(
        "api_key prefix: {}...",
        &api_key.chars().take(6).collect::<String>()
    );
    eprintln!();

    let output = Command::new("curl")
        .args([
            "-sS",
            "-i",
            "-X",
            "POST",
            "-H",
            "content-type: application/connect+proto",
            "-H",
            "connect-protocol-version: 1",
            "-H",
            "accept: application/connect+proto",
            "--data-binary",
            &format!("@{body_path}"),
            &url,
        ])
        .output()
        .expect("curl invocation failed");

    eprintln!("== curl stderr ==");
    eprintln!("{}", String::from_utf8_lossy(&output.stderr));
    eprintln!("== curl stdout (status + headers + body) ==");
    let stdout = output.stdout;
    let split = find_header_body_split(&stdout);
    let (head, body) = stdout.split_at(split);
    println!("{}", String::from_utf8_lossy(head));
    println!("--- body ({} bytes) ---", body.len());
    println!("{}", render_body(body));
}

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

fn render_body(bytes: &[u8]) -> String {
    let printable_ratio = bytes
        .iter()
        .filter(|b| b.is_ascii_graphic() || b.is_ascii_whitespace())
        .count() as f32
        / bytes.len().max(1) as f32;
    if printable_ratio > 0.85 {
        String::from_utf8_lossy(bytes).into_owned()
    } else {
        let preview: Vec<String> = bytes.iter().take(256).map(|b| format!("{b:02x}")).collect();
        format!("hex (first 256): {}", preview.join(" "))
    }
}
