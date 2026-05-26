//! SOP 批准工具
//!
//! 批准等待操作员确认的 SOP 步骤。

use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use serde_json::json;
use tracing::warn;

use super::traits::{Tool, ToolResult};
use crate::app::agent::sop::types::SopRunAction;
use crate::app::agent::sop::{SopAuditLogger, SopEngine, SopMetricsCollector};

/// SOP 批准工具
///
/// 用于批准等待操作员确认的 SOP（标准操作流程）步骤。
/// 该工具实现了 `Tool` trait，可以作为代理工具被调用。
///
/// # 字段说明
///
/// * `engine` - SOP 引擎的共享引用，用于执行批准操作
/// * `audit` - 可选的审计日志记录器，用于记录批准操作的审计日志
/// * `collector` - 可选的指标收集器，用于记录批准相关的性能指标
pub struct SopApproveTool {
    engine: Arc<Mutex<SopEngine>>,
    audit: Option<Arc<SopAuditLogger>>,
    collector: Option<Arc<SopMetricsCollector>>,
}

impl SopApproveTool {
    /// 创建一个新的 SOP 批准工具实例
    ///
    /// # 参数
    ///
    /// * `engine` - SOP 引擎的共享引用，必须是线程安全的
    ///
    /// # 返回值
    ///
    /// 返回一个新创建的 `SopApproveTool` 实例，默认不启用审计日志和指标收集
    pub fn new(engine: Arc<Mutex<SopEngine>>) -> Self {
        Self { engine, audit: None, collector: None }
    }

    /// 配置审计日志记录器
    ///
    /// 使用构建器模式为工具添加审计日志功能。
    /// 审计日志会在每次批准操作后记录相关信息。
    ///
    /// # 参数
    ///
    /// * `audit` - 审计日志记录器的共享引用
    ///
    /// # 返回值
    ///
    /// 返回配置了审计日志的 `Self`，便于链式调用
    pub fn with_audit(mut self, audit: Arc<SopAuditLogger>) -> Self {
        self.audit = Some(audit);
        self
    }

    /// 配置指标收集器
    ///
    /// 使用构建器模式为工具添加指标收集功能。
    /// 指标收集器会记录批准操作的相关性能指标。
    ///
    /// # 参数
    ///
    /// * `collector` - 指标收集器的共享引用
    ///
    /// # 返回值
    ///
    /// 返回配置了指标收集器的 `Self`，便于链式调用
    pub fn with_collector(mut self, collector: Arc<SopMetricsCollector>) -> Self {
        self.collector = Some(collector);
        self
    }
}

/// Tool trait 实现
///
/// 为 `SopApproveTool` 实现 `Tool` trait，使其能够作为代理工具被调用。
/// 支持 WASM 和原生平台，通过条件编译自动选择正确的 `async_trait` 配置。
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl Tool for SopApproveTool {
    /// 返回工具名称
    ///
    /// 工具名称为 "sop_approve"，用于在工具注册表中标识该工具。
    fn name(&self) -> &str {
        "sop_approve"
    }

    /// 返回工具描述
    ///
    /// 提供工具功能的中文描述，说明该工具用于批准等待操作员确认的 SOP 步骤。
    fn description(&self) -> &str {
        "批准等待操作员批准的待处理 SOP 步骤。返回要执行的步骤指令。使用 sop_status 查看哪些运行在等待。"
    }

    /// 返回工具参数的 JSON Schema
    ///
    /// 定义了工具所需的参数结构：
    /// - `run_id`（必需）：要批准的 SOP 运行 ID
    fn parameters_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "run_id": {
                    "type": "string",
                    "description": "要批准的运行 ID"
                }
            },
            "required": ["run_id"]
        })
    }

    /// 执行批准操作
    ///
    /// 批准指定的 SOP 运行步骤，并返回执行结果。
    ///
    /// # 参数
    ///
    /// * `args` - JSON 格式的参数，必须包含 `run_id` 字段
    ///
    /// # 返回值
    ///
    /// 返回 `ToolResult`，包含以下信息：
    /// - 成功时：`success` 为 `true`，`output` 包含批准后的执行上下文
    /// - 失败时：`success` 为 `false`，`error` 包含错误信息
    ///
    /// # 执行流程
    ///
    /// 1. 从参数中提取 `run_id`
    /// 2. 获取引擎锁，执行批准操作并快照运行状态
    /// 3. 释放锁后，异步记录审计日志（如果配置）
    /// 4. 记录指标（如果配置）
    /// 5. 返回执行结果
    ///
    /// # 错误处理
    ///
    /// - 参数缺失 `run_id` 时返回错误
    /// - 引擎锁获取失败时返回错误
    /// - 批准操作失败时在 `ToolResult` 中设置错误信息
    async fn execute(&self, args: serde_json::Value) -> anyhow::Result<ToolResult> {
        let run_id = args
            .get("run_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'run_id' parameter"))?;

        // 获取引擎锁，执行批准操作，并快照运行状态用于后续审计
        // 注意：必须在释放锁之前完成快照，以便后续异步操作使用
        let (result, run_snapshot) = {
            let mut engine =
                self.engine.lock().map_err(|e| anyhow::anyhow!("Engine lock poisoned: {e}"))?;

            match engine.approve_step(run_id) {
                Ok(action) => {
                    // 批准成功，保存运行快照用于审计和指标记录
                    let snapshot = engine.get_run(run_id).cloned();
                    (Ok(action), snapshot)
                }
                Err(e) => (Err(e), None),
            }
        };

        // 异步记录审计日志（引擎锁已释放，可以安全等待）
        // 即使审计日志记录失败，也不会影响批准操作的结果
        if let Some(ref audit) = self.audit {
            if let Some(ref run) = run_snapshot
                && let Err(e) = audit.log_approval(run, run.current_step).await
            {
                warn!("SOP audit log after approve failed: {e}");
            }
        }

        // 记录批准操作的指标（与审计日志独立，互不影响）
        if let Some(ref collector) = self.collector && let Some(ref run) = run_snapshot {
            collector.record_approval(&run.sop_name, &run.run_id);
        }

        // 根据批准结果构建返回值
        match result {
            Ok(action) => {
                let output = match action {
                    // 如果批准后需要执行步骤，返回包含上下文的详细信息
                    SopRunAction::ExecuteStep { run_id, context, .. } => {
                        format!("Approved. Proceeding with run {run_id}.\n\n{context}")
                    }
                    // 其他类型的操作，返回操作的调试信息
                    other => format!("Approved. Action: {other:?}"),
                };
                Ok(ToolResult { success: true, output, error: None })
            }
            Err(e) => Ok(ToolResult {
                success: false,
                output: String::new(),
                error: Some(format!("Approval failed: {e}")),
            }),
        }
    }
}

/// 单元测试模块
///
/// 测试代码位于 `tests/sop_approve.rs` 文件中。
#[cfg(test)]
#[path = "tests.rs"]
mod tests;
