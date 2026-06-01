use std::env;
use std::fs;
use std::process::Command;
use unified_model_proxy_v2::upstream::windsurf::{encode_message, encode_string};

fn main() {
    let api_key = env::var("WINDSURF_API_KEY").unwrap_or_else(|_| {
        let text =
            fs::read_to_string(format!("{}/.ump/auth.json", env::var("HOME").unwrap())).unwrap();
        let value: serde_json::Value = serde_json::from_str(&text).unwrap();
        value
            .pointer("/windsurf/api_key")
            .unwrap()
            .as_str()
            .unwrap()
            .to_string()
    });

    let mut payload = Vec::new();
    payload.extend(encode_string(1, &api_key));

    let path = "/tmp/probe_migrate.bin";
    fs::write(path, &payload).unwrap();

    let url = "https://server.self-serve.windsurf.com/exa.seat_management_pb.SeatManagementService/MigrateApiKey";
    let output = Command::new("curl")
        .args([
            "-sS",
            "-i",
            "-X",
            "POST",
            "-H",
            "content-type: application/proto",
            "--data-binary",
            &format!("@{path}"),
            url,
        ])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    println!("Response:\n{stdout}");
}
