use serde::{Deserialize, Serialize};

use crate::model_alias::{Provider, ResolvedTarget, TargetFormat};

#[derive(Debug, Clone, Copy, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RemoteCompactionPolicy {
    Native,
    ProxyVisibleSummary,
    Local,
    Off,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct CompactionPolicy {
    pub remote: RemoteCompactionPolicy,
}

impl CompactionPolicy {
    pub fn for_target(target: &ResolvedTarget, configured: Option<RemoteCompactionPolicy>) -> Self {
        Self {
            remote: configured.unwrap_or_else(|| default_policy_for_target(target)),
        }
    }
}

pub fn default_policy_for_target(target: &ResolvedTarget) -> RemoteCompactionPolicy {
    match (target.provider, target.target_format) {
        (Provider::Codex, TargetFormat::Responses) => RemoteCompactionPolicy::Native,
        (Provider::Bedrock | Provider::Google, _) => RemoteCompactionPolicy::Local,
        (Provider::Cursor, _) => RemoteCompactionPolicy::Local,
        (Provider::Unsupported, _) => RemoteCompactionPolicy::Off,
        _ => RemoteCompactionPolicy::Local,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serde_uses_contract_values() {
        let policy: RemoteCompactionPolicy =
            serde_json::from_str("\"proxy_visible_summary\"").unwrap();
        assert_eq!(policy, RemoteCompactionPolicy::ProxyVisibleSummary);
        assert_eq!(
            serde_json::to_string(&policy).unwrap(),
            "\"proxy_visible_summary\""
        );
    }
}
