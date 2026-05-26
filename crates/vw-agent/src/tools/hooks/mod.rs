//! 工具 Hook 适配层。
//!
//! 当前仓库里已经有通用的 `HookRunner`，但工具执行路径一直是各处直接调
//! `run_before_tool_call` / `fire_after_tool_call`。本模块把这些调用收口成窄适配层，
//! 这样 02 阶段之后权限流水线和工具执行器只依赖这里，而不再感知 HookRunner
//! 的具体 API 细节。

use super::decision::PermissionDecision;
use super::toolset::ToolCallError;
use super::{ToolCallResult, ToolResult};
use crate::app::agent::hooks::{HookResult, HookRunner};
use serde_json::Value;
use std::time::Duration;

/// 工具调用前 Hook 适配器。
pub(crate) struct PreToolHook;

impl PreToolHook {
    /// 运行工具前 Hook，并返回可能被修改后的工具名与参数。
    pub async fn run(
        runner: Option<&HookRunner>,
        tool_name: String,
        input: Value,
    ) -> Result<(String, Value), ToolCallError> {
        let Some(runner) = runner else {
            return Ok((tool_name, input));
        };

        match runner.run_before_tool_call(tool_name, input).await {
            HookResult::Continue((name, args)) => Ok((name, args)),
            HookResult::Cancel(reason) => {
                Err(ToolCallError::denied(format!("Cancelled by hook: {reason}")))
            }
        }
    }
}

/// 权限决策 Hook 适配器。
pub(crate) struct PermissionHook;

impl PermissionHook {
    /// 当前阶段仅对外提供统一入口，后续若 HookRunner 增加权限决策钩子，
    /// 只需要在这里接入即可。
    pub fn adapt(decision: PermissionDecision) -> PermissionDecision {
        decision
    }
}

/// 工具调用后 Hook 适配器。
pub(crate) struct PostToolHook;

impl PostToolHook {
    /// 使用结构化结果触发工具调用后 Hook。
    pub async fn run(
        runner: Option<&HookRunner>,
        tool_name: &str,
        result: &ToolCallResult,
        duration: Duration,
    ) {
        let legacy = ToolResult {
            success: result.is_success(),
            output: result.model_text(),
            error: result.error_text(),
        };
        Self::run_legacy(runner, tool_name, &legacy, duration).await;
    }

    /// 使用旧结果结构触发工具调用后 Hook。
    pub async fn run_legacy(
        runner: Option<&HookRunner>,
        tool_name: &str,
        result: &ToolResult,
        duration: Duration,
    ) {
        if let Some(runner) = runner {
            runner.fire_after_tool_call(tool_name, result, duration).await;
        }
    }
}
#[cfg(test)]
mod tests;
