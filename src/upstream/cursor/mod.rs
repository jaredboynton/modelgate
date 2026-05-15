//! Native Cursor Composer transport stack.
//!
//! Owns HTTP/2 + Connect framing, the manual protobuf wire layer, and the
//! `GetUsableModels` discovery cache. Higher-level run/session/indexing logic
//! lives in sibling modules owned by other lanes.

pub mod client_profile;
pub mod connect;
pub mod indexing;
pub mod models;
pub mod profiles;
pub mod proto;
pub mod run;
pub mod session;
pub mod transport;
pub mod workspace;

/// Cursor AgentService host. Used for streaming Run and unary discovery.
pub const CURSOR_API_HOST: &str = "api2.cursor.sh";

/// Cursor RepositoryService host. Used by the cloud-indexing lane.
pub const CURSOR_REPO_HOST: &str = "repo42.cursor.sh";

/// Bidirectional Composer Run RPC path.
pub const CURSOR_RUN_PATH: &str = "/agent.v1.AgentService/Run";

/// Unary model-discovery RPC path.
pub const CURSOR_GET_USABLE_MODELS_PATH: &str = "/agent.v1.AgentService/GetUsableModels";

/// Default `x-cursor-client-version` shipped to AgentService.
///
/// Override at runtime via the `CURSOR_CLIENT_VERSION` env var; live Phase 0
/// is responsible for verifying that the server still accepts this build
/// string before promoting any change.
pub const DEFAULT_CURSOR_CLIENT_VERSION: &str = "cli-2026.01.09-231024f";

/// Preflight Cursor credentials for route-level status-code contracts.
///
/// Route handlers call this through the upstream boundary so they do not reach
/// into auth internals directly; the run engine still resolves credentials
/// again when opening the real provider stream.
pub async fn ensure_credentials(state: &crate::AppState) -> crate::AppResult<()> {
    crate::auth::cursor::resolve_cursor_credentials(&state.auth_home)
        .await
        .map(|_| ())
}

pub async fn fetch_usable_models_for_state(
    state: &crate::AppState,
) -> crate::AppResult<Vec<models::ModelDescriptor>> {
    let credentials = crate::auth::cursor::resolve_cursor_credentials(&state.auth_home).await?;
    Ok(models::fetch_usable_models(&credentials.access_token).await)
}

/// Resolves the active `x-cursor-client-version` header value.
///
/// Returns the env override when present, otherwise the pinned default.
pub fn cursor_client_version() -> String {
    std::env::var("CURSOR_CLIENT_VERSION").unwrap_or_else(|_| DEFAULT_CURSOR_CLIENT_VERSION.into())
}
