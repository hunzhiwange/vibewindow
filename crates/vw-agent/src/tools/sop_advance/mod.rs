//! SOP 步骤推进工具
//!
//! 手动推进 SOP 步骤执行，用于需要外部输入的步骤。

use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use serde_json::json;
use tracing::warn;

use super::traits::{Tool, ToolResult};
use crate::app::agent::sop::types::{SopRunAction, SopStepResult, SopStepStatus};
use crate::app::agent::sop::{SopAuditLogger, SopEngine, SopMetricsCollector};

/// Report a step result and advance an SOP run to the next step.
pub struct SopAdvanceTool {
    engine: Arc<Mutex<SopEngine>>,
    audit: Option<Arc<SopAuditLogger>>,
    collector: Option<Arc<SopMetricsCollector>>,
}

impl SopAdvanceTool {
    pub fn new(engine: Arc<Mutex<SopEngine>>) -> Self {
        Self { engine, audit: None, collector: None }
    }

    pub fn with_audit(mut self, audit: Arc<SopAuditLogger>) -> Self {
        self.audit = Some(audit);
        self
    }

    pub fn with_collector(mut self, collector: Arc<SopMetricsCollector>) -> Self {
        self.collector = Some(collector);
        self
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl Tool for SopAdvanceTool {
    fn name(&self) -> &str {
        "sop_advance"
    }

    fn description(&self) -> &str {
        "报告当前 SOP 步骤的结果并前进到下一步。提供 run_id、步骤是否成功以及简要输出摘要。"
    }

    fn parameters_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "run_id": {
                    "type": "string",
                    "description": "要前进的运行 ID"
                },
                "status": {
                    "type": "string",
                    "enum": ["completed", "failed", "skipped"],
                    "description": "当前步骤的结果状态"
                },
                "output": {
                    "type": "string",
                    "description": "此步骤发生内容的简要摘要"
                }
            },
            "required": ["run_id", "status", "output"]
        })
    }

    async fn execute(&self, args: serde_json::Value) -> anyhow::Result<ToolResult> {
        let run_id = args
            .get("run_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'run_id' parameter"))?;

        let status_str = args
            .get("status")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'status' parameter"))?;

        let output = args
            .get("output")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'output' parameter"))?;

        let step_status = match status_str {
            "completed" => SopStepStatus::Completed,
            "failed" => SopStepStatus::Failed,
            "skipped" => SopStepStatus::Skipped,
            other => {
                return Ok(ToolResult {
                    success: false,
                    output: String::new(),
                    error: Some(format!(
                        "Invalid status '{other}'. Must be: completed, failed, or skipped"
                    )),
                });
            }
        };

        // Lock engine, advance step, snapshot data for audit, then drop lock
        let (action, step_result_ok, finished_run) = {
            let mut engine =
                self.engine.lock().map_err(|e| anyhow::anyhow!("Engine lock poisoned: {e}"))?;

            let current_step = engine
                .get_run(run_id)
                .map(|r| r.current_step)
                .ok_or_else(|| anyhow::anyhow!("Run not found: {run_id}"))?;

            let now = now_iso8601();
            let step_result = SopStepResult {
                step_number: current_step,
                status: step_status,
                output: output.to_string(),
                started_at: now.clone(),
                completed_at: Some(now),
            };
            let step_result_clone = step_result.clone();

            match engine.advance_step(run_id, step_result) {
                Ok(action) => {
                    // Snapshot finished run for audit (Completed/Failed/Cancelled)
                    let finished = match &action {
                        SopRunAction::Completed { run_id, .. }
                        | SopRunAction::Failed { run_id, .. } => engine.get_run(run_id).cloned(),
                        _ => None,
                    };
                    // Only audit step result when advance succeeded
                    (Ok(action), Some(step_result_clone), finished)
                }
                Err(e) => (Err(e), None, None),
            }
        };

        // Audit logging (engine lock dropped, safe to await)
        if let Some(ref audit) = self.audit {
            if let Some(ref sr) = step_result_ok
                && let Err(e) = audit.log_step_result(run_id, sr).await
            {
                warn!("SOP audit log_step_result failed: {e}");
            }
            if let Some(ref run) = finished_run
                && let Err(e) = audit.log_run_complete(run).await
            {
                warn!("SOP audit log_run_complete failed: {e}");
            }
        }

        // Metrics collector (independent of audit)
        if let Some(ref collector) = self.collector && let Some(ref run) = finished_run {
            collector.record_run_complete(run);
        }

        match action {
            Ok(action) => {
                let result_output = match action {
                    SopRunAction::ExecuteStep { run_id, context, .. } => {
                        format!("Step recorded. Next step for run {run_id}:\n\n{context}")
                    }
                    SopRunAction::WaitApproval { run_id, context, .. } => {
                        format!(
                            "Step recorded. Next step for run {run_id} (waiting for approval):\n\n{context}"
                        )
                    }
                    SopRunAction::Completed { run_id, sop_name } => {
                        format!("SOP '{sop_name}' run {run_id} completed successfully.")
                    }
                    SopRunAction::Failed { run_id, sop_name, reason } => {
                        format!("SOP '{sop_name}' run {run_id} failed: {reason}")
                    }
                };
                Ok(ToolResult { success: true, output: result_output, error: None })
            }
            Err(e) => Ok(ToolResult {
                success: false,
                output: String::new(),
                error: Some(format!("Failed to advance step: {e}")),
            }),
        }
    }
}

use crate::app::agent::sop::engine::now_iso8601;

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
