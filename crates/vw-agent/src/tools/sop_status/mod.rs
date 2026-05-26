//! SOP 状态查询工具
//!
//! 本模块提供 SOP（标准作业流程）执行状态查询功能，用于检查：
//! - 当前活动的 SOP 运行实例
//! - 已完成的 SOP 运行历史
//! - 特定运行 ID 的详细状态信息
//!
//! # 核心组件
//!
//! - [`SopStatusTool`]: 状态查询工具主结构，实现 [`Tool`] trait
//!
//! # 查询模式
//!
//! 1. **按运行 ID 查询**: 提供具体 `run_id`，返回该运行的完整状态
//! 2. **按 SOP 名称查询**: 提供 `sop_name`，返回该 SOP 的所有运行
//! 3. **全局查询**: 无参数时返回所有活动运行概览

use std::fmt::Write;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use serde_json::json;

use super::traits::{Tool, ToolResult};
use crate::app::agent::sop::{SopEngine, SopMetricsCollector};

/// SOP 状态查询工具
///
/// 用于查询 SOP 执行状态，支持多种查询模式：
/// - 按 `run_id` 查询特定运行的详细状态
/// - 按 `sop_name` 列出该 SOP 的所有运行
/// - 无参数时显示所有活动运行
///
/// # 字段说明
///
/// - `engine`: SOP 执行引擎的共享引用，用于访问运行状态数据
/// - `collector`: 可选的指标收集器，用于获取聚合的 SOP 执行指标
///
/// # 示例
///
/// ```ignore
/// use std::sync::{Arc, Mutex};
/// use crate::app::agent::sop::SopEngine;
/// use crate::app::agent::tools::sop_status::SopStatusTool;
///
/// let engine = Arc::new(Mutex::new(SopEngine::new()));
/// let tool = SopStatusTool::new(engine);
/// ```
pub struct SopStatusTool {
    /// SOP 执行引擎（线程安全共享引用）
    engine: Arc<Mutex<SopEngine>>,
    /// 可选的指标收集器，用于聚合指标查询
    collector: Option<Arc<SopMetricsCollector>>,
}

impl SopStatusTool {
    /// 创建新的 SOP 状态查询工具实例
    ///
    /// # 参数
    ///
    /// - `engine`: SOP 执行引擎的线程安全共享引用
    ///
    /// # 返回值
    ///
    /// 返回初始化后的 `SopStatusTool` 实例（不含指标收集器）
    ///
    /// # 示例
    ///
    /// ```ignore
    /// let engine = Arc::new(Mutex::new(SopEngine::new()));
    /// let tool = SopStatusTool::new(engine);
    /// ```
    pub fn new(engine: Arc<Mutex<SopEngine>>) -> Self {
        Self { engine, collector: None }
    }

    /// 配置指标收集器（构建器模式）
    ///
    /// 为工具添加指标收集器，启用 `include_metrics` 参数时可以返回聚合指标。
    ///
    /// # 参数
    ///
    /// - `collector`: 指标收集器的共享引用
    ///
    /// # 返回值
    ///
    /// 返回配置了指标收集器的工具实例
    ///
    /// # 示例
    ///
    /// ```ignore
    /// let collector = Arc::new(SopMetricsCollector::new());
    /// let tool = SopStatusTool::new(engine).with_collector(collector);
    /// ```
    pub fn with_collector(mut self, collector: Arc<SopMetricsCollector>) -> Self {
        self.collector = Some(collector);
        self
    }

    /// 追加门控状态信息到输出
    ///
    /// 当请求包含门控状态时，追加相关信息到输出字符串。
    /// 当前版本门控功能未启用，输出提示信息。
    ///
    /// # 参数
    ///
    /// - `output`: 输出字符串的可变引用
    /// - `include_gate_status`: 是否包含门控状态的标志
    fn append_gate_status(&self, output: &mut String, include_gate_status: bool) {
        if include_gate_status {
            // 门控功能当前未启用，输出提示信息
            let _ = writeln!(
                output,
                "\nGate Status: not available (ampersona-gates feature not enabled)"
            );
        }
    }
}

/// Tool trait 实现（支持 WASM 和原生平台）
///
/// 为 `SopStatusTool` 实现 [`Tool`] trait，使其可作为 Agent 工具使用。
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl Tool for SopStatusTool {
    /// 返回工具名称标识
    ///
    /// # 返回值
    ///
    /// 返回固定字符串 `"sop_status"`
    fn name(&self) -> &str {
        "sop_status"
    }

    /// 返回工具功能描述
    ///
    /// # 返回值
    ///
    /// 返回工具的使用说明，描述支持的查询模式
    fn description(&self) -> &str {
        "查询 SOP 执行状态。提供 run_id 查询特定运行，或提供 sop_name 列出该 SOP 的运行。无参数时显示所有活动运行。"
    }

    /// 返回工具参数的 JSON Schema
    ///
    /// # 返回值
    ///
    /// 返回描述参数结构的 JSON Schema，包含：
    /// - `run_id`: 要查询的特定运行 ID（可选）
    /// - `sop_name`: 要列出运行的 SOP 名称（可选）
    /// - `include_metrics`: 是否包含聚合指标（可选）
    /// - `include_gate_status`: 是否包含门控状态（可选）
    fn parameters_schema(&self) -> serde_json::Value {
        json!({
            "type": "object",
            "properties": {
                "run_id": {
                    "type": "string",
                    "description": "要查询的特定运行 ID"
                },
                "sop_name": {
                    "type": "string",
                    "description": "要列出运行的 SOP 名称"
                },
                "include_metrics": {
                    "type": "boolean",
                    "description": "包含聚合的 SOP 指标（完成率、偏差率、干预次数、窗口化变体）"
                },
                "include_gate_status": {
                    "type": "boolean",
                    "description": "包含信任阶段和门控评估状态"
                }
            }
        })
    }

    /// 执行状态查询
    ///
    /// 根据参数执行不同的查询模式：
    /// 1. **按运行 ID 查询**: 返回该运行的完整状态和步骤结果
    /// 2. **按 SOP 名称查询**: 返回该 SOP 的活动和已完成运行
    /// 3. **全局查询**: 返回所有活动运行概览
    ///
    /// # 参数
    ///
    /// - `args`: JSON 格式的查询参数，支持的字段：
    ///   - `run_id` (string, 可选): 特定运行 ID
    ///   - `sop_name` (string, 可选): SOP 名称过滤
    ///   - `include_metrics` (bool, 可选): 是否包含聚合指标，默认 false
    ///   - `include_gate_status` (bool, 可选): 是否包含门控状态，默认 false
    ///
    /// # 返回值
    ///
    /// 返回 `anyhow::Result<ToolResult>`，其中：
    /// - `success`: 始终为 true（查询失败返回友好提示而非错误）
    /// - `output`: 格式化的状态信息字符串
    /// - `error`: 始终为 None
    ///
    /// # 错误处理
    ///
    /// 仅当引擎锁被污染时返回错误，其他情况返回友好的状态提示。
    async fn execute(&self, args: serde_json::Value) -> anyhow::Result<ToolResult> {
        // 解析查询参数
        let run_id = args.get("run_id").and_then(|v| v.as_str());
        let sop_name = args.get("sop_name").and_then(|v| v.as_str());
        let include_metrics =
            args.get("include_metrics").and_then(|v| v.as_bool()).unwrap_or(false);
        let include_gate_status =
            args.get("include_gate_status").and_then(|v| v.as_bool()).unwrap_or(false);

        // 获取引擎锁，处理锁污染错误
        let engine =
            self.engine.lock().map_err(|e| anyhow::anyhow!("Engine lock poisoned: {e}"))?;

        // 模式1: 查询特定运行 ID
        if let Some(run_id) = run_id {
            return match engine.get_run(run_id) {
                Some(run) => {
                    // 构建运行详情输出
                    let mut output = format!(
                        "Run: {}\nSOP: {}\nStatus: {}\nStep: {} of {}\nStarted: {}\n",
                        run.run_id,
                        run.sop_name,
                        run.status,
                        run.current_step,
                        run.total_steps,
                        run.started_at,
                    );
                    // 追加完成时间（如果存在）
                    if let Some(ref completed) = run.completed_at {
                        let _ = writeln!(output, "Completed: {completed}");
                    }
                    // 追加步骤执行结果
                    if !run.step_results.is_empty() {
                        let _ = writeln!(output, "\nStep results:");
                        for step in &run.step_results {
                            let _ = writeln!(
                                output,
                                "  Step {}: {} — {}",
                                step.step_number, step.status, step.output
                            );
                        }
                    }
                    // 追加门控状态（如果请求）
                    self.append_gate_status(&mut output, include_gate_status);
                    Ok(ToolResult { success: true, output, error: None })
                }
                None => Ok(ToolResult {
                    success: true,
                    output: format!("No run found with ID '{run_id}'."),
                    error: None,
                }),
            };
        }

        // 模式2/3: 列出特定 SOP 的运行或所有活动运行
        let mut output = String::new();

        // 查询活动运行，按 SOP 名称过滤（如果有）
        let active: Vec<_> = engine
            .active_runs()
            .values()
            .filter(|r| sop_name.map_or(true, |name| r.sop_name == name))
            .collect();

        // 输出活动运行列表
        if active.is_empty() {
            let scope = sop_name.map_or(String::new(), |n| format!(" for '{n}'"));
            let _ = writeln!(output, "No active runs{scope}.");
        } else {
            let _ = writeln!(output, "Active runs ({}):", active.len());
            for run in &active {
                let _ = writeln!(
                    output,
                    "  {} — {} [{}] step {}/{}",
                    run.run_id, run.sop_name, run.status, run.current_step, run.total_steps
                );
            }
        }

        // 查询并输出已完成运行（最多显示10条最近的）
        let finished = engine.finished_runs(sop_name);
        if !finished.is_empty() {
            let _ = writeln!(output, "\nFinished runs ({}):", finished.len());
            for run in finished.iter().rev().take(10) {
                let _ = writeln!(
                    output,
                    "  {} — {} [{}] ({})",
                    run.run_id,
                    run.sop_name,
                    run.status,
                    run.completed_at.as_deref().unwrap_or("?")
                );
            }
        }

        // 追加聚合指标（如果请求且收集器可用）
        if include_metrics {
            if let Some(ref collector) = self.collector {
                // 构建指标键前缀：全局或特定 SOP
                let prefix = sop_name.map_or("sop".to_string(), |n| format!("sop.{n}"));
                let _ = writeln!(output, "\nMetrics ({prefix}):");
                // 遍历所有指标后缀，查询并输出指标值
                for suffix in METRIC_SUFFIXES {
                    let key = format!("{prefix}.{suffix}");
                    if let Some(val) = collector.get_metric_value(&key) {
                        let _ = writeln!(output, "  {suffix}: {}", format_metric_value(&val));
                    }
                }
            } else {
                // 收集器未配置时的提示
                let _ = writeln!(output, "\nMetrics: not available (collector not configured)");
            }
        }

        // 追加门控状态（如果请求）
        self.append_gate_status(&mut output, include_gate_status);

        Ok(ToolResult { success: true, output, error: None })
    }
}

/// 状态输出中渲染的指标后缀列表
///
/// 定义在 `include_metrics` 参数为 true 时查询的指标键后缀。
/// 这些后缀与 SOP 名称前缀组合形成完整的指标键。
///
/// # 指标分类
///
/// - **计数类**: `runs_completed`, `runs_failed`, `runs_cancelled`
/// - **比率类**: `completion_rate`, `deviation_rate`, `protocol_adherence_rate`
/// - **干预类**: `human_intervention_count`, `human_intervention_rate`
/// - **超时类**: `timeout_auto_approvals`, `timeout_approval_rate`
/// - **窗口化**: `completion_rate_7d`, `deviation_rate_7d`, `completion_rate_30d`, `deviation_rate_30d`
const METRIC_SUFFIXES: &[&str] = &[
    "runs_completed",
    "runs_failed",
    "runs_cancelled",
    "completion_rate",
    "deviation_rate",
    "protocol_adherence_rate",
    "human_intervention_count",
    "human_intervention_rate",
    "timeout_auto_approvals",
    "timeout_approval_rate",
    "completion_rate_7d",
    "deviation_rate_7d",
    "completion_rate_30d",
    "deviation_rate_30d",
];

/// 格式化指标值为可读字符串
///
/// 根据指标值的类型进行格式化：
/// - 整数: 直接显示
/// - 浮点数: 无小数时显示整数，否则保留4位小数
/// - 其他类型: 使用默认字符串表示
///
/// # 参数
///
/// - `val`: JSON 格式的指标值
///
/// # 返回值
///
/// 返回格式化后的字符串表示
///
/// # 示例
///
/// ```ignore
/// use serde_json::json;
///
/// assert_eq!(format_metric_value(&json!(42)), "42");
/// assert_eq!(format_metric_value(&json!(3.14159)), "3.1416");
/// assert_eq!(format_metric_value(&json!(4.0)), "4");
/// ```
fn format_metric_value(val: &serde_json::Value) -> String {
    match val {
        serde_json::Value::Number(n) => {
            // 尝试解析为无符号整数
            if let Some(u) = n.as_u64() {
                format!("{u}")
            } else if let Some(f) = n.as_f64() {
                // 浮点数：无小数部分时显示整数，否则保留4位小数
                if f.fract() == 0.0 { format!("{f:.0}") } else { format!("{f:.4}") }
            } else {
                n.to_string()
            }
        }
        other => other.to_string(),
    }
}

/// 单元测试模块
///
/// 测试文件位于 `tests/sop_status.rs`
#[cfg(test)]
#[path = "tests.rs"]
mod tests;
