use std::{
    collections::{BTreeMap, BTreeSet},
    time::{SystemTime, UNIX_EPOCH},
};

use serde::Serialize;
use serde_json::Value;

use crate::{
    hot_config::{
        parse_source_format, parse_target_format, source_format_matches, validate_no_secret_keys,
        ConfiguredRoute, ConfiguredSource, ConfiguredTarget, RoutingConfigFile,
    },
    model_alias::{Provider, TargetFormat, KNOWN_MODELS},
    AppError, AppResult,
};

const SCHEMA_VERSION: u32 = 2;
const CONTRACT_VERSION: &str = "config_graph.v2";
const WILDCARD_FORMAT: &str = "*";

#[derive(Debug, Clone, Serialize)]
pub struct ConfigGraph {
    pub schema_version: u32,
    pub contract_version: &'static str,
    pub generated_at: String,
    pub raw_hot_config: Value,
    pub sources: Vec<GraphSource>,
    pub runtime_formats: Vec<RuntimeFormat>,
    pub config_routes: Vec<ConfigRouteProjection>,
    pub effective_routes: Vec<EffectiveRouteProjection>,
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
    pub diagnostics: Vec<ConfigDiagnostic>,
    pub validation_issues: Vec<ConfigValidationIssue>,
    pub draft_status: DraftStatus,
    pub groups: Vec<GroupV2>,
    pub focal: Option<FocalV2>,
    pub route_cards: Vec<RouteCardV2>,
    pub diagnostics_v2: Vec<DiagnosticV2>,
}

#[derive(Debug, Clone, Serialize)]
pub struct GraphSource {
    pub model: String,
    pub origin: GraphRouteOrigin,
    pub source_provider: SourceProvider,
}

#[derive(Debug, Clone, Serialize)]
pub struct RuntimeFormat {
    pub format: &'static str,
    pub runtime_active: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ConfigRouteProjection {
    pub row_id: String,
    pub config_index: usize,
    pub enabled: bool,
    pub mutable: bool,
    pub source_model: String,
    pub source_provider: SourceProvider,
    pub source_runtime_format: String,
    pub target_provider: Provider,
    pub target_model: String,
    pub target_provider_format: String,
    pub state: ConfigRouteState,
}

#[derive(Debug, Clone, Serialize)]
pub struct EffectiveRouteProjection {
    pub route_id: String,
    pub origin: GraphRouteOrigin,
    pub mutable: bool,
    pub source_model: String,
    pub source_provider: SourceProvider,
    pub source_runtime_format: &'static str,
    pub target_provider: Provider,
    pub target_model: String,
    pub target_provider_format: String,
    pub config_index: Option<usize>,
    pub row_id: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct GraphNode {
    pub id: String,
    pub label: String,
    pub kind: GraphNodeKind,
    pub provider: Option<Provider>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_provider: Option<SourceProvider>,
    pub mutable: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct GraphEdge {
    pub id: String,
    pub source: String,
    pub target: String,
    pub route_id: String,
    pub source_runtime_format: &'static str,
    pub source_provider: SourceProvider,
    pub origin: GraphRouteOrigin,
}

#[derive(Debug, Clone, Serialize)]
pub struct ConfigDiagnostic {
    pub severity: DiagnosticSeverity,
    pub code: &'static str,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config_index: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub row_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runtime_format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_provider: Option<Provider>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_model: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ConfigValidationIssue {
    pub code: &'static str,
    pub message: String,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DraftStatus {
    Valid,
    Invalid,
    PartiallyProjected,
}

#[derive(Debug, Clone, Serialize)]
pub struct GroupV2 {
    pub id: String,
    pub label: String,
    pub kind: GroupKindV2,
    pub order: usize,
    pub route_ids: Vec<String>,
    pub collapsed_default: bool,
    pub counts: GroupCountsV2,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<Provider>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_provider: Option<SourceProvider>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runtime_format: Option<String>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GroupKindV2 {
    Provider,
    RuntimeFormat,
    State,
    SourceFamily,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct GroupCountsV2 {
    pub total: usize,
    pub effective: usize,
    pub configured: usize,
    pub errors: usize,
    pub warnings: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct FocalV2 {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub route_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub row_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runtime_format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_provider: Option<Provider>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RouteCardV2 {
    pub id: String,
    pub origin: GraphRouteOrigin,
    pub state: RouteCardState,
    pub group_ids: Vec<String>,
    pub order: usize,
    pub mutable: bool,
    pub enabled: bool,
    pub source: RouteCardSourceV2,
    pub target: RouteCardTargetV2,
    pub summary: String,
    pub diagnostic_ids: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config_index: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub row_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub precedence_label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw_config_route: Option<Value>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RouteCardSourceV2 {
    pub model: String,
    pub source_provider: SourceProvider,
    pub runtime_format: String,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceProvider {
    Cursor,
    Openai,
    Anthropic,
    Google,
    Custom,
}

#[derive(Debug, Clone, Serialize)]
pub struct RouteCardTargetV2 {
    pub provider: Provider,
    pub model: String,
    pub provider_format: Option<String>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RouteCardState {
    Effective,
    Disabled,
    Shadowed,
    Inactive,
    Invalid,
}

#[derive(Debug, Clone, Serialize)]
pub struct DiagnosticV2 {
    pub id: String,
    pub severity: DiagnosticSeverity,
    pub code: &'static str,
    pub message: String,
    pub blocking: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config_index: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub row_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub route_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub field: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggestion: Option<String>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GraphRouteOrigin {
    Catalog,
    HotConfig,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ConfigRouteState {
    Active,
    Disabled,
    Shadowed,
    Inactive,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GraphNodeKind {
    SourceModel,
    TargetModel,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticSeverity {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone)]
struct WinningHotRoute {
    config_index: usize,
    row_id: String,
    target_provider: Provider,
    target_model: String,
    target_provider_format: String,
}

#[derive(Debug, Clone)]
struct DraftRouteV2 {
    config_index: usize,
    row_id: String,
    enabled: bool,
    source_model: String,
    source_runtime_format: String,
    target_provider: Provider,
    target_model: String,
    target_provider_format: Option<String>,
    raw_config_route: Value,
}

#[derive(Debug)]
struct DraftProjectionV2 {
    config: RoutingConfigFile,
    routes: Vec<DraftRouteV2>,
    diagnostics: Vec<DiagnosticV2>,
    omitted_rows: bool,
}

const RUNTIME_FORMATS: [&str; 3] = ["responses", "chat_completions", "anthropic_messages"];
const ACCEPTED_INACTIVE_FORMATS: [&str; 2] = ["google_generate_content", "openai_images"];

pub fn build_config_graph(raw_hot_config: Value) -> AppResult<ConfigGraph> {
    validate_no_secret_keys(&raw_hot_config)?;
    validate_known_config_fields(&raw_hot_config)?;
    let draft = project_draft_routes(&raw_hot_config);
    build_config_graph_from_valid(raw_hot_config, &draft)
}

fn build_config_graph_from_valid(
    raw_hot_config: Value,
    draft: &DraftProjectionV2,
) -> AppResult<ConfigGraph> {
    let config = &draft.config;
    let mut diagnostics = Vec::new();
    let mut config_routes = Vec::new();
    let mut source_models = BTreeSet::new();

    for (config_index, route) in config.routes.iter().enumerate() {
        source_models.insert(route.source.model.clone());
        let row_id = row_id(config_index);
        let source_runtime_format = route
            .source
            .format
            .clone()
            .unwrap_or_else(|| WILDCARD_FORMAT.to_string());
        let target_provider_format =
            target_provider_format(route.target.provider, route.target.format.as_deref())?;
        let state = if !route.enabled {
            ConfigRouteState::Disabled
        } else if !is_active_source_format(&source_runtime_format) {
            ConfigRouteState::Inactive
        } else {
            ConfigRouteState::Active
        };
        config_routes.push(ConfigRouteProjection {
            row_id,
            config_index,
            enabled: route.enabled,
            mutable: true,
            source_model: route.source.model.clone(),
            source_provider: source_provider_for_model(&route.source.model),
            source_runtime_format,
            target_provider: route.target.provider,
            target_model: route.target.model.clone(),
            target_provider_format,
            state,
        });
    }

    for model in KNOWN_MODELS {
        source_models.insert(model.id.to_string());
    }

    if config.routes.is_empty() {
        diagnostics.push(diagnostic(
            DiagnosticSeverity::Info,
            "no_hot_overrides",
            "No hot overrides. Catalog routes are effective.",
            None,
            None,
            None,
        ));
    }

    let mut effective_routes = Vec::new();

    for source_model in &source_models {
        for runtime_format in RUNTIME_FORMATS {
            let winner = winning_hot_route(config, source_model, runtime_format, &mut diagnostics)?;
            if let Some(winner) = winner {
                effective_routes.push(EffectiveRouteProjection {
                    route_id: format!("effective:{}:{runtime_format}", source_model),
                    origin: GraphRouteOrigin::HotConfig,
                    mutable: true,
                    source_model: source_model.clone(),
                    source_provider: source_provider_for_model(source_model),
                    source_runtime_format: runtime_format,
                    target_provider: winner.target_provider,
                    target_model: winner.target_model,
                    target_provider_format: winner.target_provider_format,
                    config_index: Some(winner.config_index),
                    row_id: Some(winner.row_id),
                });
                continue;
            }

            if let Some(catalog_route) = catalog_effective_route(source_model, runtime_format) {
                effective_routes.push(catalog_route);
            }
        }
    }

    mark_shadowed_config_routes(&mut config_routes, &diagnostics);
    let sources = build_sources(&source_models, &config_routes);
    let (nodes, edges) = build_nodes_and_edges(&effective_routes);
    let mut diagnostics_v2 = draft.diagnostics.clone();
    diagnostics_v2.extend(convert_diagnostics_v2(&diagnostics));
    let route_cards = build_route_cards(
        &effective_routes,
        &config_routes,
        &draft.routes,
        &diagnostics_v2,
    );
    let groups = build_groups(&route_cards, &diagnostics_v2);
    let draft_status = draft_status(
        draft.omitted_rows,
        has_projected_safe_row(draft),
        &diagnostics_v2,
    );

    Ok(ConfigGraph {
        schema_version: SCHEMA_VERSION,
        contract_version: CONTRACT_VERSION,
        generated_at: generated_at(),
        raw_hot_config,
        sources,
        runtime_formats: runtime_formats(),
        config_routes,
        effective_routes,
        nodes,
        edges,
        diagnostics,
        validation_issues: Vec::new(),
        draft_status,
        groups,
        focal: None,
        route_cards,
        diagnostics_v2,
    })
}

fn validate_known_config_fields(value: &Value) -> AppResult<()> {
    let Some(root) = value.as_object() else {
        return Err(AppError::BadRequest(
            "invalid routing config: expected object at $".into(),
        ));
    };
    for key in root.keys() {
        if !matches!(key.as_str(), "routes" | "compaction") {
            return Err(unknown_field("$", key));
        }
    }
    let Some(routes) = root.get("routes") else {
        return Ok(());
    };
    let Some(routes) = routes.as_array() else {
        return Err(AppError::BadRequest(
            "invalid routing config: expected array at $.routes".into(),
        ));
    };
    for (config_index, route) in routes.iter().enumerate() {
        let path = format!("$.routes[{config_index}]");
        let Some(route) = route.as_object() else {
            continue;
        };
        for key in route.keys() {
            if !matches!(
                key.as_str(),
                "source" | "target" | "enabled" | "remote_compaction_policy" | "compaction"
            ) {
                return Err(unknown_field(&path, key));
            }
        }
        validate_endpoint_fields(
            route.get("source"),
            &format!("{path}.source"),
            &["model", "format"],
        )?;
        validate_endpoint_fields(
            route.get("target"),
            &format!("{path}.target"),
            &["provider", "model", "format"],
        )?;
    }
    Ok(())
}

fn validate_endpoint_fields(value: Option<&Value>, path: &str, allowed: &[&str]) -> AppResult<()> {
    let Some(value) = value else {
        return Ok(());
    };
    let Some(object) = value.as_object() else {
        return Ok(());
    };
    for key in object.keys() {
        if !allowed.contains(&key.as_str()) {
            return Err(unknown_field(path, key));
        }
    }
    Ok(())
}

fn unknown_field(path: &str, key: &str) -> AppError {
    AppError::BadRequest(format!(
        "invalid routing config: unknown field {path}.{key}"
    ))
}

fn project_draft_routes(value: &Value) -> DraftProjectionV2 {
    let routes = value
        .get("routes")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let mut draft_routes = Vec::new();
    let mut diagnostics = Vec::new();
    let mut omitted_rows = false;

    for (config_index, route) in routes.iter().enumerate() {
        match project_draft_route(config_index, route) {
            DraftRouteOutcome::Route(route) => {
                draft_routes.push(route);
            }
            DraftRouteOutcome::Omitted { diagnostic } => {
                omitted_rows = true;
                diagnostics.push(diagnostic);
            }
        }
    }

    let route_ids = draft_routes
        .iter()
        .map(|route| (route.row_id.clone(), route.row_id.clone()))
        .collect::<BTreeMap<_, _>>();
    for route in &draft_routes {
        diagnostics.extend(validate_draft_route(route));
    }
    for diagnostic in &mut diagnostics {
        if diagnostic.route_id.is_none() {
            if let Some(row_id) = diagnostic.row_id.as_ref() {
                diagnostic.route_id = route_ids.get(row_id).cloned();
            }
        }
    }
    let valid_routes = draft_routes
        .iter()
        .filter(|route| {
            !diagnostics
                .iter()
                .any(|diagnostic| diagnostic.row_id.as_deref() == Some(route.row_id.as_str()))
        })
        .map(|route| ConfiguredRoute {
            source: ConfiguredSource {
                model: route.source_model.clone(),
                format: (route.source_runtime_format != WILDCARD_FORMAT)
                    .then(|| route.source_runtime_format.clone()),
            },
            target: ConfiguredTarget {
                provider: route.target_provider,
                model: route.target_model.clone(),
                format: route.target_provider_format.clone(),
            },
            enabled: route.enabled,
            remote_compaction_policy: None,
            compaction: None,
        })
        .collect();

    DraftProjectionV2 {
        config: RoutingConfigFile {
            routes: valid_routes,
            compaction: None,
        },
        routes: draft_routes,
        diagnostics,
        omitted_rows,
    }
}

enum DraftRouteOutcome {
    Route(DraftRouteV2),
    Omitted { diagnostic: DiagnosticV2 },
}

fn project_draft_route(config_index: usize, route: &Value) -> DraftRouteOutcome {
    let row_id = row_id(config_index);
    let Some(route_object) = route.as_object() else {
        return omitted_row(
            config_index,
            "invalid_route_row",
            "Config route must be an object.",
            format!("$.routes[{config_index}]"),
            "route",
        );
    };
    let Some(source) = route_object.get("source").and_then(Value::as_object) else {
        return omitted_row(
            config_index,
            "invalid_route_source",
            "Config route source must be an object.",
            format!("$.routes[{config_index}].source"),
            "source",
        );
    };
    let Some(target) = route_object.get("target").and_then(Value::as_object) else {
        return omitted_row(
            config_index,
            "invalid_route_target",
            "Config route target must be an object.",
            format!("$.routes[{config_index}].target"),
            "target",
        );
    };
    let Some(source_model) = source.get("model").and_then(Value::as_str) else {
        return omitted_row(
            config_index,
            "invalid_source_model",
            "Config route source.model must be a string.",
            format!("$.routes[{config_index}].source.model"),
            "source.model",
        );
    };
    let Some(target_model) = target.get("model").and_then(Value::as_str) else {
        return omitted_row(
            config_index,
            "invalid_target_model",
            "Config route target.model must be a string.",
            format!("$.routes[{config_index}].target.model"),
            "target.model",
        );
    };
    let source_runtime_format = source
        .get("format")
        .and_then(Value::as_str)
        .unwrap_or(WILDCARD_FORMAT)
        .to_string();
    let target_provider = target
        .get("provider")
        .and_then(Value::as_str)
        .and_then(parse_provider)
        .unwrap_or(Provider::Unsupported);
    let target_provider_format = target
        .get("format")
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
        .or_else(|| {
            target_provider
                .default_target_format()
                .map(|format| format.as_str().to_string())
        });
    let enabled = route_object
        .get("enabled")
        .and_then(Value::as_bool)
        .unwrap_or(true);

    DraftRouteOutcome::Route(DraftRouteV2 {
        config_index,
        row_id,
        enabled,
        source_model: source_model.to_string(),
        source_runtime_format,
        target_provider,
        target_model: target_model.to_string(),
        target_provider_format,
        raw_config_route: route.clone(),
    })
}

fn omitted_row(
    config_index: usize,
    code: &'static str,
    message: &'static str,
    path: String,
    field: &'static str,
) -> DraftRouteOutcome {
    let row_id = row_id(config_index);
    DraftRouteOutcome::Omitted {
        diagnostic: diagnostic_v2(
            format!("diag:{row_id}:{code}"),
            DiagnosticSeverity::Error,
            code,
            message,
            Some(path),
            Some(config_index),
            Some(row_id.clone()),
            Some(row_id),
            Some(field.to_string()),
            Some("Fix this row before saving.".to_string()),
        ),
    }
}

fn validate_draft_route(route: &DraftRouteV2) -> Vec<DiagnosticV2> {
    let mut diagnostics = Vec::new();
    let route_path = format!("$.routes[{}]", route.config_index);

    if route.source_runtime_format != WILDCARD_FORMAT
        && parse_source_format(&route.source_runtime_format).is_none()
    {
        diagnostics.push(diagnostic_v2(
            format!("diag:{}:unsupported_source_format", route.row_id),
            DiagnosticSeverity::Error,
            "unsupported_source_format",
            format!(
                "Config route {} uses unsupported source format {}.",
                route.row_id, route.source_runtime_format
            ),
            Some(format!("{route_path}.source.format")),
            Some(route.config_index),
            Some(route.row_id.clone()),
            Some(route.row_id.clone()),
            Some("source.format".to_string()),
            Some("Use responses, chat_completions, anthropic_messages, google_generate_content, or openai_images.".to_string()),
        ));
    }

    if route.target_provider == Provider::Unsupported {
        diagnostics.push(diagnostic_v2(
            format!("diag:{}:unsupported_target_provider", route.row_id),
            DiagnosticSeverity::Error,
            "unsupported_target_provider",
            format!(
                "Config route {} uses unsupported target provider.",
                route.row_id
            ),
            Some(format!("{route_path}.target.provider")),
            Some(route.config_index),
            Some(route.row_id.clone()),
            Some(route.row_id.clone()),
            Some("target.provider".to_string()),
            Some("Choose codex, bedrock, or google.".to_string()),
        ));
    }

    match route.target_provider_format.as_deref() {
        Some(format) if parse_target_format(format).is_none() => {
            diagnostics.push(diagnostic_v2(
                format!("diag:{}:unsupported_target_format", route.row_id),
                DiagnosticSeverity::Error,
                "unsupported_target_format",
                format!(
                    "Config route {} uses unsupported target format {format}.",
                    route.row_id
                ),
                Some(format!("{route_path}.target.format")),
                Some(route.config_index),
                Some(route.row_id.clone()),
                Some(route.row_id.clone()),
                Some("target.format".to_string()),
                Some(
                    "Use responses, anthropic_messages, google_generate_content, or openai_images."
                        .to_string(),
                ),
            ));
        }
        None => {
            diagnostics.push(diagnostic_v2(
                format!("diag:{}:missing_target_format", route.row_id),
                DiagnosticSeverity::Error,
                "missing_target_format",
                format!(
                    "Config route {} cannot infer a target format.",
                    route.row_id
                ),
                Some(format!("{route_path}.target.format")),
                Some(route.config_index),
                Some(route.row_id.clone()),
                Some(route.row_id.clone()),
                Some("target.format".to_string()),
                Some(
                    "Choose a supported target provider or add an explicit target format."
                        .to_string(),
                ),
            ));
        }
        _ => {}
    }

    if route.raw_config_route.get("enabled").is_some()
        && route
            .raw_config_route
            .get("enabled")
            .and_then(Value::as_bool)
            .is_none()
    {
        diagnostics.push(diagnostic_v2(
            format!("diag:{}:invalid_enabled", route.row_id),
            DiagnosticSeverity::Error,
            "invalid_enabled",
            format!("Config route {} enabled must be a boolean.", route.row_id),
            Some(format!("{route_path}.enabled")),
            Some(route.config_index),
            Some(route.row_id.clone()),
            Some(route.row_id.clone()),
            Some("enabled".to_string()),
            Some("Use true, false, or omit enabled.".to_string()),
        ));
    }

    diagnostics
}

fn parse_provider(provider: &str) -> Option<Provider> {
    match provider {
        "bedrock" => Some(Provider::Bedrock),
        "codex" => Some(Provider::Codex),
        "cursor" => Some(Provider::Cursor),
        "google" => Some(Provider::Google),
        "unsupported" => Some(Provider::Unsupported),
        _ => None,
    }
}

fn build_route_cards(
    effective_routes: &[EffectiveRouteProjection],
    config_routes: &[ConfigRouteProjection],
    draft_routes: &[DraftRouteV2],
    diagnostics: &[DiagnosticV2],
) -> Vec<RouteCardV2> {
    let mut cards = Vec::new();
    for route in effective_routes
        .iter()
        .filter(|route| route.origin == GraphRouteOrigin::Catalog)
    {
        let mut card = RouteCardV2 {
            id: route.route_id.clone(),
            origin: route.origin,
            state: RouteCardState::Effective,
            group_ids: Vec::new(),
            order: cards.len(),
            mutable: route.mutable,
            enabled: true,
            source: RouteCardSourceV2 {
                model: route.source_model.clone(),
                source_provider: route.source_provider,
                runtime_format: route.source_runtime_format.to_string(),
            },
            target: RouteCardTargetV2 {
                provider: route.target_provider,
                model: route.target_model.clone(),
                provider_format: Some(route.target_provider_format.clone()),
            },
            summary: route_summary(
                &route.source_model,
                route.source_runtime_format,
                route.target_provider,
                &route.target_model,
            ),
            diagnostic_ids: Vec::new(),
            config_index: None,
            row_id: None,
            precedence_label: Some("Catalog fallback".to_string()),
            raw_config_route: None,
        };
        card.group_ids = group_ids_for_card(&card);
        cards.push(card);
    }

    let effective_row_ids = effective_routes
        .iter()
        .filter_map(|route| route.row_id.as_deref())
        .collect::<BTreeSet<_>>();
    for route in draft_routes {
        let config_state = config_routes
            .iter()
            .find(|config_route| config_route.row_id == route.row_id)
            .map(|config_route| config_route.state);
        let mut card = RouteCardV2 {
            id: route.row_id.clone(),
            origin: GraphRouteOrigin::HotConfig,
            state: draft_route_card_state(route, config_state, &effective_row_ids, diagnostics),
            group_ids: Vec::new(),
            order: cards.len(),
            mutable: true,
            enabled: route.enabled,
            source: RouteCardSourceV2 {
                model: route.source_model.clone(),
                source_provider: source_provider_for_model(&route.source_model),
                runtime_format: route.source_runtime_format.clone(),
            },
            target: RouteCardTargetV2 {
                provider: route.target_provider,
                model: route.target_model.clone(),
                provider_format: route.target_provider_format.clone(),
            },
            summary: route_summary(
                &route.source_model,
                &route.source_runtime_format,
                route.target_provider,
                &route.target_model,
            ),
            diagnostic_ids: diagnostic_ids_for_route(diagnostics, &route.row_id),
            config_index: Some(route.config_index),
            row_id: Some(route.row_id.clone()),
            precedence_label: Some(format!("Hot config #{}", route.config_index + 1)),
            raw_config_route: Some(route.raw_config_route.clone()),
        };
        card.group_ids = group_ids_for_card(&card);
        cards.push(card);
    }

    cards
}

fn draft_route_card_state(
    route: &DraftRouteV2,
    config_state: Option<ConfigRouteState>,
    effective_row_ids: &BTreeSet<&str>,
    diagnostics: &[DiagnosticV2],
) -> RouteCardState {
    if diagnostics.iter().any(|diagnostic| {
        diagnostic.row_id.as_deref() == Some(route.row_id.as_str())
            && diagnostic.severity == DiagnosticSeverity::Error
    }) {
        return RouteCardState::Invalid;
    }
    match config_state {
        Some(ConfigRouteState::Disabled) => RouteCardState::Disabled,
        Some(ConfigRouteState::Shadowed) => RouteCardState::Shadowed,
        Some(ConfigRouteState::Inactive) => RouteCardState::Inactive,
        Some(ConfigRouteState::Active) if effective_row_ids.contains(route.row_id.as_str()) => {
            RouteCardState::Effective
        }
        Some(ConfigRouteState::Active) => RouteCardState::Effective,
        None => RouteCardState::Invalid,
    }
}

fn diagnostic_ids_for_route(diagnostics: &[DiagnosticV2], route_id: &str) -> Vec<String> {
    diagnostics
        .iter()
        .filter(|diagnostic| {
            diagnostic.route_id.as_deref() == Some(route_id)
                || diagnostic.row_id.as_deref() == Some(route_id)
        })
        .map(|diagnostic| diagnostic.id.clone())
        .collect()
}

fn group_ids_for_card(card: &RouteCardV2) -> Vec<String> {
    vec![
        format!(
            "source_provider:{}",
            source_provider_slug(card.source.source_provider)
        ),
        format!("runtime:{}", card.source.runtime_format),
        format!("state:{}", route_card_state_slug(card.state)),
    ]
}

fn build_groups(route_cards: &[RouteCardV2], diagnostics: &[DiagnosticV2]) -> Vec<GroupV2> {
    let diagnostic_severities = diagnostics
        .iter()
        .map(|diagnostic| (diagnostic.id.as_str(), diagnostic.severity))
        .collect::<BTreeMap<_, _>>();
    let mut groups = BTreeMap::<String, Vec<String>>::new();
    for card in route_cards {
        for group_id in &card.group_ids {
            groups
                .entry(group_id.clone())
                .or_default()
                .push(card.id.clone());
        }
    }

    groups
        .into_iter()
        .enumerate()
        .map(|(order, (id, route_ids))| {
            let cards = route_ids
                .iter()
                .filter_map(|route_id| route_cards.iter().find(|card| &card.id == route_id))
                .collect::<Vec<_>>();
            let mut counts = GroupCountsV2 {
                total: cards.len(),
                effective: cards
                    .iter()
                    .filter(|card| card.state == RouteCardState::Effective)
                    .count(),
                configured: cards
                    .iter()
                    .filter(|card| card.origin == GraphRouteOrigin::HotConfig)
                    .count(),
                errors: 0,
                warnings: 0,
            };
            for card in &cards {
                for diagnostic_id in &card.diagnostic_ids {
                    match diagnostic_severities.get(diagnostic_id.as_str()) {
                        Some(DiagnosticSeverity::Error) => counts.errors += 1,
                        Some(DiagnosticSeverity::Warning) => counts.warnings += 1,
                        _ => {}
                    }
                }
            }
            let (kind, label, provider, source_provider, runtime_format) = group_metadata(&id);
            GroupV2 {
                id,
                label,
                kind,
                order,
                route_ids,
                collapsed_default: false,
                counts,
                provider,
                source_provider,
                runtime_format,
            }
        })
        .collect()
}

fn group_metadata(
    id: &str,
) -> (
    GroupKindV2,
    String,
    Option<Provider>,
    Option<SourceProvider>,
    Option<String>,
) {
    if let Some(source_provider) = id.strip_prefix("source_provider:") {
        let source_provider =
            parse_source_provider(source_provider).unwrap_or(SourceProvider::Custom);
        return (
            GroupKindV2::SourceFamily,
            format!(
                "Source provider: {source_provider}",
                source_provider = source_provider_slug(source_provider)
            ),
            None,
            Some(source_provider),
            None,
        );
    }
    if let Some(provider) = id.strip_prefix("provider:") {
        let provider = parse_provider(provider).unwrap_or(Provider::Unsupported);
        return (
            GroupKindV2::Provider,
            format!("Provider: {provider}", provider = provider_slug(provider)),
            Some(provider),
            None,
            None,
        );
    }
    if let Some(runtime_format) = id.strip_prefix("runtime:") {
        return (
            GroupKindV2::RuntimeFormat,
            format!("Runtime: {runtime_format}"),
            None,
            None,
            Some(runtime_format.to_string()),
        );
    }
    let state = id.strip_prefix("state:").unwrap_or(id);
    (
        GroupKindV2::State,
        format!("State: {state}"),
        None,
        None,
        None,
    )
}

fn convert_diagnostics_v2(diagnostics: &[ConfigDiagnostic]) -> Vec<DiagnosticV2> {
    diagnostics
        .iter()
        .enumerate()
        .map(|(index, diagnostic)| {
            let route_id = diagnostic
                .row_id
                .clone()
                .or_else(|| diagnostic.config_index.map(row_id));
            diagnostic_v2(
                format!("diag:v1:{index}:{}", diagnostic.code),
                diagnostic.severity,
                diagnostic.code,
                diagnostic.message.clone(),
                diagnostic
                    .config_index
                    .map(|index| format!("$.routes[{index}]")),
                diagnostic.config_index,
                diagnostic.row_id.clone(),
                route_id,
                None,
                None,
            )
        })
        .collect()
}

fn has_projected_safe_row(draft: &DraftProjectionV2) -> bool {
    draft.routes.iter().any(|route| {
        !draft
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.row_id.as_deref() == Some(route.row_id.as_str()))
    })
}

fn draft_status(
    omitted_rows: bool,
    has_projected_safe_row: bool,
    diagnostics: &[DiagnosticV2],
) -> DraftStatus {
    let has_blocking_error = diagnostics
        .iter()
        .any(|diagnostic| diagnostic.severity == DiagnosticSeverity::Error || diagnostic.blocking);
    if omitted_rows || (has_projected_safe_row && has_blocking_error) {
        return DraftStatus::PartiallyProjected;
    }
    if has_blocking_error {
        DraftStatus::Invalid
    } else {
        DraftStatus::Valid
    }
}

#[allow(clippy::too_many_arguments)]
fn diagnostic_v2(
    id: String,
    severity: DiagnosticSeverity,
    code: &'static str,
    message: impl Into<String>,
    path: Option<String>,
    config_index: Option<usize>,
    row_id: Option<String>,
    route_id: Option<String>,
    field: Option<String>,
    suggestion: Option<String>,
) -> DiagnosticV2 {
    DiagnosticV2 {
        id,
        severity,
        code,
        message: message.into(),
        blocking: severity == DiagnosticSeverity::Error,
        path,
        config_index,
        row_id,
        route_id,
        field,
        suggestion,
    }
}

fn route_summary(
    source_model: &str,
    runtime_format: &str,
    target_provider: Provider,
    target_model: &str,
) -> String {
    format!(
        "{source_model} ({runtime_format}) → {}:{target_model}",
        provider_slug(target_provider)
    )
}

fn provider_slug(provider: Provider) -> &'static str {
    match provider {
        Provider::Bedrock => "bedrock",
        Provider::Codex => "codex",
        Provider::Cursor => "cursor",
        Provider::Google => "google",
        Provider::Unsupported => "unsupported",
    }
}

fn source_provider_for_model(model: &str) -> SourceProvider {
    let lower = model.to_ascii_lowercase();
    if matches!(
        lower.as_str(),
        "composer-2" | "composer-2-fast" | "composer-1.5"
    ) || lower.starts_with("composer-")
    {
        SourceProvider::Cursor
    } else if lower.starts_with("openai/")
        || lower.starts_with("openai:")
        || lower.starts_with("gpt-")
        || openai_o_series_source_model(&lower)
    {
        SourceProvider::Openai
    } else if lower.starts_with("anthropic/")
        || lower.starts_with("claude-")
        || lower.starts_with("anthropic.")
    {
        SourceProvider::Anthropic
    } else if lower.starts_with("google/")
        || lower.starts_with("gemini-")
        || lower.starts_with("models/gemini-")
    {
        SourceProvider::Google
    } else {
        SourceProvider::Custom
    }
}

fn openai_o_series_source_model(model: &str) -> bool {
    ["o1", "o3", "o4"]
        .into_iter()
        .any(|prefix| model == prefix || model.starts_with(&format!("{prefix}-")))
}

fn parse_source_provider(source_provider: &str) -> Option<SourceProvider> {
    match source_provider {
        "cursor" => Some(SourceProvider::Cursor),
        "openai" => Some(SourceProvider::Openai),
        "anthropic" => Some(SourceProvider::Anthropic),
        "google" => Some(SourceProvider::Google),
        "custom" => Some(SourceProvider::Custom),
        _ => None,
    }
}

fn source_provider_slug(source_provider: SourceProvider) -> &'static str {
    match source_provider {
        SourceProvider::Cursor => "cursor",
        SourceProvider::Openai => "openai",
        SourceProvider::Anthropic => "anthropic",
        SourceProvider::Google => "google",
        SourceProvider::Custom => "custom",
    }
}

fn route_card_state_slug(state: RouteCardState) -> &'static str {
    match state {
        RouteCardState::Effective => "effective",
        RouteCardState::Disabled => "disabled",
        RouteCardState::Shadowed => "shadowed",
        RouteCardState::Inactive => "inactive",
        RouteCardState::Invalid => "invalid",
    }
}

fn winning_hot_route(
    config: &RoutingConfigFile,
    source_model: &str,
    runtime_format: &'static str,
    diagnostics: &mut Vec<ConfigDiagnostic>,
) -> AppResult<Option<WinningHotRoute>> {
    let mut winner: Option<WinningHotRoute> = None;

    for (config_index, route) in config.routes.iter().enumerate() {
        if route.source.model != source_model
            || !route.enabled
            || !source_format_matches(route.source.format.as_deref(), Some(runtime_format))
        {
            continue;
        }

        if !is_active_source_format(route.source.format.as_deref().unwrap_or(WILDCARD_FORMAT)) {
            continue;
        }

        let current = WinningHotRoute {
            config_index,
            row_id: row_id(config_index),
            target_provider: route.target.provider,
            target_model: route.target.model.clone(),
            target_provider_format: target_provider_format(
                route.target.provider,
                route.target.format.as_deref(),
            )?,
        };

        if winner.is_none() {
            winner = Some(current);
            continue;
        }

        diagnostics.push(ConfigDiagnostic {
            severity: DiagnosticSeverity::Warning,
            code: "hot_route_shadowed",
            message: format!(
                "Config route {} is shadowed for {source_model} {runtime_format}",
                current.row_id
            ),
            config_index: Some(config_index),
            row_id: Some(current.row_id),
            source_model: Some(source_model.to_string()),
            runtime_format: Some(runtime_format.to_string()),
            target_provider: Some(current.target_provider),
            target_model: Some(current.target_model),
        });
    }

    Ok(winner)
}

fn catalog_effective_route(
    source_model: &str,
    runtime_format: &'static str,
) -> Option<EffectiveRouteProjection> {
    let model = KNOWN_MODELS.iter().find(|model| model.id == source_model)?;
    let target_format = model.provider.default_target_format()?;
    if !catalog_route_supports_runtime(model.provider, target_format, runtime_format) {
        return None;
    }
    Some(EffectiveRouteProjection {
        route_id: format!("catalog:{}:{runtime_format}", model.id),
        origin: GraphRouteOrigin::Catalog,
        mutable: false,
        source_model: model.id.to_string(),
        source_provider: source_provider_for_model(model.id),
        source_runtime_format: runtime_format,
        target_provider: model.provider,
        target_model: model.upstream_model.to_string(),
        target_provider_format: target_format.as_str().to_string(),
        config_index: None,
        row_id: None,
    })
}

fn catalog_route_supports_runtime(
    provider: Provider,
    target_format: TargetFormat,
    runtime_format: &str,
) -> bool {
    matches!(
        (runtime_format, provider, target_format),
        ("responses", Provider::Codex, TargetFormat::Responses)
            | (
                "responses",
                Provider::Bedrock,
                TargetFormat::AnthropicMessages
            )
            | (
                "responses",
                Provider::Google,
                TargetFormat::GoogleGenerateContent
            )
            | (
                "anthropic_messages",
                Provider::Codex,
                TargetFormat::Responses
            )
            | (
                "anthropic_messages",
                Provider::Bedrock,
                TargetFormat::AnthropicMessages
            )
            | ("chat_completions", Provider::Codex, TargetFormat::Responses)
            | (
                "chat_completions",
                Provider::Bedrock,
                TargetFormat::AnthropicMessages
            )
            | ("responses", Provider::Cursor, TargetFormat::CursorAgent)
            | (
                "chat_completions",
                Provider::Cursor,
                TargetFormat::CursorAgent
            )
            | (
                "anthropic_messages",
                Provider::Cursor,
                TargetFormat::CursorAgent
            )
    )
}

fn mark_shadowed_config_routes(
    config_routes: &mut [ConfigRouteProjection],
    diagnostics: &[ConfigDiagnostic],
) {
    for diagnostic in diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.code == "hot_route_shadowed")
    {
        if let Some(config_index) = diagnostic.config_index {
            if let Some(route) = config_routes.get_mut(config_index) {
                route.state = ConfigRouteState::Shadowed;
            }
        }
    }
}

fn build_sources(
    source_models: &BTreeSet<String>,
    config_routes: &[ConfigRouteProjection],
) -> Vec<GraphSource> {
    source_models
        .iter()
        .map(|model| {
            let origin = if config_routes
                .iter()
                .any(|route| route.source_model == *model && route.enabled)
            {
                GraphRouteOrigin::HotConfig
            } else {
                GraphRouteOrigin::Catalog
            };
            GraphSource {
                model: model.clone(),
                origin,
                source_provider: source_provider_for_model(model),
            }
        })
        .collect()
}

fn build_nodes_and_edges(
    effective_routes: &[EffectiveRouteProjection],
) -> (Vec<GraphNode>, Vec<GraphEdge>) {
    let mut node_keys = BTreeSet::new();
    let mut nodes = Vec::new();
    let mut edges = Vec::new();

    for route in effective_routes {
        let source_id = source_node_id(&route.source_model, route.source_runtime_format);
        if node_keys.insert(source_id.clone()) {
            nodes.push(GraphNode {
                id: source_id.clone(),
                label: format!("{} ({})", route.source_model, route.source_runtime_format),
                kind: GraphNodeKind::SourceModel,
                provider: None,
                source_provider: Some(route.source_provider),
                mutable: route.mutable,
            });
        }

        let target_id = target_node_id(route.target_provider, &route.target_model);
        if node_keys.insert(target_id.clone()) {
            nodes.push(GraphNode {
                id: target_id.clone(),
                label: route.target_model.clone(),
                kind: GraphNodeKind::TargetModel,
                provider: Some(route.target_provider),
                source_provider: None,
                mutable: route.mutable,
            });
        }

        edges.push(GraphEdge {
            id: format!("edge:{}", route.route_id),
            source: source_id,
            target: target_id,
            route_id: route.route_id.clone(),
            source_runtime_format: route.source_runtime_format,
            source_provider: route.source_provider,
            origin: route.origin,
        });
    }

    (nodes, edges)
}

fn runtime_formats() -> Vec<RuntimeFormat> {
    RUNTIME_FORMATS
        .into_iter()
        .map(|format| RuntimeFormat {
            format,
            runtime_active: true,
        })
        .chain(
            ACCEPTED_INACTIVE_FORMATS
                .into_iter()
                .map(|format| RuntimeFormat {
                    format,
                    runtime_active: false,
                }),
        )
        .collect()
}

fn target_provider_format(provider: Provider, format: Option<&str>) -> AppResult<String> {
    if let Some(format) = format {
        parse_target_format(format)
            .map(|target_format| target_format.as_str().to_string())
            .ok_or_else(|| {
                AppError::BadRequest(format!(
                    "invalid routing config: unsupported target format {format}"
                ))
            })
    } else {
        provider
            .default_target_format()
            .map(|target_format| target_format.as_str().to_string())
            .ok_or_else(|| {
                AppError::BadRequest("invalid routing config: unsupported target provider".into())
            })
    }
}

fn is_active_source_format(format: &str) -> bool {
    format == WILDCARD_FORMAT || RUNTIME_FORMATS.contains(&format)
}

fn row_id(config_index: usize) -> String {
    format!("config:{config_index}")
}

fn diagnostic(
    severity: DiagnosticSeverity,
    code: &'static str,
    message: impl Into<String>,
    config_index: Option<usize>,
    row_id: Option<String>,
    source_model: Option<String>,
) -> ConfigDiagnostic {
    ConfigDiagnostic {
        severity,
        code,
        message: message.into(),
        config_index,
        row_id,
        source_model,
        runtime_format: None,
        target_provider: None,
        target_model: None,
    }
}

fn source_node_id(source_model: &str, runtime_format: &str) -> String {
    format!("source:{source_model}:{runtime_format}")
}

fn target_node_id(provider: Provider, target_model: &str) -> String {
    format!("target:{provider:?}:{target_model}")
}

fn generated_at() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs().to_string())
        .unwrap_or_else(|_| "0".to_string())
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn graph(value: Value) -> ConfigGraph {
        build_config_graph(value).unwrap()
    }

    fn effective<'a>(
        graph: &'a ConfigGraph,
        source_model: &str,
        runtime_format: &str,
    ) -> &'a EffectiveRouteProjection {
        graph
            .effective_routes
            .iter()
            .find(|route| {
                route.source_model == source_model && route.source_runtime_format == runtime_format
            })
            .unwrap()
    }

    #[test]
    fn hot_config_graph_empty_config_projects_catalog_routes() {
        let graph = graph(json!({ "routes": [] }));

        assert_eq!(graph.schema_version, 2);
        assert_eq!(graph.contract_version, "config_graph.v2");
        assert_eq!(graph.draft_status, DraftStatus::Valid);
        assert!(!graph.groups.is_empty());
        assert!(graph.focal.is_none());
        assert!(!graph.route_cards.is_empty());
        assert!(graph
            .route_cards
            .iter()
            .any(|route| route.origin == GraphRouteOrigin::Catalog));
        assert!(graph.diagnostics_v2.iter().all(|diagnostic| {
            diagnostic.blocking == (diagnostic.severity == DiagnosticSeverity::Error)
        }));
        assert!(graph
            .effective_routes
            .iter()
            .any(|route| route.origin == GraphRouteOrigin::Catalog));
        assert!(graph
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "no_hot_overrides"));
    }

    #[test]
    fn hot_config_graph_simple_hot_row_becomes_effective() {
        let graph = graph(json!({
            "routes": [{
                "source": { "model": "custom-model", "format": "responses" },
                "target": { "provider": "codex", "model": "gpt-5.5", "format": "responses" }
            }]
        }));

        assert_eq!(graph.config_routes[0].row_id, "config:0");
        let route = effective(&graph, "custom-model", "responses");
        assert_eq!(route.origin, GraphRouteOrigin::HotConfig);
        assert_eq!(route.row_id.as_deref(), Some("config:0"));
        assert_eq!(route.target_provider_format, "responses");
        let card = graph
            .route_cards
            .iter()
            .find(|card| card.id == "config:0")
            .unwrap();
        assert_eq!(card.state, RouteCardState::Effective);
        assert_eq!(card.source.source_provider, SourceProvider::Custom);
        assert!(card
            .group_ids
            .iter()
            .any(|group| group == "source_provider:custom"));
        assert!(card
            .group_ids
            .iter()
            .any(|group| group == "runtime:responses"));
        assert!(card
            .group_ids
            .iter()
            .any(|group| group == "state:effective"));
        assert!(graph.groups.iter().any(|group| {
            group.id == "source_provider:custom"
                && group.counts.total > 0
                && group
                    .route_ids
                    .iter()
                    .any(|route_id| route_id == "config:0")
        }));
    }

    #[test]
    fn hot_config_graph_classifies_composer_source_provider_as_cursor() {
        let graph = graph(json!({
            "routes": [{
                "source": { "model": "composer-2-fast", "format": "responses" },
                "target": { "provider": "codex", "model": "composer-2-fast", "format": "responses" }
            }]
        }));

        let config_route = &graph.config_routes[0];
        assert_eq!(config_route.source_provider, SourceProvider::Cursor);
        assert_eq!(config_route.target_provider, Provider::Codex);
        assert_eq!(config_route.target_model, "composer-2-fast");

        let route = effective(&graph, "composer-2-fast", "responses");
        assert_eq!(route.source_provider, SourceProvider::Cursor);
        assert_eq!(route.target_provider, Provider::Codex);
        assert_eq!(route.target_model, "composer-2-fast");

        let source = graph
            .sources
            .iter()
            .find(|source| source.model == "composer-2-fast")
            .unwrap();
        assert_eq!(source.source_provider, SourceProvider::Cursor);

        let card = graph
            .route_cards
            .iter()
            .find(|card| card.id == "config:0")
            .unwrap();
        assert_eq!(card.source.source_provider, SourceProvider::Cursor);
        assert!(card
            .group_ids
            .iter()
            .any(|group| group == "source_provider:cursor"));
    }

    #[test]
    fn hot_config_graph_wildcard_before_exact_shadows_exact_for_that_runtime() {
        let graph = graph(json!({
            "routes": [
                {
                    "source": { "model": "same-model" },
                    "target": { "provider": "codex", "model": "gpt-5.5" }
                },
                {
                    "source": { "model": "same-model", "format": "responses" },
                    "target": { "provider": "google", "model": "gemini-3.1-flash-lite" }
                }
            ]
        }));

        let responses = effective(&graph, "same-model", "responses");
        assert_eq!(responses.row_id.as_deref(), Some("config:0"));
        assert_eq!(graph.config_routes[1].state, ConfigRouteState::Shadowed);
        assert!(graph
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.row_id.as_deref() == Some("config:1")));
    }

    #[test]
    fn hot_config_graph_exact_before_wildcard_wins_only_its_runtime() {
        let graph = graph(json!({
            "routes": [
                {
                    "source": { "model": "same-model", "format": "responses" },
                    "target": { "provider": "google", "model": "gemini-3.1-flash-lite" }
                },
                {
                    "source": { "model": "same-model" },
                    "target": { "provider": "codex", "model": "gpt-5.5" }
                }
            ]
        }));

        assert_eq!(
            effective(&graph, "same-model", "responses")
                .row_id
                .as_deref(),
            Some("config:0")
        );
        assert_eq!(
            effective(&graph, "same-model", "chat_completions")
                .row_id
                .as_deref(),
            Some("config:1")
        );
        assert_eq!(graph.config_routes[1].state, ConfigRouteState::Shadowed);
    }

    #[test]
    fn hot_config_graph_disabled_and_inactive_rows_never_win() {
        let graph = graph(json!({
            "routes": [
                {
                    "enabled": false,
                    "source": { "model": "same-model", "format": "responses" },
                    "target": { "provider": "google", "model": "gemini-3.1-flash-lite" }
                },
                {
                    "source": { "model": "same-model", "format": "openai_images" },
                    "target": { "provider": "codex", "model": "gpt-5.5", "format": "responses" }
                },
                {
                    "source": { "model": "same-model", "format": "responses" },
                    "target": { "provider": "codex", "model": "gpt-5.5" }
                }
            ]
        }));

        assert_eq!(graph.config_routes[0].state, ConfigRouteState::Disabled);
        assert_eq!(graph.config_routes[1].state, ConfigRouteState::Inactive);
        assert_eq!(
            effective(&graph, "same-model", "responses")
                .row_id
                .as_deref(),
            Some("config:2")
        );
        assert!(!graph
            .effective_routes
            .iter()
            .any(|route| route.source_runtime_format == "openai_images"));
    }

    #[test]
    fn hot_config_graph_rejects_unknown_fields_and_secret_keys() {
        let unknown = build_config_graph(json!({
            "routes": [],
            "extra": true
        }))
        .unwrap_err()
        .to_string();
        assert!(unknown.contains("unknown field"));

        let secret = build_config_graph(json!({
            "routes": [{
                "source": { "model": "x" },
                "target": { "provider": "codex", "model": "gpt-5.5" },
                "api-key": "do-not-echo"
            }]
        }))
        .unwrap_err()
        .to_string();
        assert!(secret.contains("$.routes[0].api-key"));
        assert!(!secret.contains("do-not-echo"));
    }

    #[test]
    fn hot_config_graph_unsupported_target_provider_is_invalid_draft() {
        let graph = build_config_graph(json!({
            "routes": [{
                "source": { "model": "x", "format": "responses" },
                "target": { "provider": "unsupported", "model": "gpt-image-2" }
            }]
        }))
        .unwrap();

        assert_eq!(graph.draft_status, DraftStatus::Invalid);
        let card = graph
            .route_cards
            .iter()
            .find(|card| card.id == "config:0")
            .unwrap();
        assert_eq!(card.state, RouteCardState::Invalid);
        assert_eq!(card.target.provider, Provider::Unsupported);
        assert_eq!(card.target.provider_format, None);
        assert!(graph.diagnostics_v2.iter().any(|diagnostic| {
            diagnostic.severity == DiagnosticSeverity::Error
                && diagnostic.blocking
                && diagnostic.path.as_deref() == Some("$.routes[0].target.provider")
                && diagnostic.route_id.as_deref() == Some("config:0")
        }));
    }

    #[test]
    fn hot_config_graph_cursor_target_provider_is_valid() {
        let graph = build_config_graph(json!({
            "routes": [{
                "source": { "model": "composer-2", "format": "responses" },
                "target": { "provider": "cursor", "model": "composer-2" }
            }]
        }))
        .unwrap();

        assert_eq!(graph.draft_status, DraftStatus::Valid);
        let card = graph
            .route_cards
            .iter()
            .find(|card| card.id == "config:0")
            .unwrap();
        assert_eq!(card.source.source_provider, SourceProvider::Cursor);
        assert_eq!(card.target.provider, Provider::Cursor);
        assert_eq!(card.target.provider_format.as_deref(), Some("cursor_agent"));
        assert!(!graph.diagnostics_v2.iter().any(|diagnostic| {
            diagnostic.code == "unsupported_target_provider"
                && diagnostic.path.as_deref() == Some("$.routes[0].target.provider")
        }));
    }

    #[test]
    fn hot_config_graph_omitted_unprojectable_row_is_partially_projected() {
        let graph = build_config_graph(json!({
            "routes": [
                {
                    "source": { "model": "safe-model", "format": "responses" },
                    "target": { "provider": "codex", "model": "gpt-5.5" }
                },
                {
                    "source": "not-an-object",
                    "target": { "provider": "codex", "model": "gpt-5.5" }
                }
            ]
        }))
        .unwrap();

        assert_eq!(graph.draft_status, DraftStatus::PartiallyProjected);
        assert!(graph.route_cards.iter().any(|card| card.id == "config:0"));
        assert!(!graph.route_cards.iter().any(|card| card.id == "config:1"));
        assert!(graph.diagnostics_v2.iter().any(|diagnostic| {
            diagnostic.severity == DiagnosticSeverity::Error
                && diagnostic.blocking
                && diagnostic.path.as_deref() == Some("$.routes[1].source")
                && diagnostic.row_id.as_deref() == Some("config:1")
        }));
    }
}
