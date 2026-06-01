use std::env;
use std::fs;
use std::process::Command;

use unified_model_proxy_v2::upstream::windsurf::{
    encode_message, encode_string, encode_varint_field,
};
use uuid::Uuid;

const ASSIGN_ARENA_PATH: &str = "/exa.api_server_pb.ApiServerService/AssignArenaModel";
const CLIENT_VERSION: &str = "1.13.104";

fn main() {
    let mut args = env::args().skip(1);
    let model = args.next().unwrap_or_else(|| "swe-grep".to_string());
    let host_alias = args.next().unwrap_or_else(|| "codeium".to_string());

    let host = match host_alias.as_str() {
        "codeium" => "https://server.codeium.com",
        "self-serve" => "https://server.self-serve.windsurf.com",
        other => other,
    };

    let api_key = read_api_key().expect("missing api key");
    let cascade_id = Uuid::new_v4().to_string();
    let arena_id = Uuid::new_v4().to_string();

    let assign_payload =
        build_assign_arena_model_request(&api_key, CLIENT_VERSION, &model, &cascade_id, &arena_id);
    let assign_url = format!("{host}{ASSIGN_ARENA_PATH}");
    let path = "/tmp/probe_arena.bin";
    fs::write(path, &assign_payload).unwrap();
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
            &assign_url,
        ])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    println!("Response:\n{stdout}");
}

fn build_assign_arena_model_request(
    api_key: &str,
    version: &str,
    model: &str,
    cascade_id: &str,
    arena_id: &str,
) -> Vec<u8> {
    let metadata = build_metadata(api_key, version);

    let mut out = Vec::new();
    out.extend(encode_message(1, &metadata));
    out.extend(encode_string(2, arena_id));
    out.extend(encode_varint_field(3, 1));
    out.extend(encode_string(4, cascade_id));
    out.extend(encode_string(5, model)); // model_router_uid
    out
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
