#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum CodexWsProtocol {
    Rfc6455,
}

pub fn parse_codex_ws_protocol(value: Option<&str>) -> Result<CodexWsProtocol, String> {
    match value.unwrap_or("rfc6455") {
        "rfc6455" => Ok(CodexWsProtocol::Rfc6455),
        "rfc8441" | "rfc9220" => {
            Err("Codex WSS v0.1 only supports RFC 6455; H2/H3 lanes need capture evidence".into())
        }
        other => Err(format!("unknown Codex WebSocket protocol: {other}")),
    }
}
