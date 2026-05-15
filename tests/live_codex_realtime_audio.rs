mod common;

use std::{
    path::PathBuf,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use reqwest::{header, StatusCode};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use specter::Message as SpecterMessage;

const LIVE_OPT_IN: &str = "UMP_V2_LIVE_CODEX_REALTIME_AUDIO";
const LIVE_CI_OPT_IN: &str = "UMP_V2_ALLOW_LIVE_TESTS_IN_CI";
const LIVE_STT_AUDIO_FILE: &str = "UMP_V2_LIVE_STT_AUDIO_FILE";
const LIVE_STT_EXPECTED_PHRASE: &str = "UMP_V2_LIVE_STT_EXPECTED_PHRASE";
const EXPECTED_STT_PHRASE: &str = "guidewire realtime transcription test";
const REALTIME_MODEL: &str = "gpt-realtime-2";

#[tokio::test]
#[ignore = "requires UMP_V2_LIVE_CODEX_REALTIME_AUDIO=1 and local Codex OAuth auth"]
async fn live_codex_realtime_ws_text_roundtrip_when_opted_in() {
    let Some(_guard) = LiveGuard::from_env("realtime_ws_text_roundtrip") else {
        return;
    };

    let url = live_ws_url("/v1/realtime", &[("model", REALTIME_MODEL)]);
    let mut ws = match specter::Client::new()
        .expect("create specter client")
        .websocket(url)
        .connect()
        .await
    {
        Ok(ws) => ws,
        Err(_) => panic!("live realtime websocket connect failed"),
    };

    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time after UNIX_EPOCH")
        .as_nanos();
    ws.send_text(
        json!({
            "type": "response.create",
            "response": {
                "instructions": format!("Reply with exactly one short text token. Ignore this run nonce: {nonce}."),
                "output_modalities": ["text"]
            }
        })
        .to_string(),
    )
    .await
    .unwrap_or_else(|_| panic!("live realtime websocket send failed"));

    let mut event_names = Vec::new();
    let mut saw_text = false;
    let mut saw_done = false;

    for _ in 0..32 {
        let Some(value) = next_ws_json(&mut ws).await else {
            break;
        };
        let Some(event_name) = value.get("type").and_then(Value::as_str) else {
            continue;
        };
        event_names.push(event_name.to_string());
        if event_name == "error" {
            panic!("live realtime websocket returned error event");
        }
        if value
            .get("delta")
            .and_then(Value::as_str)
            .is_some_and(|delta| !delta.is_empty())
            || value
                .pointer("/response/output")
                .and_then(Value::as_array)
                .is_some_and(|items| !items.is_empty())
        {
            saw_text = true;
        }
        if event_name == "response.done" {
            saw_done = true;
            break;
        }
    }

    assert!(
        saw_done && saw_text,
        "live realtime websocket did not complete text roundtrip: saw_done={saw_done} saw_text={saw_text}"
    );
    println!(
        "live_status endpoint_class=realtime_ws_text status=ok event_names={} text_present={} done_present={}",
        event_names.join(","),
        saw_text,
        saw_done
    );
}

#[tokio::test]
#[ignore = "requires UMP_V2_LIVE_CODEX_REALTIME_AUDIO=1 and local Codex OAuth auth"]
async fn live_codex_realtime_transcription_session_when_opted_in() {
    let Some(_guard) = LiveGuard::from_env("realtime_transcription_session") else {
        return;
    };

    let response = reqwest::Client::new()
        .post(live_http_url("/v1/realtime/transcription_sessions"))
        .header(header::CONTENT_TYPE, "application/json")
        .json(&json!({
            "input_audio_format": "pcm16",
            "input_audio_transcription": {
                "model": "gpt-4o-transcribe"
            },
            "turn_detection": {
                "type": "server_vad"
            }
        }))
        .send()
        .await
        .expect("send live realtime transcription session request");

    let status = response.status();
    let request_id_hash = response_request_id_hash(&response);
    let text = response
        .text()
        .await
        .expect("read live realtime transcription session response");
    common::assert_no_unredacted_sensitive_values(&text);

    let body = parse_json_body(&text, "realtime_transcription_session");
    assert!(
        status.is_success(),
        "live realtime transcription session failed status={} request_id_hash={}",
        status.as_u16(),
        request_id_hash.as_deref().unwrap_or("none")
    );
    let event_names = response_event_names(&body);
    let session_id_present = string_at_any(&body, &["/id", "/client_secret/value"]).is_some();
    println!(
        "live_status endpoint_class=realtime_transcription_session status={} event_names={} session_or_secret_present={} request_id_hash={}",
        status.as_u16(),
        event_names.join(","),
        session_id_present,
        request_id_hash.as_deref().unwrap_or("none")
    );
}

#[tokio::test]
#[ignore = "requires UMP_V2_LIVE_CODEX_REALTIME_AUDIO=1 and local Codex OAuth auth"]
async fn live_codex_realtime_calls_invalid_offer_auth_passes_when_opted_in() {
    let Some(_guard) = LiveGuard::from_env("realtime_calls_invalid_offer") else {
        return;
    };

    let response = reqwest::Client::new()
        .post(live_http_url("/v1/realtime/calls"))
        .header(header::CONTENT_TYPE, "application/sdp")
        .body("v=0\r\no=- 0 0 IN IP4 127.0.0.1\r\ns=ump-v2-live-invalid-offer\r\n")
        .send()
        .await
        .expect("send live realtime calls invalid-offer request");

    let status = response.status();
    let request_id_hash = response_request_id_hash(&response);
    let text = response
        .text()
        .await
        .expect("read live realtime calls invalid-offer response");
    common::assert_no_unredacted_sensitive_values(&text);
    let body = parse_json_body(&text, "realtime_calls_invalid_offer");
    let code = body
        .pointer("/error/code")
        .and_then(Value::as_str)
        .unwrap_or("none");

    if status == StatusCode::NOT_FOUND {
        println!(
            "live_status endpoint_class=realtime_calls status={} route_exposed=false request_id_hash={}",
            status.as_u16(),
            request_id_hash.as_deref().unwrap_or("none")
        );
        return;
    }

    if status == StatusCode::NOT_IMPLEMENTED && code == "unsupported_feature" {
        println!(
            "live_status endpoint_class=realtime_calls status={} route_gated=true request_id_hash={}",
            status.as_u16(),
            request_id_hash.as_deref().unwrap_or("none")
        );
        return;
    }

    assert_ne!(
        status,
        StatusCode::UNAUTHORIZED,
        "live realtime calls invalid offer failed auth status={} code={} request_id_hash={}",
        status.as_u16(),
        code,
        request_id_hash.as_deref().unwrap_or("none")
    );
    assert!(
        status.is_client_error(),
        "live realtime calls invalid offer expected client error after auth status={} code={} request_id_hash={}",
        status.as_u16(),
        code,
        request_id_hash.as_deref().unwrap_or("none")
    );
    println!(
        "live_status endpoint_class=realtime_calls status={} route_gated=false auth_passed=true error_code={} request_id_hash={}",
        status.as_u16(),
        code,
        request_id_hash.as_deref().unwrap_or("none")
    );
}

#[tokio::test]
#[ignore = "requires UMP_V2_LIVE_CODEX_REALTIME_AUDIO=1, UMP_V2_LIVE_STT_AUDIO_FILE, and local Codex OAuth auth"]
async fn live_codex_audio_transcription_nonempty_spoken_fixture_when_opted_in() {
    let Some(_guard) = LiveGuard::from_env("audio_transcription_spoken_fixture") else {
        return;
    };
    let Some(audio_path) = common::optional_env(LIVE_STT_AUDIO_FILE).map(PathBuf::from) else {
        eprintln!("skipping live audio transcription: {LIVE_STT_AUDIO_FILE} is not set");
        return;
    };
    if !audio_path.is_file() {
        eprintln!("skipping live audio transcription: fixture_present=false");
        return;
    }

    let audio_bytes = std::fs::read(&audio_path).expect("read live STT audio fixture");
    let file_name = audio_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("fixture.wav");
    let (content_type, body) = multipart_audio_body(
        "umpv2livecodexrealtimeaudio",
        file_name,
        &audio_bytes,
        &[
            ("model", "gpt-4o-transcribe"),
            ("response_format", "json"),
            ("language", "en"),
        ],
    );

    let response = reqwest::Client::new()
        .post(live_http_url("/v1/audio/transcriptions"))
        .header(header::CONTENT_TYPE, content_type)
        .body(body)
        .send()
        .await
        .expect("send live audio transcription request");

    let status = response.status();
    let request_id_hash = response_request_id_hash(&response);
    let text = response
        .text()
        .await
        .expect("read live audio transcription response");
    common::assert_no_unredacted_sensitive_values(&text);
    let body = parse_json_body(&text, "audio_transcription");
    assert!(
        status.is_success(),
        "live audio transcription failed status={} request_id_hash={}",
        status.as_u16(),
        request_id_hash.as_deref().unwrap_or("none")
    );

    let transcript = string_at_any(&body, &["/text", "/transcript"])
        .or_else(|| first_segment_text(&body))
        .unwrap_or_default();
    let transcript_present = !transcript.trim().is_empty();
    let expected_phrase = common::optional_env(LIVE_STT_EXPECTED_PHRASE)
        .unwrap_or_else(|| EXPECTED_STT_PHRASE.to_string());
    let expected_phrase = expected_phrase.to_ascii_lowercase();
    let expected_phrase_present = transcript.to_ascii_lowercase().contains(&expected_phrase);
    assert!(
        transcript_present && expected_phrase_present,
        "live audio transcription did not return expected spoken fixture text: transcript_present={transcript_present} expected_phrase_present={expected_phrase_present}"
    );
    println!(
        "live_status endpoint_class=audio_transcription status={} transcript_present={} expected_phrase_present={} request_id_hash={}",
        status.as_u16(),
        transcript_present,
        expected_phrase_present,
        request_id_hash.as_deref().unwrap_or("none")
    );
}

struct LiveGuard;

impl LiveGuard {
    fn from_env(test_name: &str) -> Option<Self> {
        if common::optional_env(LIVE_OPT_IN).as_deref() != Some("1") {
            eprintln!("skipping {test_name}: {LIVE_OPT_IN}=1 is required");
            return None;
        }
        if std::env::var_os("CI").is_some()
            && common::optional_env(LIVE_CI_OPT_IN).as_deref() != Some("1")
        {
            eprintln!("skipping {test_name}: {LIVE_CI_OPT_IN}=1 is required in CI");
            return None;
        }
        if !codex_auth_path().is_file() {
            eprintln!("skipping {test_name}: codex_auth_present=false");
            return None;
        }
        Some(Self)
    }
}

fn codex_auth_path() -> PathBuf {
    dirs::home_dir()
        .expect("resolve home directory")
        .join(".codex")
        .join("auth.json")
}

fn live_http_url(path: &str) -> String {
    format!("{}{}", common::live_base_url(), path)
}

fn live_ws_url(path: &str, query: &[(&str, &str)]) -> String {
    let mut base = common::live_base_url();
    if let Some(rest) = base.strip_prefix("https://") {
        base = format!("wss://{rest}");
    } else if let Some(rest) = base.strip_prefix("http://") {
        base = format!("ws://{rest}");
    }
    let query = query
        .iter()
        .map(|(key, value)| format!("{key}={value}"))
        .collect::<Vec<_>>()
        .join("&");
    format!("{base}{path}?{query}")
}

async fn next_ws_json(ws: &mut specter::WebSocket) -> Option<Value> {
    loop {
        let message = match tokio::time::timeout(Duration::from_secs(30), ws.next()).await {
            Ok(Ok(Some(message))) => message,
            Ok(Ok(None)) => return None,
            Ok(Err(_)) => panic!("live realtime websocket read failed"),
            Err(_) => panic!("live realtime websocket read timed out"),
        };
        match message {
            SpecterMessage::Text(text) => {
                common::assert_no_unredacted_sensitive_values(&text);
                return Some(
                    serde_json::from_str(&text).expect("parse live realtime websocket JSON event"),
                );
            }
            SpecterMessage::Binary(bytes) => {
                return Some(
                    serde_json::from_slice(&bytes)
                        .expect("parse live realtime websocket binary JSON"),
                );
            }
            SpecterMessage::Ping(_) | SpecterMessage::Pong(_) => {}
            SpecterMessage::Close(_) => return None,
        }
    }
}

fn parse_json_body(text: &str, endpoint_class: &str) -> Value {
    serde_json::from_str(text).unwrap_or_else(|_| {
        panic!("live {endpoint_class} response was not JSON");
    })
}

fn response_request_id_hash(response: &reqwest::Response) -> Option<String> {
    response
        .headers()
        .get("x-request-id")
        .and_then(|value| value.to_str().ok())
        .map(hash_for_log)
}

fn hash_for_log(value: &str) -> String {
    let digest = Sha256::digest(value.as_bytes());
    digest[..8]
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn response_event_names(body: &Value) -> Vec<String> {
    ["/type", "/object", "/session/type"]
        .into_iter()
        .filter_map(|path| body.pointer(path).and_then(Value::as_str))
        .map(ToOwned::to_owned)
        .collect()
}

fn string_at_any(body: &Value, pointers: &[&str]) -> Option<String> {
    pointers
        .iter()
        .find_map(|pointer| body.pointer(pointer).and_then(Value::as_str))
        .map(ToOwned::to_owned)
}

fn first_segment_text(body: &Value) -> Option<String> {
    body.get("segments")
        .and_then(Value::as_array)
        .and_then(|segments| segments.first())
        .and_then(|segment| segment.get("text"))
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
}

fn multipart_audio_body(
    boundary: &str,
    file_name: &str,
    audio_bytes: &[u8],
    fields: &[(&str, &str)],
) -> (String, Vec<u8>) {
    let mut body = Vec::new();
    for (name, value) in fields {
        body.extend_from_slice(format!("--{boundary}\r\n").as_bytes());
        body.extend_from_slice(
            format!("Content-Disposition: form-data; name=\"{name}\"\r\n\r\n").as_bytes(),
        );
        body.extend_from_slice(value.as_bytes());
        body.extend_from_slice(b"\r\n");
    }
    body.extend_from_slice(format!("--{boundary}\r\n").as_bytes());
    body.extend_from_slice(
        format!(
            "Content-Disposition: form-data; name=\"file\"; filename=\"{}\"\r\n",
            sanitize_multipart_filename(file_name)
        )
        .as_bytes(),
    );
    body.extend_from_slice(b"Content-Type: audio/wav\r\n\r\n");
    body.extend_from_slice(audio_bytes);
    body.extend_from_slice(b"\r\n");
    body.extend_from_slice(format!("--{boundary}--\r\n").as_bytes());
    (format!("multipart/form-data; boundary={boundary}"), body)
}

fn sanitize_multipart_filename(file_name: &str) -> String {
    file_name
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '.' | '-' | '_'))
        .collect::<String>()
}
