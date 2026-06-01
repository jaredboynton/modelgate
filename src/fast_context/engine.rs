use std::time::Instant;

use serde::{Deserialize, Serialize};

use crate::{upstream::windsurf, AppError, AppResult, AppState};

use super::{
    cascade_wire::{build_swe_grep_request, SWE_GREP_MINI_MODEL, SWE_GREP_MODEL},
    sandbox::RepoSandbox,
    tools::{search_repo, ContextSnippet},
};

#[derive(Debug, Clone, Deserialize)]
pub struct FastContextRequest {
    #[serde(alias = "query")]
    pub search_string: String,
    #[serde(alias = "repo_root")]
    pub repo_path: String,
    #[serde(default)]
    pub search_type: SearchType,
    #[serde(default)]
    pub execution_mode: ExecutionMode,
    #[serde(default)]
    pub model: FastContextModel,
    #[serde(default = "default_fallback_local")]
    pub fallback_local: bool,
    #[serde(default = "default_max_files")]
    pub max_files: usize,
    #[serde(default = "default_max_turns")]
    pub max_turns: usize,
}

#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SearchType {
    #[default]
    All,
    NodeModules,
}

#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionMode {
    #[default]
    Windsurf,
    Local,
}

#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize, Eq, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum FastContextModel {
    #[default]
    Both,
    SweGrepMini,
    SweGrep,
}

impl FastContextModel {
    fn models(self) -> &'static [&'static str] {
        match self {
            Self::Both => &[SWE_GREP_MINI_MODEL, SWE_GREP_MODEL],
            Self::SweGrepMini => &[SWE_GREP_MINI_MODEL],
            Self::SweGrep => &[SWE_GREP_MODEL],
        }
    }
}

#[derive(Debug, Clone, Serialize, Eq, PartialEq)]
pub struct FastContextResponse {
    pub model: String,
    pub snippets: Vec<ContextSnippet>,
    pub upstream_text: Option<String>,
    pub diagnostics: FastContextDiagnostics,
}

#[derive(Debug, Clone, Serialize, Eq, PartialEq)]
pub struct FastContextDiagnostics {
    pub latency_ms: u128,
    pub turns_used: usize,
    pub tool_calls_used: usize,
    pub mode: String,
    pub attempted_models: Vec<String>,
    pub upstream_error: Option<String>,
}

pub async fn run_fast_context(request: FastContextRequest) -> AppResult<FastContextResponse> {
    let started = Instant::now();
    let max_files = request.max_files.clamp(1, 64);
    let max_turns = request.max_turns.clamp(1, 4);

    if request.execution_mode == ExecutionMode::Windsurf {
        match run_windsurf_fast_context(&request, started, max_turns).await {
            Ok(response) => return Ok(response),
            Err(error) if request.fallback_local => {
                return run_local_fast_context(
                    &request,
                    started,
                    max_files,
                    max_turns,
                    Some(error.to_string()),
                );
            }
            Err(error) => return Err(error),
        }
    }

    run_local_fast_context(&request, started, max_files, max_turns, None)
}

async fn run_windsurf_fast_context(
    request: &FastContextRequest,
    started: Instant,
    max_turns: usize,
) -> AppResult<FastContextResponse> {
    let state = AppState::from_env();
    let mut attempted_models = Vec::new();
    let mut errors = Vec::new();
    for model in request.model.models() {
        attempted_models.push((*model).to_string());
        let upstream_request =
            build_swe_grep_request(&request.search_string, &request.repo_path, model);
        match windsurf::collect_chat_text(&state, &upstream_request, model).await {
            Ok(text) => {
                return Ok(FastContextResponse {
                    model: (*model).to_string(),
                    snippets: Vec::new(),
                    upstream_text: Some(text),
                    diagnostics: FastContextDiagnostics {
                        latency_ms: started.elapsed().as_millis(),
                        turns_used: max_turns.min(1),
                        tool_calls_used: 1,
                        mode: "windsurf_upstream".to_string(),
                        attempted_models,
                        upstream_error: None,
                    },
                });
            }
            Err(error) => {
                errors.push(format!("{model}: {error}"));
            }
        }
    }

    Err(AppError::Upstream(format!(
        "Windsurf Fast Context models failed: {}",
        if errors.is_empty() {
            "no model attempted".to_string()
        } else {
            errors.join("; ")
        }
    )))
}

fn run_local_fast_context(
    request: &FastContextRequest,
    started: Instant,
    max_files: usize,
    max_turns: usize,
    upstream_error: Option<String>,
) -> AppResult<FastContextResponse> {
    let sandbox = RepoSandbox::new(&request.repo_path)?;
    let snippets = search_repo(
        &sandbox,
        &request.search_string,
        max_files,
        request.search_type,
    )?;

    Ok(FastContextResponse {
        model: "local-read-only".to_string(),
        upstream_text: None,
        diagnostics: FastContextDiagnostics {
            latency_ms: started.elapsed().as_millis(),
            turns_used: max_turns.min(1),
            tool_calls_used: 1,
            mode: "local_read_only".to_string(),
            attempted_models: Vec::new(),
            upstream_error,
        },
        snippets,
    })
}

fn default_max_files() -> usize {
    16
}

fn default_max_turns() -> usize {
    4
}

fn default_fallback_local() -> bool {
    false
}
