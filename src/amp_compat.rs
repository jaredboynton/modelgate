use std::{
    collections::BTreeSet,
    env, io,
    path::{Path, PathBuf},
};

use axum::{
    body::Body,
    http::{header, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use base64::{engine::general_purpose, Engine as _};
use serde::Deserialize;
use serde_json::{json, Map, Value};
use tokio::fs;

const DEFAULT_LIST_LIMIT: usize = 200;
const DEFAULT_SEARCH_LIMIT: usize = 50;

#[derive(Clone, Debug)]
pub struct AmpStore {
    root: PathBuf,
}

#[derive(Debug, Deserialize)]
pub struct InternalEnvelope {
    method: Option<String>,
    params: Option<Value>,
}

#[derive(Debug, Deserialize)]
pub struct ThreadsFindQuery {
    #[serde(default)]
    pub q: Option<String>,
    #[serde(default)]
    pub limit: Option<usize>,
    #[serde(default)]
    pub offset: Option<usize>,
}

#[derive(Debug, Deserialize)]
pub struct ThreadsMarkdownQuery {
    #[serde(default)]
    pub truncate_tool_results: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AttachmentUpload {
    data: String,
    #[serde(rename = "mediaType")]
    media_type: String,
}

impl AmpStore {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    pub fn from_env() -> Self {
        Self::new(resolve_default_root())
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    fn threads_dir(&self) -> PathBuf {
        self.root.join("threads")
    }

    fn thread_file(&self, id: &str) -> PathBuf {
        self.threads_dir().join(format!("{}.json", safe_id(id)))
    }

    async fn ensure_threads_dir(&self) -> io::Result<()> {
        fs::create_dir_all(self.threads_dir()).await
    }

    pub async fn read_thread(&self, id: &str) -> io::Result<Option<Value>> {
        if id.is_empty() {
            return Ok(None);
        }

        match fs::read(self.thread_file(id)).await {
            Ok(bytes) => match serde_json::from_slice::<Value>(&bytes) {
                Ok(mut thread) => {
                    if let Some(obj) = thread.as_object_mut() {
                        obj.entry("id".to_string())
                            .or_insert_with(|| Value::String(id.to_string()));
                    }
                    Ok(Some(thread))
                }
                Err(_) => Ok(None),
            },
            Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(err) => Err(err),
        }
    }

    pub async fn write_thread(&self, thread: Value) -> io::Result<Value> {
        let Some(id) = thread.get("id").and_then(Value::as_str).map(str::to_owned) else {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "thread must include id",
            ));
        };

        self.ensure_threads_dir().await?;
        let existing = self.read_thread(&id).await?;
        let merged = merge_thread(existing, thread);
        write_json_atomically(&self.thread_file(&id), &merged).await?;
        Ok(merged)
    }

    pub async fn delete_thread(&self, id: &str) -> io::Result<bool> {
        match fs::remove_file(self.thread_file(id)).await {
            Ok(()) => Ok(true),
            Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(false),
            Err(err) => Err(err),
        }
    }

    pub async fn list_threads(
        &self,
        include_archived: bool,
        limit: Option<usize>,
    ) -> io::Result<Vec<Value>> {
        let mut entries: Vec<Value> = self
            .load_all_threads()
            .await?
            .into_iter()
            .filter(|thread| include_archived || !is_archived(thread))
            .map(|thread| {
                let mut entry = derive_thread_entry(&thread);
                if workspace_filter_mode() == WorkspaceFilterMode::AlwaysShow {
                    strip_workspace_uris(&mut entry);
                }
                entry
            })
            .collect();
        entries.sort_by_key(|entry| std::cmp::Reverse(thread_sort_key(entry)));
        entries.truncate(clamp_limit(limit, DEFAULT_LIST_LIMIT));
        Ok(entries)
    }

    pub async fn search_threads(
        &self,
        q: &str,
        limit: Option<usize>,
        offset: Option<usize>,
    ) -> io::Result<(Vec<Value>, bool)> {
        let query = q.trim().to_lowercase();
        let limit = clamp_limit(limit, DEFAULT_SEARCH_LIMIT);
        let offset = offset.unwrap_or(0);
        let mut entries: Vec<Value> = self
            .load_all_threads()
            .await?
            .into_iter()
            .filter(|thread| query.is_empty() || thread_matches(thread, &query))
            .map(|thread| {
                let mut entry = derive_thread_entry(&thread);
                if workspace_filter_mode() == WorkspaceFilterMode::AlwaysShow {
                    strip_workspace_uris(&mut entry);
                }
                entry
            })
            .collect();
        entries.sort_by_key(|entry| std::cmp::Reverse(thread_sort_key(entry)));

        let total = entries.len();
        let end = (offset + limit).min(total);
        let page = if offset >= total {
            Vec::new()
        } else {
            entries[offset..end].to_vec()
        };
        Ok((page, end < total))
    }

    pub async fn get_labels(&self, id: &str) -> io::Result<Vec<String>> {
        let Some(thread) = self.read_thread(id).await? else {
            return Ok(Vec::new());
        };
        Ok(thread_labels(&thread))
    }

    pub async fn set_labels(&self, id: &str, labels: Vec<String>) -> io::Result<Option<Value>> {
        let labels = labels.into_iter().map(Value::String).collect::<Vec<_>>();
        self.set_meta(id, json!({ "labels": labels })).await
    }

    pub async fn collect_all_labels(&self) -> io::Result<Vec<String>> {
        let mut labels = BTreeSet::new();
        for thread in self.load_all_threads().await? {
            for label in thread_labels(&thread) {
                if !label.is_empty() {
                    labels.insert(label);
                }
            }
        }
        Ok(labels.into_iter().collect())
    }

    pub async fn set_meta(&self, id: &str, meta: Value) -> io::Result<Option<Value>> {
        let Some(mut thread) = self.read_thread(id).await? else {
            return Ok(None);
        };

        let merged = match (thread.get("meta").cloned(), meta) {
            (Some(Value::Object(mut base)), Value::Object(extra)) => {
                base.extend(extra);
                Value::Object(base)
            }
            (_, other) => other,
        };
        if let Some(obj) = thread.as_object_mut() {
            obj.insert("meta".to_string(), merged);
        }
        Ok(Some(self.write_thread(thread).await?))
    }

    pub async fn archive_thread(&self, id: &str, archived: bool) -> io::Result<Option<Value>> {
        let Some(mut thread) = self.read_thread(id).await? else {
            return Ok(None);
        };
        if let Some(obj) = thread.as_object_mut() {
            obj.insert("archived".to_string(), Value::Bool(archived));
        }
        Ok(Some(self.write_thread(thread).await?))
    }

    pub async fn render_markdown(&self, id: &str, truncate: bool) -> io::Result<Option<String>> {
        let Some(thread) = self.read_thread(id).await? else {
            return Ok(None);
        };
        Ok(Some(render_thread_markdown(&thread, truncate)))
    }

    async fn load_all_threads(&self) -> io::Result<Vec<Value>> {
        let mut entries = match fs::read_dir(self.threads_dir()).await {
            Ok(entries) => entries,
            Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(err) => return Err(err),
        };

        let mut threads = Vec::new();
        while let Some(entry) = entries.next_entry().await? {
            if !entry
                .path()
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("json"))
            {
                continue;
            }
            let Ok(bytes) = fs::read(entry.path()).await else {
                continue;
            };
            let Ok(thread) = serde_json::from_slice::<Value>(&bytes) else {
                continue;
            };
            if thread.get("id").and_then(Value::as_str).is_some() {
                threads.push(thread);
            }
        }
        Ok(threads)
    }
}

pub async fn dispatch_internal(store: &AmpStore, query: &str, body: bytes::Bytes) -> Response {
    let envelope_result = if body.is_empty() {
        Ok(InternalEnvelope {
            method: None,
            params: None,
        })
    } else {
        serde_json::from_slice::<InternalEnvelope>(&body)
    };
    let envelope = match envelope_result {
        Ok(envelope) => envelope,
        Err(err) => {
            return amp_error("invalid-request", format!("Invalid JSON body: {err}"));
        }
    };
    let method = envelope
        .method
        .or_else(|| first_query_key(query))
        .unwrap_or_default();
    let params = envelope.params.unwrap_or_else(|| json!({}));

    match method.as_str() {
        "getUserInfo" => ok(json!({
            "id": "local-ump-v2-user",
            "email": "jared@ampcode.com",
            "name": "Jared Boynton",
            "features": [],
        })),
        "loadPlugins" => ok(Value::Array(Vec::new())),
        "getUserFreeTierStatus" => ok(json!({ "canUseAmpFree": false })),
        "getThread" => {
            let Some(id) = param_thread_id(&params) else {
                return amp_error("invalid-request", "getThread requires a thread id");
            };
            match store.read_thread(&id).await {
                Ok(Some(thread)) => ok(json!({ "thread": { "data": thread } })),
                Ok(None) => amp_error("thread-not-found", "Thread not found"),
                Err(err) => amp_error("internal-error", err.to_string()),
            }
        }
        "uploadThread" => {
            let Some(thread) = param_thread_object(&params) else {
                return amp_error(
                    "invalid-request",
                    "uploadThread requires a thread object with an id",
                );
            };
            match store.write_thread(thread).await {
                Ok(saved) => ok(json!({
                    "thread": {
                        "id": saved.get("id").cloned().unwrap_or(Value::Null),
                        "v": saved.get("v").cloned().unwrap_or_else(|| Value::Number(0_u64.into())),
                    }
                })),
                Err(err) => amp_error("internal-error", err.to_string()),
            }
        }
        "deleteThread" => {
            let Some(id) = param_thread_id(&params) else {
                return amp_error("invalid-request", "deleteThread requires a thread id");
            };
            match store.delete_thread(&id).await {
                Ok(deleted) => ok(json!({ "deleted": deleted })),
                Err(err) => amp_error("internal-error", err.to_string()),
            }
        }
        "listThreads" => {
            let include_archived = params
                .get("includeArchived")
                .and_then(Value::as_bool)
                .unwrap_or(false);
            let limit = params
                .get("limit")
                .and_then(Value::as_u64)
                .map(|value| value as usize);
            match store.list_threads(include_archived, limit).await {
                Ok(threads) => ok(json!({ "threads": threads })),
                Err(err) => amp_error("internal-error", err.to_string()),
            }
        }
        "searchThreads" => {
            let q = params.get("q").and_then(Value::as_str).unwrap_or("");
            let limit = params
                .get("limit")
                .and_then(Value::as_u64)
                .map(|value| value as usize);
            let offset = params
                .get("offset")
                .and_then(Value::as_u64)
                .map(|value| value as usize);
            match store.search_threads(q, limit, offset).await {
                Ok((threads, has_more)) => ok(json!({ "threads": threads, "hasMore": has_more })),
                Err(err) => amp_error("internal-error", err.to_string()),
            }
        }
        "setThreadMeta" => {
            let Some(id) = param_thread_id(&params) else {
                return amp_error("invalid-request", "setThreadMeta requires a thread id");
            };
            let meta = params.get("meta").cloned().unwrap_or_else(|| json!({}));
            match store.set_meta(&id, meta).await {
                Ok(Some(thread)) => ok(json!({
                    "thread": {
                        "id": thread.get("id").cloned().unwrap_or(Value::Null),
                        "meta": thread.get("meta").cloned().unwrap_or_else(|| json!({})),
                    }
                })),
                Ok(None) => amp_error("thread-not-found", "Thread not found"),
                Err(err) => amp_error("internal-error", err.to_string()),
            }
        }
        "archiveThread" => {
            let Some(id) = param_thread_id(&params) else {
                return amp_error("invalid-request", "archiveThread requires a thread id");
            };
            let archived = params
                .get("archived")
                .and_then(Value::as_bool)
                .unwrap_or(false);
            match store.archive_thread(&id, archived).await {
                Ok(Some(thread)) => ok(json!({
                    "thread": {
                        "id": thread.get("id").cloned().unwrap_or(Value::Null),
                        "archived": thread.get("archived").cloned().unwrap_or(Value::Bool(false)),
                    }
                })),
                Ok(None) => amp_error("thread-not-found", "Thread not found"),
                Err(err) => amp_error("internal-error", err.to_string()),
            }
        }
        "getThreadLabels" => {
            let Some(id) = param_thread_id(&params) else {
                return amp_error("invalid-request", "getThreadLabels requires a thread id");
            };
            match store.get_labels(&id).await {
                Ok(labels) => ok(json!({ "labels": labels })),
                Err(err) => amp_error("internal-error", err.to_string()),
            }
        }
        "setThreadLabels" => {
            let Some(id) = param_thread_id(&params) else {
                return amp_error("invalid-request", "setThreadLabels requires a thread id");
            };
            let labels = labels_from_params(&params);
            match store.set_labels(&id, labels.clone()).await {
                Ok(Some(_)) => ok(json!({ "labels": labels })),
                Ok(None) => amp_error("thread-not-found", "Thread not found"),
                Err(err) => amp_error("internal-error", err.to_string()),
            }
        }
        "addThreadLabels" => {
            let Some(id) = param_thread_id(&params) else {
                return amp_error("invalid-request", "addThreadLabels requires a thread id");
            };
            let mut labels = match store.get_labels(&id).await {
                Ok(labels) => labels,
                Err(err) => return amp_error("internal-error", err.to_string()),
            };
            for label in labels_from_params(&params) {
                if !labels.contains(&label) {
                    labels.push(label);
                }
            }
            match store.set_labels(&id, labels.clone()).await {
                Ok(Some(_)) => ok(json!({ "labels": labels })),
                Ok(None) => amp_error("thread-not-found", "Thread not found"),
                Err(err) => amp_error("internal-error", err.to_string()),
            }
        }
        "getUserLabels" => match store.collect_all_labels().await {
            Ok(labels) => ok(json!({ "labels": labels })),
            Err(err) => amp_error("internal-error", err.to_string()),
        },
        "shareThreadWithOperator" => ok(json!({
            "shared": false,
            "threadID": params.get("threadID").and_then(Value::as_str).map(str::to_owned).or_else(|| param_thread_id(&params)),
            "local": true,
        })),
        "getThreadLinkInfo" => ok(json!({
            "threadID": param_thread_id(&params),
            "url": Value::Null,
            "expiresAt": Value::Null,
            "visibility": "private",
            "local": true,
        })),
        "threadDisplayCostInfo" => ok(json!({
            "threadID": param_thread_id(&params),
            "totalCost": 0,
            "currency": "USD",
            "breakdown": [],
            "local": true,
        })),
        "userDisplayBalanceInfo" => ok(json!({
            "balance": { "amount": 0, "currency": "USD" },
            "plan": { "name": "local-proxy", "limits": {} },
            "local": true,
        })),
        "markAsReadMysteriousMessage" => ok(json!({ "marked": true })),
        "sendReport" => {
            persist_report(store.root(), &params).await;
            ok(json!({ "reportID": format!("local-{}", now_ms()) }))
        }
        "createTask" => match create_task(store.root(), &params).await {
            Ok(task) => ok(json!({ "task": task })),
            Err(err) if err.kind() == io::ErrorKind::InvalidInput => {
                amp_error("invalid-request", err.to_string())
            }
            Err(err) => amp_error("internal-error", err.to_string()),
        },
        "listTasks" => match list_tasks(store.root(), &params).await {
            Ok(tasks) => ok(json!({ "tasks": tasks })),
            Err(err) => amp_error("internal-error", err.to_string()),
        },
        "getTask" => {
            let Some(id) = param_task_id(&params) else {
                return amp_error("invalid-request", "getTask requires a task id");
            };
            match read_task(store.root(), &id).await {
                Ok(Some(task)) => ok(json!({ "task": task })),
                Ok(None) => amp_error("task-not-found", "Task not found"),
                Err(err) => amp_error("internal-error", err.to_string()),
            }
        }
        "updateTask" => {
            let Some(id) = param_task_id(&params) else {
                return amp_error("invalid-request", "updateTask requires a task id");
            };
            match update_task(store.root(), &id, &params).await {
                Ok(Some(task)) => ok(json!({ "task": task })),
                Ok(None) => amp_error("task-not-found", "Task not found"),
                Err(err) => amp_error("internal-error", err.to_string()),
            }
        }
        "deleteTask" => {
            let Some(id) = param_task_id(&params) else {
                return amp_error("invalid-request", "deleteTask requires a task id");
            };
            match delete_task(store.root(), &id).await {
                Ok(deleted) => ok(json!({ "deleted": deleted })),
                Err(err) => amp_error("internal-error", err.to_string()),
            }
        }
        "webSearch2" => amp_error(
            "not-available",
            "web search requires upstream Amp and is not handled by the local proxy",
        ),
        "extractWebPageContent" => amp_error(
            "not-available",
            "web page extraction requires upstream Amp and is not handled by the local proxy",
        ),
        other => amp_error(
            "not-found",
            format!("Unsupported Amp internal method: {other}"),
        ),
    }
}

pub async fn threads_find(store: &AmpStore, params: ThreadsFindQuery) -> Response {
    match store
        .search_threads(
            params.q.as_deref().unwrap_or_default(),
            params.limit,
            params.offset,
        )
        .await
    {
        Ok((threads, has_more)) => {
            Json(json!({ "threads": threads, "hasMore": has_more })).into_response()
        }
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response(),
    }
}

pub async fn thread_markdown(
    store: &AmpStore,
    file_name: String,
    query: ThreadsMarkdownQuery,
) -> Response {
    let Some(id) = file_name.strip_suffix(".md") else {
        return (StatusCode::NOT_FOUND, "not found").into_response();
    };
    let truncate = matches!(
        query.truncate_tool_results.as_deref(),
        Some("1") | Some("true")
    );
    match store.render_markdown(id, truncate).await {
        Ok(Some(markdown)) => {
            let mut response = Response::new(Body::from(markdown));
            response.headers_mut().insert(
                header::CONTENT_TYPE,
                HeaderValue::from_static("text/markdown; charset=utf-8"),
            );
            response.headers_mut().insert(
                header::ACCESS_CONTROL_ALLOW_ORIGIN,
                HeaderValue::from_static("*"),
            );
            response
        }
        Ok(None) => {
            let mut response = Response::new(Body::from("# Thread not found\n"));
            *response.status_mut() = StatusCode::NOT_FOUND;
            response.headers_mut().insert(
                header::CONTENT_TYPE,
                HeaderValue::from_static("text/markdown; charset=utf-8"),
            );
            response
        }
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_response(),
    }
}

pub async fn attachment_post(store: &AmpStore, origin: String, body: bytes::Bytes) -> Response {
    let parsed = match serde_json::from_slice::<AttachmentUpload>(&body) {
        Ok(parsed) => parsed,
        Err(_) => return amp_error("invalid-request", "attachment upload requires JSON body"),
    };
    if parsed.data.is_empty() {
        return amp_error("invalid-request", "attachment upload requires base64 data");
    }
    let bytes = match general_purpose::STANDARD.decode(parsed.data.as_bytes()) {
        Ok(bytes) if !bytes.is_empty() => bytes,
        _ => return amp_error("invalid-request", "attachment data must be base64"),
    };

    let id = format!(
        "att-{}-{:x}",
        now_ms(),
        std::process::id() ^ (bytes.len() as u32)
    );
    let ext = attachment_ext(&parsed.media_type);
    let dir = attachments_dir(store.root());
    if let Err(err) = fs::create_dir_all(&dir).await {
        return amp_error("dispatch-failed", err.to_string());
    }

    if let Err(err) = fs::write(dir.join(format!("{id}.{ext}")), &bytes).await {
        return amp_error("dispatch-failed", err.to_string());
    }
    let meta = json!({
        "id": id,
        "mediaType": parsed.media_type,
        "size": bytes.len(),
        "createdAt": now_ms(),
        "ext": ext,
    });
    if let Err(err) = write_json_atomically(&dir.join(format!("{id}.meta.json")), &meta).await {
        return amp_error("dispatch-failed", err.to_string());
    }

    Json(json!({ "url": format!("{}/api/attachments/{id}", origin.trim_end_matches('/')) }))
        .into_response()
}

pub async fn attachment_get(store: &AmpStore, id: String) -> Response {
    if !id.starts_with("att-") || id.contains('/') {
        return (StatusCode::NOT_FOUND, "attachment not found").into_response();
    }

    let dir = attachments_dir(store.root());
    let meta_bytes = match fs::read(dir.join(format!("{id}.meta.json"))).await {
        Ok(bytes) => bytes,
        Err(_) => return (StatusCode::NOT_FOUND, "attachment not found").into_response(),
    };
    let meta = serde_json::from_slice::<Value>(&meta_bytes).unwrap_or_else(|_| json!({}));
    let ext = meta.get("ext").and_then(Value::as_str).unwrap_or("bin");
    let media_type = meta
        .get("mediaType")
        .and_then(Value::as_str)
        .unwrap_or("application/octet-stream");
    let bytes = match fs::read(dir.join(format!("{id}.{ext}"))).await {
        Ok(bytes) => bytes,
        Err(_) => return (StatusCode::NOT_FOUND, "attachment not found").into_response(),
    };

    let mut response = Response::new(Body::from(bytes));
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_str(media_type)
            .unwrap_or_else(|_| HeaderValue::from_static("application/octet-stream")),
    );
    response.headers_mut().insert(
        header::ACCESS_CONTROL_ALLOW_ORIGIN,
        HeaderValue::from_static("*"),
    );
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("private, max-age=31536000, immutable"),
    );
    response
}

pub fn decode_request_body(headers: &axum::http::HeaderMap, body: bytes::Bytes) -> bytes::Bytes {
    use std::io::Read;

    let encoding = headers
        .get(header::CONTENT_ENCODING)
        .and_then(|value| value.to_str().ok())
        .map(str::to_ascii_lowercase)
        .unwrap_or_default();
    match encoding.as_str() {
        "gzip" | "x-gzip" => {
            let mut decoder = flate2::read::GzDecoder::new(&body[..]);
            let mut out = Vec::with_capacity(body.len().saturating_mul(2));
            if decoder.read_to_end(&mut out).is_ok() {
                bytes::Bytes::from(out)
            } else {
                body
            }
        }
        "deflate" => {
            let mut decoder = flate2::read::ZlibDecoder::new(&body[..]);
            let mut out = Vec::with_capacity(body.len().saturating_mul(2));
            if decoder.read_to_end(&mut out).is_ok() {
                bytes::Bytes::from(out)
            } else {
                body
            }
        }
        _ => body,
    }
}

fn resolve_default_root() -> PathBuf {
    if let Ok(value) = env::var("UMP_AMP_THREAD_STORE") {
        let trimmed = value.trim();
        if !trimmed.is_empty() {
            return PathBuf::from(trimmed);
        }
    }
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join(".unified-model-proxy")
        .join("amp-threads")
}

async fn write_json_atomically(path: &Path, value: &Value) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).await?;
    }
    let bytes = serde_json::to_vec_pretty(value).map_err(io_other)?;
    let tmp = path.with_extension(format!("tmp-{}", std::process::id()));
    fs::write(&tmp, &bytes).await?;
    fs::rename(&tmp, path).await?;
    Ok(())
}

fn io_other(err: impl std::fmt::Display) -> io::Error {
    io::Error::other(err.to_string())
}

fn safe_id(id: &str) -> String {
    id.chars()
        .map(|ch| match ch {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '.' | '_' | '-' => ch,
            _ => '_',
        })
        .collect()
}

fn clamp_limit(limit: Option<usize>, fallback: usize) -> usize {
    match limit {
        Some(value) if value > 0 => value.min(1000),
        _ => fallback,
    }
}

fn merge_thread(existing: Option<Value>, incoming: Value) -> Value {
    let mut merged = existing
        .and_then(|value| match value {
            Value::Object(map) => Some(map),
            _ => None,
        })
        .unwrap_or_default();
    let incoming = match incoming {
        Value::Object(map) => map,
        _ => Map::new(),
    };
    let existing_v = merged.get("v").and_then(Value::as_u64).unwrap_or_default();
    let incoming_v = incoming.get("v").and_then(Value::as_u64);
    let created = merged.get("created").and_then(Value::as_u64);

    for (key, value) in incoming {
        merged.insert(key, value);
    }
    merged.insert(
        "v".to_string(),
        Value::Number(existing_v.max(incoming_v.unwrap_or(existing_v)).into()),
    );
    if merged.get("created").and_then(Value::as_u64).is_none() {
        merged.insert(
            "created".to_string(),
            Value::Number(created.unwrap_or_else(now_ms).into()),
        );
    }
    if merged
        .get("userLastInteractedAt")
        .and_then(Value::as_u64)
        .is_none()
    {
        merged.insert(
            "userLastInteractedAt".to_string(),
            Value::Number(now_ms().into()),
        );
    }
    Value::Object(merged)
}

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or(0)
}

fn is_archived(thread: &Value) -> bool {
    thread
        .get("archived")
        .and_then(Value::as_bool)
        .unwrap_or(false)
}

fn message_count(thread: &Value) -> u64 {
    if let Some(messages) = thread.get("messages").and_then(Value::as_array) {
        return messages
            .iter()
            .filter(|message| {
                if message.get("role").and_then(Value::as_str) != Some("user") {
                    return false;
                }
                match message.get("content") {
                    Some(Value::String(_)) => true,
                    Some(Value::Array(blocks)) => blocks.iter().any(|block| {
                        block.get("type").and_then(Value::as_str) != Some("tool_result")
                    }),
                    _ => false,
                }
            })
            .count() as u64;
    }
    thread
        .get("summaryStats")
        .and_then(|stats| stats.get("messageCount"))
        .and_then(Value::as_u64)
        .unwrap_or(0)
}

fn derive_thread_entry(thread: &Value) -> Value {
    let count = message_count(thread);
    let mut entry = Map::new();
    entry.insert(
        "id".to_string(),
        thread.get("id").cloned().unwrap_or(Value::Null),
    );
    entry.insert(
        "v".to_string(),
        thread.get("v").cloned().unwrap_or(Value::Null),
    );
    entry.insert(
        "title".to_string(),
        thread.get("title").cloned().unwrap_or(Value::Null),
    );
    if let Some(created) = thread.get("created").and_then(Value::as_u64) {
        entry.insert("created".to_string(), Value::Number(created.into()));
    }
    if let Some(last) = user_last_interacted_at(thread) {
        entry.insert(
            "userLastInteractedAt".to_string(),
            Value::Number(last.into()),
        );
    }
    entry.insert("messageCount".to_string(), Value::Number(count.into()));
    entry.insert(
        "summaryStats".to_string(),
        json!({ "diffStats": diff_stats(thread), "messageCount": count }),
    );
    for key in [
        "env",
        "originThreadID",
        "mainThreadID",
        "relationships",
        "agentMode",
        "creatorUserID",
    ] {
        if let Some(value) = thread.get(key) {
            entry.insert(key.to_string(), value.clone());
        }
    }
    if !entry.contains_key("relationships") {
        entry.insert("relationships".to_string(), Value::Array(Vec::new()));
    }
    if let Some(meta) = thread.get("meta").and_then(canonical_meta) {
        entry.insert("meta".to_string(), meta);
    }
    if let Some(archived) = thread.get("archived").and_then(Value::as_bool) {
        entry.insert("archived".to_string(), Value::Bool(archived));
    }
    entry.insert(
        "usesDtw".to_string(),
        Value::Bool(
            thread
                .get("meta")
                .and_then(|meta| meta.get("usesDtw"))
                .and_then(Value::as_bool)
                .unwrap_or(false),
        ),
    );
    entry.insert(
        "usesThreadActors".to_string(),
        Value::Bool(
            thread
                .get("meta")
                .and_then(|meta| meta.get("usesThreadActors"))
                .and_then(Value::as_bool)
                .unwrap_or(false),
        ),
    );
    Value::Object(entry)
}

fn canonical_meta(meta: &Value) -> Option<Value> {
    let obj = meta.as_object()?;
    let visibility = obj.get("visibility").and_then(Value::as_str)?;
    if !matches!(
        visibility,
        "private" | "public_unlisted" | "public_discoverable" | "thread_workspace_shared"
    ) {
        return None;
    }
    let shared = obj
        .get("sharedGroupIDs")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(|value| value.as_str().map(|item| Value::String(item.to_string())))
                .collect()
        })
        .unwrap_or_else(Vec::new);
    Some(json!({ "visibility": visibility, "sharedGroupIDs": shared }))
}

fn diff_stats(thread: &Value) -> Value {
    let stats = thread
        .get("summaryStats")
        .and_then(|value| value.get("diffStats"));
    json!({
        "added": stats.and_then(|value| value.get("added")).and_then(Value::as_u64).unwrap_or(0),
        "changed": stats.and_then(|value| value.get("changed")).and_then(Value::as_u64).unwrap_or(0),
        "deleted": stats.and_then(|value| value.get("deleted")).and_then(Value::as_u64).unwrap_or(0),
    })
}

fn user_last_interacted_at(thread: &Value) -> Option<u64> {
    let mut values = Vec::new();
    if let Some(created) = thread.get("created").and_then(Value::as_u64) {
        values.push(created);
    }
    if let Some(interacted) = thread.get("userLastInteractedAt").and_then(Value::as_u64) {
        values.push(interacted);
    }
    if let Some(messages) = thread.get("messages").and_then(Value::as_array) {
        for message in messages {
            if message.get("role").and_then(Value::as_str) == Some("user") {
                if let Some(sent_at) = message
                    .get("meta")
                    .and_then(|meta| meta.get("sentAt"))
                    .and_then(Value::as_u64)
                {
                    values.push(sent_at);
                }
            }
        }
    }
    values.into_iter().max()
}

fn thread_sort_key(entry: &Value) -> u64 {
    entry
        .get("userLastInteractedAt")
        .and_then(Value::as_u64)
        .or_else(|| entry.get("created").and_then(Value::as_u64))
        .unwrap_or(0)
}

fn thread_matches(thread: &Value, q_lower: &str) -> bool {
    if thread
        .get("title")
        .and_then(Value::as_str)
        .is_some_and(|title| title.to_lowercase().contains(q_lower))
    {
        return true;
    }
    if let Some(env) = thread.get("env") {
        for key in ["repo", "tree"] {
            if env
                .get(key)
                .and_then(Value::as_str)
                .is_some_and(|value| value.to_lowercase().contains(q_lower))
            {
                return true;
            }
        }
    }
    thread
        .get("messages")
        .and_then(Value::as_array)
        .is_some_and(|messages| {
            messages.iter().any(|message| {
                collect_message_text(message)
                    .to_lowercase()
                    .contains(q_lower)
            })
        })
}

fn collect_message_text(message: &Value) -> String {
    if let Some(text) = message.get("content").and_then(Value::as_str) {
        return text.to_string();
    }
    let Some(blocks) = message.get("content").and_then(Value::as_array) else {
        return String::new();
    };
    let mut out = String::new();
    for block in blocks {
        for key in ["text", "content"] {
            if let Some(text) = block.get(key).and_then(Value::as_str) {
                out.push_str(text);
                out.push('\n');
            }
        }
    }
    out
}

fn render_thread_markdown(thread: &Value, truncate: bool) -> String {
    let title = thread
        .get("title")
        .and_then(Value::as_str)
        .filter(|title| !title.trim().is_empty())
        .map(str::to_owned)
        .unwrap_or_else(|| {
            format!(
                "Thread {}",
                thread.get("id").and_then(Value::as_str).unwrap_or("")
            )
        });
    let mut out = format!("# {title}\n\n");
    if let Some(id) = thread.get("id").and_then(Value::as_str) {
        out.push_str(&format!("- Thread ID: `{id}`\n"));
    }
    if let Some(mode) = thread.get("agentMode").and_then(Value::as_str) {
        out.push_str(&format!("- Agent mode: `{mode}`\n"));
    }
    if let Some(env) = thread.get("env") {
        if let Some(repo) = env.get("repo").and_then(Value::as_str) {
            out.push_str(&format!("- Repo: `{repo}`\n"));
        }
        if let Some(tree) = env.get("tree").and_then(Value::as_str) {
            out.push_str(&format!("- Tree: `{tree}`\n"));
        }
    }
    out.push_str(&format!("- Messages: {}\n\n", message_count(thread)));

    if let Some(messages) = thread.get("messages").and_then(Value::as_array) {
        for message in messages {
            let role = message
                .get("role")
                .and_then(Value::as_str)
                .unwrap_or("info");
            out.push_str(&format!("## {role}\n\n"));
            for block in normalize_blocks(message.get("content")) {
                render_block(&block, truncate, &mut out);
            }
            out.push('\n');
        }
    }
    out
}

fn normalize_blocks(content: Option<&Value>) -> Vec<Value> {
    match content {
        Some(Value::String(text)) => vec![json!({ "type": "text", "text": text })],
        Some(Value::Array(blocks)) => blocks.clone(),
        _ => Vec::new(),
    }
}

fn render_block(block: &Value, truncate: bool, out: &mut String) {
    match block.get("type").and_then(Value::as_str).unwrap_or("text") {
        "text" => {
            if let Some(text) = block.get("text").and_then(Value::as_str) {
                out.push_str(text);
                out.push_str("\n\n");
            }
        }
        "tool_use" => {
            let name = block.get("name").and_then(Value::as_str).unwrap_or("tool");
            let input = block.get("input").cloned().unwrap_or_else(|| json!({}));
            out.push_str(&format!("**tool_use: {name}**\n\n```json\n"));
            out.push_str(&json_pretty(&input));
            out.push_str("\n```\n\n");
        }
        "tool_result" => {
            out.push_str("**tool_result**\n\n```\n");
            let text = match block.get("content") {
                Some(Value::String(text)) => text.clone(),
                Some(value) => json_pretty(value),
                None => String::new(),
            };
            if truncate && text.len() > 2000 {
                out.push_str(&text[..2000]);
                out.push_str(&format!("\n... [truncated {} chars]", text.len() - 2000));
            } else {
                out.push_str(&text);
            }
            out.push_str("\n```\n\n");
        }
        _ => {
            if let Some(text) = block.get("text").and_then(Value::as_str) {
                out.push_str(text);
                out.push_str("\n\n");
            } else {
                out.push_str("```json\n");
                out.push_str(&json_pretty(block));
                out.push_str("\n```\n\n");
            }
        }
    }
}

fn json_pretty(value: &Value) -> String {
    serde_json::to_string_pretty(value).unwrap_or_else(|_| "null".to_string())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum WorkspaceFilterMode {
    AlwaysShow,
    Client,
    Strict,
}

fn workspace_filter_mode() -> WorkspaceFilterMode {
    match env::var("UMP_AMP_WORKSPACE_FILTER")
        .ok()
        .map(|value| value.trim().to_lowercase())
        .as_deref()
    {
        Some("client") => WorkspaceFilterMode::Client,
        Some("strict") => WorkspaceFilterMode::Strict,
        _ => WorkspaceFilterMode::AlwaysShow,
    }
}

fn strip_workspace_uris(entry: &mut Value) {
    let Some(trees) = entry
        .get_mut("env")
        .and_then(Value::as_object_mut)
        .and_then(|env| env.get_mut("initial"))
        .and_then(Value::as_object_mut)
        .and_then(|initial| initial.get_mut("trees"))
        .and_then(Value::as_array_mut)
    else {
        return;
    };
    for tree in trees {
        if let Some(tree) = tree.as_object_mut() {
            tree.remove("uri");
        }
    }
}

fn thread_labels(thread: &Value) -> Vec<String> {
    thread
        .get("meta")
        .and_then(|meta| meta.get("labels"))
        .and_then(Value::as_array)
        .map(|labels| {
            labels
                .iter()
                .filter_map(|label| label.as_str().map(str::to_owned))
                .collect()
        })
        .unwrap_or_default()
}

fn labels_from_params(params: &Value) -> Vec<String> {
    params
        .get("labels")
        .and_then(Value::as_array)
        .map(|labels| {
            labels
                .iter()
                .filter_map(|label| label.as_str().map(str::to_owned))
                .collect()
        })
        .unwrap_or_default()
}

fn param_thread_id(params: &Value) -> Option<String> {
    if let Some(thread) = params.get("thread") {
        if let Some(id) = thread.as_str() {
            return Some(id.to_string());
        }
        if let Some(id) = thread.get("id").and_then(Value::as_str) {
            return Some(id.to_string());
        }
    }
    params
        .get("threadID")
        .and_then(Value::as_str)
        .map(str::to_owned)
}

fn param_thread_object(params: &Value) -> Option<Value> {
    let thread = params.get("thread")?;
    if thread.is_object() && thread.get("id").and_then(Value::as_str).is_some() {
        Some(thread.clone())
    } else {
        None
    }
}

fn reports_dir(root: &Path) -> PathBuf {
    root.join("reports")
}

async fn persist_report(root: &Path, payload: &Value) {
    let dir = reports_dir(root);
    if fs::create_dir_all(&dir).await.is_err() {
        return;
    }
    let body = json!({ "at": now_ms(), "payload": payload });
    let file = dir.join(format!("{}-{}.json", now_ms(), std::process::id()));
    let _ = write_json_atomically(&file, &body).await;
}

fn tasks_dir(root: &Path) -> PathBuf {
    root.join("tasks")
}

fn task_file(root: &Path, id: &str) -> PathBuf {
    tasks_dir(root).join(format!("{}.json", safe_id(id)))
}

fn param_task_id(params: &Value) -> Option<String> {
    for key in ["taskID", "id", "task_id"] {
        if let Some(value) = params.get(key).and_then(Value::as_str) {
            if !value.is_empty() {
                return Some(value.to_string());
            }
        }
    }
    params
        .get("task")
        .and_then(|task| task.get("id"))
        .and_then(Value::as_str)
        .filter(|id| !id.is_empty())
        .map(str::to_owned)
}

async fn read_task(root: &Path, id: &str) -> io::Result<Option<Value>> {
    match fs::read(task_file(root, id)).await {
        Ok(bytes) => Ok(serde_json::from_slice::<Value>(&bytes).ok()),
        Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(err) => Err(err),
    }
}

async fn write_task(root: &Path, task: &Value) -> io::Result<()> {
    let id = task
        .get("id")
        .and_then(Value::as_str)
        .filter(|id| !id.is_empty())
        .unwrap_or("unknown");
    write_json_atomically(&task_file(root, id), task).await
}

async fn create_task(root: &Path, params: &Value) -> io::Result<Value> {
    let id = param_task_id(params).unwrap_or_else(|| {
        format!(
            "task-{}-{:x}",
            now_ms(),
            std::process::id() ^ (now_ms() as u32)
        )
    });
    let now = now_ms();
    let task = json!({
        "id": id,
        "threadID": params.get("threadID").and_then(Value::as_str).map(str::to_owned),
        "status": params.get("status").and_then(Value::as_str).unwrap_or("queued"),
        "createdAt": now,
        "updatedAt": now,
        "payload": params.get("payload").cloned().unwrap_or_else(|| params.clone()),
    });
    write_task(root, &task).await?;
    Ok(task)
}

async fn update_task(root: &Path, id: &str, params: &Value) -> io::Result<Option<Value>> {
    let Some(mut task) = read_task(root, id).await? else {
        return Ok(None);
    };
    if let Some(obj) = task.as_object_mut() {
        if let Some(status) = params.get("status").and_then(Value::as_str) {
            obj.insert("status".to_string(), Value::String(status.to_string()));
        }
        if let Some(thread_id) = params.get("threadID").and_then(Value::as_str) {
            obj.insert("threadID".to_string(), Value::String(thread_id.to_string()));
        }
        if let Some(payload) = params.get("payload") {
            let mut merged = obj
                .get("payload")
                .and_then(Value::as_object)
                .cloned()
                .unwrap_or_default();
            if let Some(extra) = payload.as_object() {
                merged.extend(extra.clone());
            }
            obj.insert("payload".to_string(), Value::Object(merged));
        }
        obj.insert("updatedAt".to_string(), Value::Number(now_ms().into()));
    }
    write_task(root, &task).await?;
    Ok(Some(task))
}

async fn delete_task(root: &Path, id: &str) -> io::Result<bool> {
    match fs::remove_file(task_file(root, id)).await {
        Ok(()) => Ok(true),
        Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(err) => Err(err),
    }
}

async fn list_tasks(root: &Path, params: &Value) -> io::Result<Vec<Value>> {
    let thread_filter = params
        .get("threadID")
        .and_then(Value::as_str)
        .map(str::to_owned);
    let status_filter = params
        .get("status")
        .and_then(Value::as_str)
        .map(str::to_owned);
    let mut entries = match fs::read_dir(tasks_dir(root)).await {
        Ok(entries) => entries,
        Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(err) => return Err(err),
    };

    let mut tasks = Vec::new();
    while let Some(entry) = entries.next_entry().await? {
        if !entry
            .path()
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("json"))
        {
            continue;
        }
        let Ok(bytes) = fs::read(entry.path()).await else {
            continue;
        };
        let Ok(task) = serde_json::from_slice::<Value>(&bytes) else {
            continue;
        };
        if let Some(thread_id) = thread_filter.as_deref() {
            if task.get("threadID").and_then(Value::as_str) != Some(thread_id) {
                continue;
            }
        }
        if let Some(status) = status_filter.as_deref() {
            if task.get("status").and_then(Value::as_str) != Some(status) {
                continue;
            }
        }
        tasks.push(task);
    }
    tasks.sort_by(|left, right| {
        let left = left.get("updatedAt").and_then(Value::as_u64).unwrap_or(0);
        let right = right.get("updatedAt").and_then(Value::as_u64).unwrap_or(0);
        right.cmp(&left)
    });
    Ok(tasks)
}

fn attachments_dir(root: &Path) -> PathBuf {
    root.join("attachments")
}

fn attachment_ext(media_type: &str) -> &'static str {
    match media_type.to_ascii_lowercase().as_str() {
        "image/png" => "png",
        "image/jpeg" | "image/jpg" => "jpg",
        "image/gif" => "gif",
        "image/webp" => "webp",
        "image/svg+xml" => "svg",
        _ => "bin",
    }
}

fn first_query_key(query: &str) -> Option<String> {
    query
        .split('&')
        .next()
        .filter(|pair| !pair.is_empty())
        .map(|pair| match pair.split_once('=') {
            Some((key, _)) => key.to_string(),
            None => pair.to_string(),
        })
}

fn ok(result: Value) -> Response {
    Json(json!({ "ok": true, "result": result })).into_response()
}

fn amp_error(code: &str, message: impl Into<String>) -> Response {
    let message = message.into();
    tracing::warn!(%code, %message, "Amp compatibility request failed");
    Json(json!({
        "ok": false,
        "error": {
            "code": code,
            "message": message,
        }
    }))
    .into_response()
}
