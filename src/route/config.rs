use axum::{body::Bytes, extract::State, response::Html, Json};
use serde_json::{json, Value};

use crate::{AppResult, AppState};

pub async fn config_page(State(state): State<AppState>) -> AppResult<Html<String>> {
    let config = state.routing_config.read_json()?;
    let pretty = serde_json::to_string_pretty(&config)?;
    Ok(Html(render_config_page(&pretty)))
}

pub async fn get_config(State(state): State<AppState>) -> AppResult<Json<Value>> {
    Ok(Json(state.routing_config.read_json()?))
}

pub async fn put_config(State(state): State<AppState>, body: Bytes) -> AppResult<Json<Value>> {
    let config: Value = serde_json::from_slice(&body)?;
    state.routing_config.write_json(&config)?;
    Ok(Json(json!({
        "ok": true,
        "config": config,
    })))
}

fn render_config_page(config_json: &str) -> String {
    format!(
        r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>Unified Model Proxy v2 Config</title>
  <style>
    body {{ margin: 0; padding: 32px; font-family: ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif; background: #111827; color: #f9fafb; }}
    main {{ max-width: 960px; margin: 0 auto; }}
    textarea {{ box-sizing: border-box; width: 100%; min-height: 440px; padding: 16px; border: 1px solid #374151; border-radius: 12px; background: #030712; color: #d1fae5; font: 14px/1.5 ui-monospace, SFMono-Regular, Menlo, monospace; }}
    button {{ margin-top: 12px; padding: 10px 14px; border: 0; border-radius: 10px; background: #10b981; color: #022c22; font-weight: 700; cursor: pointer; }}
    .row {{ display: flex; align-items: center; justify-content: space-between; gap: 16px; }}
    .hint, #status {{ color: #9ca3af; }}
    code {{ color: #93c5fd; }}
  </style>
</head>
<body>
  <main>
    <div class="row">
      <div>
        <h1>Unified Model Proxy v2</h1>
        <p class="hint">Edit hot routing JSON. Saves to <code>/api/config</code> and affects the next request.</p>
      </div>
      <button id="save" type="button">Save config</button>
    </div>
    <textarea id="config" spellcheck="false">{}</textarea>
    <p id="status">Ready.</p>
  </main>
  <script>
    const textarea = document.getElementById('config');
    const status = document.getElementById('status');
    document.getElementById('save').addEventListener('click', async () => {{
      status.textContent = 'Saving...';
      let parsed;
      try {{
        parsed = JSON.parse(textarea.value);
      }} catch (error) {{
        status.textContent = `Invalid JSON: ${{error.message}}`;
        return;
      }}
      const response = await fetch('/api/config', {{
        method: 'PUT',
        headers: {{ 'content-type': 'application/json' }},
        body: JSON.stringify(parsed),
      }});
      const body = await response.json().catch(() => ({{}}));
      if (!response.ok) {{
        status.textContent = body.error?.message || `Save failed: ${{response.status}}`;
        return;
      }}
      textarea.value = JSON.stringify(body.config, null, 2);
      status.textContent = 'Saved. New routing applies to the next request.';
    }});
  </script>
</body>
</html>"#,
        escape_html(config_json)
    )
}

fn escape_html(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
