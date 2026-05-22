use std::{net::SocketAddr, time::Duration};

use anyhow::Context;
use unified_model_proxy_v2::{build_router, AppState, RuntimeConfig};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "unified_model_proxy_v2=info,tower_http=info".into()),
        )
        .init();

    let runtime = RuntimeConfig::from_env().map_err(anyhow::Error::msg)?;
    let addr: SocketAddr = runtime.listen_addr;
    let state = AppState::from_env_with_config(runtime);
    unified_model_proxy_v2::upstream::codex::warm_codex_model_catalog_with_timeout(
        &state,
        Duration::from_secs(2),
    )
    .await;
    let _codex_catalog_refresher =
        unified_model_proxy_v2::upstream::codex::spawn_codex_model_catalog_refresher(state.clone());
    install_sighup_latch_reset(state.clone());
    let app = build_router(state);
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .with_context(|| format!("failed to bind {addr}"))?;

    tracing::info!(%addr, "ump-v2 listening");
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("server failed")?;
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        if let Err(err) = tokio::signal::ctrl_c().await {
            tracing::warn!(%err, "failed to install ctrl-c signal handler");
        }
    };

    #[cfg(unix)]
    let terminate = async {
        match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
            Ok(mut signal) => {
                signal.recv().await;
            }
            Err(err) => tracing::warn!(%err, "failed to install SIGTERM handler"),
        }
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}

#[cfg(unix)]
fn install_sighup_latch_reset(state: AppState) {
    tokio::spawn(async move {
        match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::hangup()) {
            Ok(mut signal) => {
                while signal.recv().await.is_some() {
                    state.reset_codex_wss_latch();
                    tracing::info!("reset Codex WSS latch after SIGHUP");
                }
            }
            Err(err) => tracing::warn!(%err, "failed to install SIGHUP handler"),
        }
    });
}

#[cfg(not(unix))]
fn install_sighup_latch_reset(_state: AppState) {}
