use serde_json::{json, Value};
use tokio::io::{AsyncBufRead, AsyncBufReadExt, AsyncReadExt, AsyncWrite, AsyncWriteExt};

use crate::AppResult;

use super::{run_fast_context, FastContextRequest};

const SERVER_NAME: &str = "fast-context-mcp";
const PROTOCOL_VERSION: &str = "2025-06-18";

pub async fn serve_stdio() -> AppResult<()> {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();
    serve(tokio::io::BufReader::new(stdin), stdout).await
}

pub async fn serve<R, W>(mut reader: R, mut writer: W) -> AppResult<()>
where
    R: AsyncBufRead + Unpin,
    W: AsyncWrite + Unpin,
{
    loop {
        let Some(request) = read_mcp_message(&mut reader).await? else {
            break;
        };
        if let Some(response) = handle_request(request).await {
            write_mcp_message(&mut writer, &response).await?;
        }
    }
    Ok(())
}

async fn read_mcp_message<R>(reader: &mut R) -> AppResult<Option<Value>>
where
    R: AsyncBufRead + Unpin,
{
    let mut first_line = Vec::new();
    loop {
        first_line.clear();
        let bytes = reader.read_until(b'\n', &mut first_line).await?;
        if bytes == 0 {
            return Ok(None);
        }
        if !first_line.iter().all(u8::is_ascii_whitespace) {
            break;
        }
    }

    let first = String::from_utf8_lossy(&first_line);
    if !first.to_ascii_lowercase().starts_with("content-length:") {
        return Ok(Some(serde_json::from_slice(first.trim().as_bytes())?));
    }

    let length = first
        .split_once(':')
        .and_then(|(_, value)| value.trim().parse::<usize>().ok())
        .ok_or_else(|| crate::AppError::BadRequest("invalid MCP Content-Length".into()))?;

    let mut header_line = Vec::new();
    loop {
        header_line.clear();
        let bytes = reader.read_until(b'\n', &mut header_line).await?;
        if bytes == 0 {
            return Ok(None);
        }
        if header_line == b"\n" || header_line == b"\r\n" {
            break;
        }
    }

    let mut body = vec![0; length];
    reader.read_exact(&mut body).await?;
    Ok(Some(serde_json::from_slice(&body)?))
}

async fn write_mcp_message<W>(writer: &mut W, response: &Value) -> AppResult<()>
where
    W: AsyncWrite + Unpin,
{
    let body = serde_json::to_vec(response)?;
    writer
        .write_all(format!("Content-Length: {}\r\n\r\n", body.len()).as_bytes())
        .await?;
    writer.write_all(&body).await?;
    writer.flush().await?;
    Ok(())
}

pub async fn handle_request(request: Value) -> Option<Value> {
    let id = request.get("id").cloned().unwrap_or(Value::Null);
    let method = request.get("method").and_then(Value::as_str).unwrap_or("");
    let result = match method {
        "initialize" => Ok(initialize_result()),
        "notifications/initialized" => return None,
        "tools/list" => Ok(tools_list_result()),
        "tools/call" => {
            handle_tools_call(request.get("params").cloned().unwrap_or(Value::Null)).await
        }
        _ => Err(json_rpc_error(
            -32601,
            format!("method not found: {method}"),
        )),
    };

    Some(match result {
        Ok(result) => json!({ "jsonrpc": "2.0", "id": id, "result": result }),
        Err(error) => json!({ "jsonrpc": "2.0", "id": id, "error": error }),
    })
}

fn initialize_result() -> Value {
    json!({
        "protocolVersion": PROTOCOL_VERSION,
        "serverInfo": {
            "name": SERVER_NAME,
            "version": env!("CARGO_PKG_VERSION")
        },
        "capabilities": {
            "tools": {}
        }
    })
}

fn tools_list_result() -> Value {
    json!({
        "tools": [{
            "name": "fast_context",
            "description": "Find relevant code context in a repository using a bounded read-only search loop.",
            "inputSchema": {
                "type": "object",
                "additionalProperties": false,
                "properties": {
                    "search_string": {
                        "type": "string",
                        "description": "Natural-language question/description about the code you want to understand. This tool does NOT accept regex, keyword dumps, or symbol-only queries."
                    },
                    "repo_path": {
                        "type": "string",
                        "description": "The absolute path of the folder where the search should be performed. In multi-repo workspaces, specify a subfolder to avoid searching across all repos."
                    },
                    "search_type": {
                        "type": "string",
                        "enum": ["all", "node_modules"],
                        "description": "Search type hint. Use 'node_modules' when searching inside node_modules or other dependency directories that are normally excluded."
                    },
                    "execution_mode": {
                        "type": "string",
                        "enum": ["windsurf", "local"],
                        "default": "windsurf",
                        "description": "Execution backend. 'windsurf' calls Windsurf upstream swe-grep models; 'local' uses the local read-only fallback search."
                    },
                    "model": {
                        "type": "string",
                        "enum": ["both", "swe-grep-mini", "swe-grep"],
                        "default": "both",
                        "description": "Windsurf Fast Context model selection. 'both' tries swe-grep-mini first, then swe-grep."
                    },
                    "fallback_local": {
                        "type": "boolean",
                        "default": false,
                        "description": "When true, return local read-only search results if the Windsurf upstream model call fails."
                    },
                    "max_files": {
                        "type": "integer",
                        "minimum": 1,
                        "maximum": 64,
                        "default": 16
                    },
                    "max_turns": {
                        "type": "integer",
                        "minimum": 1,
                        "maximum": 4,
                        "default": 4
                    }
                },
                "required": ["search_string", "repo_path"]
            }
        }]
    })
}

async fn handle_tools_call(params: Value) -> Result<Value, Value> {
    let name = params.get("name").and_then(Value::as_str).unwrap_or("");
    if name != "fast_context" {
        return Err(json_rpc_error(-32602, format!("unknown tool: {name}")));
    }
    let arguments = params.get("arguments").cloned().unwrap_or(Value::Null);
    let request = serde_json::from_value::<FastContextRequest>(arguments)
        .map_err(|error| json_rpc_error(-32602, format!("invalid arguments: {error}")))?;
    let response = run_fast_context(request)
        .await
        .map_err(|error| json_rpc_error(-32000, error.to_string()))?;
    let text = serde_json::to_string_pretty(&response)
        .map_err(|error| json_rpc_error(-32000, error.to_string()))?;
    Ok(json!({
        "content": [{
            "type": "text",
            "text": text
        }],
        "structuredContent": response,
        "isError": false
    }))
}

fn json_rpc_error(code: i64, message: impl Into<String>) -> Value {
    json!({
        "code": code,
        "message": message.into()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn lists_fast_context_tool() {
        let response = handle_request(json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/list"
        }))
        .await
        .unwrap();

        assert_eq!(response["result"]["tools"][0]["name"], "fast_context");
    }

    #[tokio::test]
    async fn serve_handles_content_length_framing() {
        let request = serde_json::to_string(&json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/list"
        }))
        .unwrap();
        let input = format!("Content-Length: {}\r\n\r\n{}", request.len(), request);
        let mut output = Vec::new();

        serve(tokio::io::BufReader::new(input.as_bytes()), &mut output)
            .await
            .unwrap();
        let output = String::from_utf8(output).unwrap();

        assert!(output.starts_with("Content-Length: "));
        assert!(output.contains("fast_context"));
        assert!(output.contains("search_string"));
        assert!(output.contains("repo_path"));
    }
}
