#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    if args.iter().any(|arg| arg == "--help" || arg == "-h") {
        println!("fast-context-mcp {}", env!("CARGO_PKG_VERSION"));
        println!();
        println!("Usage: fast-context-mcp [allowed-directory ...]");
        println!();
        println!("Runs a stdio MCP server exposing the fast_context tool.");
        return Ok(());
    }
    if args
        .iter()
        .any(|arg| arg == "--version" || arg == "-v" || arg == "-V")
    {
        println!("fast-context-mcp {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "unified_model_proxy_v2=info".into()),
        )
        .with_writer(std::io::stderr)
        .init();

    unified_model_proxy_v2::fast_context::mcp::serve_stdio()
        .await
        .map_err(anyhow::Error::msg)
}
