use std::io::Write;

use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
};
use flate2::{write::GzEncoder, Compression};
use serde_json::{json, Value};
use tempfile::TempDir;
use tower::ServiceExt;
use unified_model_proxy_v2::{build_router, AppState};

struct Harness {
    _temp: TempDir,
    app: axum::Router,
}

impl Harness {
    fn new() -> Self {
        let temp = tempfile::tempdir().expect("tempdir");
        let state = AppState::for_tests(temp.path().join("codex"), temp.path().join("auth"));
        let app = build_router(state);
        Self { _temp: temp, app }
    }

    async fn post_internal(&self, method: &str, params: Value) -> Value {
        let request = Request::builder()
            .method("POST")
            .uri(format!("/api/internal?{method}"))
            .header("content-type", "application/json")
            .body(Body::from(
                json!({ "method": method, "params": params }).to_string(),
            ))
            .expect("request");
        let response = self.app.clone().oneshot(request).await.expect("response");
        assert_eq!(response.status(), StatusCode::OK);
        read_json(response).await
    }
}

async fn read_json(response: axum::response::Response) -> Value {
    let bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    serde_json::from_slice(&bytes).expect("json")
}

fn sample_thread(id: &str, last_interacted: u64) -> Value {
    json!({
        "id": id,
        "v": 1,
        "created": 1_700_000_000_000u64,
        "userLastInteractedAt": last_interacted,
        "title": format!("Thread {id}"),
        "agentMode": "default",
        "env": {
            "repo": "demo-repo",
            "initial": { "trees": [{ "uri": "file:///tmp/demo", "name": "demo" }] }
        },
        "messages": [
            { "role": "user", "content": [{ "type": "text", "text": "hello world" }] },
            { "role": "assistant", "content": [{ "type": "text", "text": "hi there" }] }
        ]
    })
}

#[tokio::test]
async fn amp_internal_upload_then_get_persists_thread() {
    let harness = Harness::new();
    let uploaded = harness
        .post_internal(
            "uploadThread",
            json!({ "thread": sample_thread("T-aaa", 1_700_000_000_000) }),
        )
        .await;
    assert_eq!(uploaded["ok"], true);
    assert_eq!(uploaded["result"]["thread"]["id"], "T-aaa");

    let fetched = harness
        .post_internal("getThread", json!({ "thread": "T-aaa" }))
        .await;
    assert_eq!(fetched["ok"], true);
    assert_eq!(fetched["result"]["thread"]["data"]["id"], "T-aaa");
    assert_eq!(
        fetched["result"]["thread"]["data"]["messages"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
}

#[tokio::test]
async fn amp_internal_list_search_archive_and_delete_threads() {
    let harness = Harness::new();
    let older = sample_thread("T-older", 1_700_000_000_000);
    let mut newer = sample_thread("T-newer", 1_700_000_100_000);
    newer["title"] = Value::String("Refactor parser".to_string());
    let mut archived = sample_thread("T-archived", 1_700_000_200_000);
    archived["archived"] = Value::Bool(true);

    for thread in [older, newer, archived] {
        harness
            .post_internal("uploadThread", json!({ "thread": thread }))
            .await;
    }

    let listed = harness
        .post_internal("listThreads", json!({ "includeArchived": false }))
        .await;
    let ids: Vec<&str> = listed["result"]["threads"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|thread| thread["id"].as_str())
        .collect();
    assert_eq!(ids, vec!["T-newer", "T-older"]);
    assert!(listed["result"]["threads"][0]["env"]["initial"]["trees"][0]
        .get("uri")
        .is_none());

    let search = harness
        .post_internal("searchThreads", json!({ "q": "refactor", "limit": 10 }))
        .await;
    let search_ids: Vec<&str> = search["result"]["threads"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|thread| thread["id"].as_str())
        .collect();
    assert_eq!(search_ids, vec!["T-newer"]);
    assert_eq!(search["result"]["hasMore"], false);

    let archive = harness
        .post_internal(
            "archiveThread",
            json!({ "thread": "T-older", "archived": true }),
        )
        .await;
    assert_eq!(archive["result"]["thread"]["archived"], true);

    let delete = harness
        .post_internal("deleteThread", json!({ "thread": "T-newer" }))
        .await;
    assert_eq!(delete["result"]["deleted"], true);
    let missing = harness
        .post_internal("getThread", json!({ "thread": "T-newer" }))
        .await;
    assert_eq!(missing["ok"], false);
    assert_eq!(missing["error"]["code"], "thread-not-found");
}

#[tokio::test]
async fn amp_internal_labels_merge_and_user_labels() {
    let harness = Harness::new();
    harness
        .post_internal(
            "uploadThread",
            json!({ "thread": sample_thread("T-labels", 1_700_000_000_000) }),
        )
        .await;

    let set = harness
        .post_internal(
            "setThreadLabels",
            json!({ "thread": "T-labels", "labels": ["alpha", "beta"] }),
        )
        .await;
    assert_eq!(set["result"]["labels"], json!(["alpha", "beta"]));

    let added = harness
        .post_internal(
            "addThreadLabels",
            json!({ "thread": "T-labels", "labels": ["beta", "gamma"] }),
        )
        .await;
    let mut labels: Vec<&str> = added["result"]["labels"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(Value::as_str)
        .collect();
    labels.sort();
    assert_eq!(labels, vec!["alpha", "beta", "gamma"]);

    let user_labels = harness.post_internal("getUserLabels", json!({})).await;
    assert_eq!(
        user_labels["result"]["labels"],
        json!(["alpha", "beta", "gamma"])
    );
}

#[tokio::test]
async fn amp_internal_tasks_round_trip() {
    let harness = Harness::new();
    let created = harness
        .post_internal(
            "createTask",
            json!({
                "threadID": "T-task",
                "status": "queued",
                "payload": { "hello": "world" }
            }),
        )
        .await;
    let id = created["result"]["task"]["id"]
        .as_str()
        .unwrap()
        .to_string();

    let fetched = harness
        .post_internal("getTask", json!({ "taskID": id }))
        .await;
    assert_eq!(fetched["result"]["task"]["threadID"], "T-task");

    let updated = harness
        .post_internal(
            "updateTask",
            json!({
                "taskID": id,
                "status": "completed",
                "payload": { "extra": 1 }
            }),
        )
        .await;
    assert_eq!(updated["result"]["task"]["status"], "completed");
    assert_eq!(updated["result"]["task"]["payload"]["hello"], "world");
    assert_eq!(updated["result"]["task"]["payload"]["extra"], 1);

    let listed = harness
        .post_internal("listTasks", json!({ "threadID": "T-task" }))
        .await;
    assert_eq!(
        listed["result"]["tasks"][0]["id"],
        updated["result"]["task"]["id"]
    );

    let deleted = harness
        .post_internal("deleteTask", json!({ "taskID": id }))
        .await;
    assert_eq!(deleted["result"]["deleted"], true);
    let missing = harness
        .post_internal("getTask", json!({ "taskID": id }))
        .await;
    assert_eq!(missing["error"]["code"], "task-not-found");
}

#[tokio::test]
async fn api_threads_find_and_markdown_routes_use_store() {
    let harness = Harness::new();
    let mut thread = sample_thread("T-md", 1_700_000_000_000);
    thread["title"] = Value::String("Markdown demo".to_string());
    harness
        .post_internal("uploadThread", json!({ "thread": thread }))
        .await;

    let find = harness
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/threads/find?q=markdown&limit=10")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(find.status(), StatusCode::OK);
    let body = read_json(find).await;
    assert_eq!(body["threads"][0]["id"], "T-md");

    let markdown = harness
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/threads/T-md.md?truncate_tool_results=1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(markdown.status(), StatusCode::OK);
    assert_eq!(
        markdown
            .headers()
            .get("content-type")
            .unwrap()
            .to_str()
            .unwrap(),
        "text/markdown; charset=utf-8"
    );
    let bytes = to_bytes(markdown.into_body(), usize::MAX).await.unwrap();
    let text = String::from_utf8(bytes.to_vec()).unwrap();
    assert!(text.contains("# Markdown demo"));
    assert!(text.contains("Thread ID: `T-md`"));
    assert!(text.contains("hello world"));

    let missing = harness
        .app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/threads/T-missing.md")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(missing.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn api_attachments_upload_and_get_round_trip() {
    let harness = Harness::new();
    let upload = harness
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/attachments")
                .header("host", "localhost:18743")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({ "data": "cG5nLWlzaA==", "mediaType": "image/png" }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(upload.status(), StatusCode::OK);
    let body = read_json(upload).await;
    let url = body["url"].as_str().unwrap();
    assert!(url.starts_with("http://localhost:18743/api/attachments/"));
    let id = url.rsplit('/').next().unwrap();

    let fetched = harness
        .app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/attachments/{id}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(fetched.status(), StatusCode::OK);
    assert_eq!(
        fetched
            .headers()
            .get("content-type")
            .unwrap()
            .to_str()
            .unwrap(),
        "image/png"
    );
    let bytes = to_bytes(fetched.into_body(), usize::MAX).await.unwrap();
    assert_eq!(&bytes[..], b"png-ish");
}

#[tokio::test]
async fn amp_internal_gzip_upload_thread_body_is_decoded() {
    let harness = Harness::new();
    let body = json!({
        "method": "uploadThread",
        "params": { "thread": sample_thread("T-gzip", 1_700_000_000_000) }
    })
    .to_string();
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(body.as_bytes()).unwrap();
    let compressed = encoder.finish().unwrap();

    let response = harness
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/internal?uploadThread")
                .header("content-type", "application/json")
                .header("content-encoding", "gzip")
                .body(Body::from(compressed))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = read_json(response).await;
    assert_eq!(body["ok"], true);

    let fetched = harness
        .post_internal("getThread", json!({ "thread": "T-gzip" }))
        .await;
    assert_eq!(fetched["result"]["thread"]["data"]["id"], "T-gzip");
}

#[tokio::test]
async fn compatibility_helper_routes_are_local_and_safe() {
    let harness = Harness::new();
    let bitbucket = harness
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/internal/bitbucket-instance-url")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(bitbucket.status(), StatusCode::OK);
    assert_eq!(read_json(bitbucket).await, json!({}));

    let github = harness
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/internal/github-auth-status")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(github.status(), StatusCode::OK);
    assert_eq!(read_json(github).await, json!({ "authenticated": false }));

    let github_proxy = harness
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/internal/github-proxy/repos/octocat/Hello-World")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(github_proxy.status(), StatusCode::OK);
    assert_eq!(
        read_json(github_proxy).await["error"]["code"],
        "provider-auth-failed"
    );

    let rss = harness
        .app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/news.rss")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(rss.status(), StatusCode::OK);
}

#[tokio::test]
async fn api_telemetry_accepts_json_and_sets_cors() {
    let harness = Harness::new();
    let telemetry = harness
        .app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/telemetry")
                .header("content-type", "application/json")
                .body(Body::from("[]"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(telemetry.status(), StatusCode::NO_CONTENT);
    assert_eq!(
        telemetry
            .headers()
            .get("access-control-allow-origin")
            .unwrap(),
        "*"
    );
}
