use std::env;
use std::fs;
use std::process::Command;
use uuid::Uuid;

use unified_model_proxy_v2::upstream::windsurf::{
    encode_message, encode_string, encode_varint_field,
};

const CONFIGS_PATH: &str = "/exa.api_server_pb.ApiServerService/GetCascadeModelConfigs";
const STATUSES_PATH: &str = "/exa.api_server_pb.ApiServerService/GetModelStatuses";
const CLIENT_VERSION: &str = "1.13.104";

fn main() {
    let api_key = read_api_key().expect("missing api key");

    let host = "https://server.codeium.com";

    // 1. GetCascadeModelConfigs
    let metadata = build_metadata(&api_key, CLIENT_VERSION);
    let mut payload = Vec::new();
    payload.extend(encode_message(1, &metadata));

    let path = "/tmp/probe_configs.bin";
    fs::write(path, &payload).unwrap();

    let url = format!("{host}{CONFIGS_PATH}");
    eprintln!("Sending GetCascadeModelConfigs to {url}...");
    let output = Command::new("curl")
        .args([
            "-sS",
            "-i",
            "-X",
            "POST",
            "-H",
            "content-type: application/proto",
            "-H",
            "accept: application/proto",
            "--data-binary",
            &format!("@{path}"),
            &url,
        ])
        .output()
        .unwrap();

    let stdout_bytes = output.stdout;
    let split = find_header_body_split(&stdout_bytes);
    let (head, body) = stdout_bytes.split_at(split);
    println!("=== GetCascadeModelConfigs Headers ===");
    println!("{}", String::from_utf8_lossy(head));

    let out_file =
        "/Users/jaredboynton/__devlocal/unified-model-proxy-v2/docs/windsurf/configs_response.bin";
    fs::write(out_file, body).unwrap();
    println!(
        "Wrote raw response body ({} bytes) to {}",
        body.len(),
        out_file
    );

    // 2. GetModelStatuses
    let metadata_statuses = build_metadata(&api_key, CLIENT_VERSION);
    let mut payload_statuses = Vec::new();
    payload_statuses.extend(encode_message(1, &metadata_statuses));

    let path_statuses = "/tmp/probe_statuses.bin";
    fs::write(path_statuses, &payload_statuses).unwrap();

    let url_statuses = format!("{host}{STATUSES_PATH}");
    eprintln!("\nSending GetModelStatuses to {url_statuses}...");
    let output_statuses = Command::new("curl")
        .args([
            "-sS",
            "-i",
            "-X",
            "POST",
            "-H",
            "content-type: application/proto",
            "-H",
            "accept: application/proto",
            "--data-binary",
            &format!("@{path_statuses}"),
            &url_statuses,
        ])
        .output()
        .unwrap();

    let stdout_bytes_statuses = output_statuses.stdout;
    let split_statuses = find_header_body_split(&stdout_bytes_statuses);
    let (head_statuses, body_statuses) = stdout_bytes_statuses.split_at(split_statuses);
    println!("=== GetModelStatuses Headers ===");
    println!("{}", String::from_utf8_lossy(head_statuses));

    let out_file_statuses =
        "/Users/jaredboynton/__devlocal/unified-model-proxy-v2/docs/windsurf/statuses_response.bin";
    fs::write(out_file_statuses, body_statuses).unwrap();
    println!(
        "Wrote raw response body ({} bytes) to {}",
        body_statuses.len(),
        out_file_statuses
    );
}

fn build_metadata(api_key: &str, version: &str) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend(encode_string(1, "windsurf"));
    out.extend(encode_string(2, version));
    out.extend(encode_string(3, api_key));
    out.extend(encode_string(4, "en-US"));
    out.extend(encode_string(7, version));
    out.extend(encode_varint_field(9, 12345));
    out.extend(encode_string(10, &Uuid::new_v4().to_string()));
    out.extend(encode_string(12, "windsurf"));
    out.extend(encode_varint_field(15, 1));
    out.extend(encode_string(28, "windsurf"));
    out
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
