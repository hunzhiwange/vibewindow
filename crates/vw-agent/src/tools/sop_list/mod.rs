//! SOP 列表工具
//!
//! 列出所有已加载的 SOP 及其触发器、优先级、步骤数和活动运行。

use std::fmt::Write;
use std::sync::Mutex;

use async_trait::async_trait;
use serde_json::json;

use super::traits::{Tool, ToolResult};
use crate::app::agent::sop::SopEngine;

/// Lists all loaded SOPs with their triggers, priority, step count, and active runs.
pub struct SopListTool {
    engine: std::sync::Arc<Mutex<SopEngine>>,
}

impl SopListTool {
    pub fn new(engine: std::sync::Arc<Mutex<SopEngine>>) -> Self {
        Self { engine }
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl Tool for SopListTool {
    fn name(&self) -> &str {
        "sop_list"
    }

    fn description(&self) -> &str {
        "列出所有已加载的标准操作程序（SOP），包括触发器、优先级、步骤数和活动运行数。可选按名称或优先级筛选。"
    }

    fn parameters_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "filter": {
                    "type": "string",
                    "description": "按名称子字符串或优先级（low/normal/high/critical）筛选 SOP"
                }
            }
        })
    }

    async fn execute(&self, args: serde_json::Value) -> anyhow::Result<ToolResult> {
        let filter = args.get("filter").and_then(|v| v.as_str()).unwrap_or("");
        let filter_lower = filter.to_lowercase();

        let engine =
            self.engine.lock().map_err(|e| anyhow::anyhow!("Engine lock poisoned: {e}"))?;
        let sops = engine.sops();

        if sops.is_empty() {
            return Ok(ToolResult { success: true, output: "No SOPs loaded.".into(), error: None });
        }

        let filtered: Vec<_> = if filter_lower.is_empty() {
            sops.iter().collect()
        } else {
            sops.iter()
                .filter(|s| {
                    s.name.to_lowercase().contains(&filter_lower)
                        || s.priority.to_string() == filter_lower
                })
                .collect()
        };

        if filtered.is_empty() {
            return Ok(ToolResult {
                success: true,
                output: format!("No SOPs match filter '{filter}'."),
                error: None,
            });
        }

        let active_runs = engine.active_runs();
        let mut output =
            format!("Loaded SOPs ({} total, {} shown):\n\n", sops.len(), filtered.len());

        for sop in &filtered {
            let active_count = active_runs.values().filter(|r| r.sop_name == sop.name).count();
            let triggers: Vec<String> = sop.triggers.iter().map(|t| t.to_string()).collect();

            let _ = writeln!(
                output,
                "- **{}** [{}] — {} steps, {} trigger(s): {}{}",
                sop.name,
                sop.priority,
                sop.steps.len(),
                sop.triggers.len(),
                triggers.join(", "),
                if active_count > 0 {
                    format!(" (active runs: {active_count})")
                } else {
                    String::new()
                }
            );
        }

        Ok(ToolResult { success: true, output, error: None })
    }
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
