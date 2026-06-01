use std::env;
use std::fs;
use std::process::Command;

use unified_model_proxy_v2::upstream::windsurf::{
    connect_envelope, encode_message, encode_string, encode_varint_field,
};
use uuid::Uuid;

const ASSIGN_PATH: &str = "/exa.api_server_pb.ApiServerService/AssignModel";
const CLIENT_VERSION: &str = "1.13.104";

fn main() {
    let mut args = env::args().skip(1);
    let model = args.next().unwrap_or_else(|| "swe-grep".to_string());
    let host_alias = args.next().unwrap_or_else(|| "self-serve".to_string());

    let host = match host_alias.as_str() {
        "codeium" => "https://server.codeium.com",
        "self-serve" => "https://server.self-serve.windsurf.com",
        other => other,
    };

    let api_key = read_api_key().expect("missing api key");
    let user_jwt = if let Some(stripped) = api_key.strip_prefix("devin-session-token$") {
        stripped.to_string()
    } else {
        String::new()
    };

    let cascade_id = Uuid::new_v4().to_string();
    let prompt_id = Uuid::new_v4().to_string();
    let prompt_text = "List the top-level files in this repository.";

    let metadata = build_metadata(&api_key, CLIENT_VERSION, &user_jwt);
    let chat_message_prompt = build_chat_message_prompt(&prompt_id, prompt_text);

    let mut out = Vec::with_capacity(256);
    out.extend(encode_message(1, &metadata)); // metadata
    out.extend(encode_string(2, &model)); // model_router_uid
    out.extend(encode_string(3, &cascade_id)); // cascade_id
    out.extend(encode_message(5, &chat_message_prompt)); // chat_message_prompt

    let assign_url = format!("{host}{ASSIGN_PATH}");
    let path = "/tmp/probe_assign.bin";
    fs::write(path, &out).unwrap();
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

fn build_chat_message_prompt(prompt_id: &str, prompt_text: &str) -> Vec<u8> {
    let mut out = Vec::with_capacity(prompt_text.len() + 64);
    out.extend(encode_string(1, prompt_id));
    out.extend(encode_varint_field(2, 1));
    out.extend(encode_string(3, prompt_text));
    out
}

fn build_metadata(api_key: &str, version: &str, user_jwt: &str) -> Vec<u8> {
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
    if !user_jwt.is_empty() {
        out.extend(encode_string(21, user_jwt));
    }
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
    for path in [format!("{home}/.windsurf/auth.json")] {
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
            "/accessToken",
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
