pub mod adapter;
pub mod amp_compat;
pub mod auth;
pub mod codex_catalog;
pub mod compaction;
pub mod config_graph;
pub mod cursor_agent;
pub mod error;
pub mod failure_capture;
pub mod hot_config;
pub mod model_alias;
pub mod rate_limit;
pub mod request_body;
pub mod route;
pub mod router;
pub mod sse;
pub mod state;
pub mod upstream;
pub mod upstream_response;

#[cfg(test)]
pub(crate) static TEST_ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

pub use error::{AppError, AppResult};
pub use router::build_router;
pub use state::{AppState, RuntimeConfig};
pub use upstream_response::UpstreamResponse;
