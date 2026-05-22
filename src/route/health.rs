use crate::build_info;
use axum::Json;
use serde_json::{json, Value};

pub async fn health() -> Json<Value> {
    Json(json!({
        "status": "ok",
        "version": build_info::version(),
        "git_revision": build_info::git_revision(),
        "build_time_utc": build_info::build_time_utc(),
    }))
}
