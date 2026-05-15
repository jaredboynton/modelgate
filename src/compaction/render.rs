use serde_json::{json, Value};

use crate::{
    compaction::DecodedUmpPack,
    model_alias::{ResolvedTarget, TargetFormat},
};

pub fn render_ump_pack_for_target(pack: &DecodedUmpPack, target: &ResolvedTarget) -> Vec<Value> {
    let text = restored_context_text(pack);
    match target.target_format {
        TargetFormat::Responses
        | TargetFormat::AnthropicMessages
        | TargetFormat::GoogleGenerateContent => {
            vec![json!({
                "type": "message",
                "role": "system",
                "content": [{ "type": "input_text", "text": text }]
            })]
        }
        TargetFormat::OpenaiImages => Vec::new(),
    }
}

fn restored_context_text(pack: &DecodedUmpPack) -> String {
    let mut lines = Vec::new();
    lines.push("Restored UMP compaction context.".to_string());
    if pack.visible.context_degraded {
        lines
            .push("context_degraded: some prior visible context was summarized or omitted.".into());
    }
    if !pack.visible.durable_constraints.is_empty() {
        lines.push("Durable constraints:".into());
        lines.extend(
            pack.visible
                .durable_constraints
                .iter()
                .map(|constraint| format!("- {constraint}")),
        );
    }
    if let Some(task_objective) = &pack.visible.task_objective {
        lines.push(format!("Task objective: {task_objective}"));
    }
    if let Some(summary) = &pack.visible.summary {
        lines.push(format!("Visible summary: {summary}"));
    }
    lines.join("\n")
}
