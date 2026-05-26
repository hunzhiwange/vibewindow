//! SOP 执行工具
//!
//! 手动触发指定名称的 SOP 执行。返回运行 ID 和第一步指令。

use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use serde_json::json;
use tracing::warn;

use super::traits::{Tool, ToolResult};
use crate::app::agent::sop::types::{SopEvent, SopRunAction, SopTriggerSource};
use crate::app::agent::sop::{SopAuditLogger, SopEngine};

/// SOP 执行工具
///
/// 用于手动触发指定名称的标准操作程序（SOP）执行。
/// 该工具会返回运行 ID 和第一步的执行指令。
///
/// # 字段说明
///
/// * `engine` - SOP 引擎的共享引用，使用互斥锁保护以确保线程安全
/// * `audit` - 可选的审计日志记录器，用于记录 SOP 执行过程
pub struct SopExecuteTool {
    engine: Arc<Mutex<SopEngine>>,
    audit: Option<Arc<SopAuditLogger>>,
}

impl SopExecuteTool {
    /// 创建新的 SOP 执行工具实例
    ///
    /// # 参数
    ///
    /// * `engine` - SOP 引擎的共享引用，包装在 `Arc<Mutex<>>` 中以支持并发访问
    ///
    /// # 返回值
    ///
    /// 返回一个新的 `SopExecuteTool` 实例，初始时不包含审计日志记录器
    ///
    /// # 示例
    ///
    /// ```ignore
    /// let engine = Arc::new(Mutex::new(SopEngine::new()));
    /// let tool = SopExecuteTool::new(engine);
    /// ```
    pub fn new(engine: Arc<Mutex<SopEngine>>) -> Self {
        Self { engine, audit: None }
    }

    /// 为工具添加审计日志记录器
    ///
    /// 使用构建器模式添加可选的审计日志记录器。
    ///
    /// # 参数
    ///
    /// * `audit` - 审计日志记录器的共享引用
    ///
    /// # 返回值
    ///
    /// 返回配置了审计日志记录器的 `SopExecuteTool` 实例
    ///
    /// # 示例
    ///
    /// ```ignore
    /// let audit = Arc::new(SopAuditLogger::new());
    /// let tool = SopExecuteTool::new(engine).with_audit(audit);
    /// ```
    pub fn with_audit(mut self, audit: Arc<SopAuditLogger>) -> Self {
        self.audit = Some(audit);
        self
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl Tool for SopExecuteTool {
    /// 返回工具名称
    ///
    /// 工具名称为 "sop_execute"，用于在工具注册表中标识此工具
    fn name(&self) -> &str {
        "sop_execute"
    }

    /// 返回工具描述
    ///
    /// 提供工具的中文描述，说明其用途和如何使用
    fn description(&self) -> &str {
        "手动按名称触发标准操作程序（SOP）。返回运行 ID 和第一步指令。使用 sop_list 查看可用的 SOP。"
    }

    /// 返回工具参数的 JSON Schema
    ///
    /// 定义工具接受的参数结构：
    /// - `name`（必需）：要执行的 SOP 名称
    /// - `payload`（可选）：触发载荷，以 JSON 字符串形式传递
    fn parameters_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "name": {
                    "type": "string",
                    "description": "要执行的 SOP 名称"
                },
                "payload": {
                    "type": "string",
                    "description": "可选的触发载荷（JSON 字符串）"
                }
            },
            "required": ["name"]
        })
    }

    /// 执行 SOP 触发操作
    ///
    /// 根据提供的参数手动触发指定的 SOP 执行流程。
    ///
    /// # 参数
    ///
    /// * `args` - JSON 格式的参数对象，包含：
    ///   - `name`: SOP 名称（必需）
    ///   - `payload`: 可选的触发载荷（JSON 字符串）
    ///
    /// # 返回值
    ///
    /// 返回 `ToolResult`，包含：
    /// - 成功时：运行 ID 和第一步指令
    /// - 失败时：错误信息
    ///
    /// # 错误处理
    ///
    /// - 缺少 `name` 参数时返回错误
    /// - 引擎锁获取失败时返回错误
    /// - SOP 启动失败时返回错误
    async fn execute(&self, args: serde_json::Value) -> anyhow::Result<ToolResult> {
        // 从参数中提取 SOP 名称（必需参数）
        let sop_name = args
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing 'name' parameter"))?;

        // 从参数中提取载荷（可选参数）
        let payload = args.get("payload").and_then(|v| v.as_str()).map(String::from);

        // 构建 SOP 事件对象
        let event = SopEvent {
            source: SopTriggerSource::Manual, // 标记为手动触发
            topic: None,                      // 手动触发无主题
            payload,                          // 可选载荷
            timestamp: now_iso8601(),         // 当前时间戳
        };

        // 锁定引擎，启动运行，快照运行状态用于审计，然后释放锁
        // 使用作用域确保锁在审计日志记录前释放，避免死锁
        let (action, run_snapshot) = {
            let mut engine =
                self.engine.lock().map_err(|e| anyhow::anyhow!("Engine lock poisoned: {e}"))?;

            match engine.start_run(sop_name, event) {
                Ok(action) => {
                    // 从返回的动作中提取运行 ID
                    let run_id = action_run_id(&action);
                    // 根据运行 ID 获取运行状态的快照
                    let snapshot = run_id.and_then(|id| engine.get_run(id).cloned());
                    (Ok(action), snapshot)
                }
                Err(e) => (Err(e), None),
            }
        };

        // 记录审计日志（引擎锁已释放，可以安全地 await）
        if let Some(ref audit) = self.audit
            && let Some(ref run) = run_snapshot
            && let Err(e) = audit.log_run_start(run).await
        {
            warn!("SOP audit log_run_start failed: {e}");
        }

        // 处理执行结果并生成输出
        match action {
            Ok(action) => {
                // 根据不同的动作类型生成相应的输出消息
                let output = match action {
                    SopRunAction::ExecuteStep { run_id, context, .. } => {
                        // SOP 运行已启动，返回运行 ID 和上下文
                        format!("SOP run started: {run_id}\n\n{context}")
                    }
                    SopRunAction::WaitApproval { run_id, context, .. } => {
                        // SOP 运行已启动，但等待审批
                        format!("SOP run started: {run_id} (waiting for approval)\n\n{context}")
                    }
                    SopRunAction::Completed { run_id, sop_name } => {
                        // SOP 立即完成（无步骤）
                        format!("SOP '{sop_name}' run {run_id} completed immediately (no steps).")
                    }
                    SopRunAction::Failed { run_id, reason, .. } => {
                        // SOP 运行失败
                        format!("SOP run {run_id} failed: {reason}")
                    }
                };
                Ok(ToolResult { success: true, output, error: None })
            }
            Err(e) => Ok(ToolResult {
                success: false,
                output: String::new(),
                error: Some(format!("Failed to start SOP: {e}")),
            }),
        }
    }
}

/// 从 SopRunAction 枚举变体中提取运行 ID
///
/// 该函数匹配所有可能的 `SopRunAction` 变体，并返回其中包含的运行 ID。
/// 由于所有变体都包含 `run_id` 字段，该函数总是能返回 `Some(&str)`。
///
/// # 参数
///
/// * `action` - SOP 运行动作的引用
///
/// # 返回值
///
/// 返回运行 ID 的字符串切片引用，包装在 `Option` 中
///
/// # 示例
///
/// ```ignore
/// let action = SopRunAction::ExecuteStep {
///     run_id: "run-123".to_string(),
///     context: "Step 1".to_string(),
///     step_id: 1,
/// };
/// let run_id = action_run_id(&action); // Some("run-123")
/// ```
fn action_run_id(action: &SopRunAction) -> Option<&str> {
    match action {
        SopRunAction::ExecuteStep { run_id, .. }
        | SopRunAction::WaitApproval { run_id, .. }
        | SopRunAction::Completed { run_id, .. }
        | SopRunAction::Failed { run_id, .. } => Some(run_id),
    }
}

use crate::app::agent::sop::engine::now_iso8601;

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
