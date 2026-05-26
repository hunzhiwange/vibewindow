//! # 命令日志记录钩子
//!
//! 提供工具调用审计功能的钩子实现，用于记录所有工具执行结果。
//!
//! ## 功能特性
//!
//! - 记录工具调用的时间戳、名称、执行时长和结果状态
//! - 支持多线程安全的日志存储
//! - 自动输出到 tracing 日志系统
//!
//! ## 使用场景
//!
//! - 审计追踪：记录所有工具调用历史
//! - 性能分析：统计工具执行耗时
//! - 调试排障：回溯工具执行序列
//!
//! ## 优先级说明
//!
//! 该钩子优先级为 `-50`（较低），确保在其他业务钩子执行完毕后再记录日志。

use async_trait::async_trait;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::app::agent::hooks::traits::HookHandler;
use crate::app::agent::tools::traits::ToolResult;

/// 命令日志记录钩子
///
/// 该钩子实现 [`HookHandler`] trait，用于在工具调用完成后记录审计日志。
/// 所有日志条目存储在内存中的线程安全缓冲区中，同时输出到 tracing 系统。
///
/// # 线程安全性
///
/// 内部使用 `Arc<Mutex<Vec<String>>>` 存储日志，支持多线程并发写入。
///
/// # 示例
///
/// ```ignore
/// use crate::app::agent::hooks::builtin::CommandLoggerHook;
/// use crate::app::agent::hooks::traits::HookHandler;
///
/// let hook = CommandLoggerHook::new();
/// assert_eq!(hook.name(), "command-logger");
/// assert_eq!(hook.priority(), -50);
/// ```
pub struct CommandLoggerHook {
    /// 日志条目缓冲区
    ///
    /// 使用 `Arc<Mutex<>>` 包装以支持：
    /// - 多线程安全访问
    /// - 跨异步任务共享
    /// - 内部可变性
    log: Arc<Mutex<Vec<String>>>,
}

impl CommandLoggerHook {
    /// 创建新的命令日志记录钩子实例
    ///
    /// 初始化一个空的日志缓冲区，准备接收工具调用记录。
    ///
    /// # 返回值
    ///
    /// 返回一个新创建的 [`CommandLoggerHook`] 实例，其日志缓冲区为空。
    ///
    /// # 示例
    ///
    /// ```ignore
    /// let hook = CommandLoggerHook::new();
    /// ```
    pub fn new() -> Self {
        Self { log: Arc::new(Mutex::new(Vec::new())) }
    }

    /// 获取已记录的所有日志条目（仅测试可用）
    ///
    /// 返回日志缓冲区的完整副本，用于测试验证。
    ///
    /// # 返回值
    ///
    /// 返回所有已记录日志条目的克隆向量。
    ///
    /// # 注意
    ///
    /// 此方法仅在测试配置下可用（`#[cfg(test)]`）。
    #[cfg(test)]
    pub fn entries(&self) -> Vec<String> {
        self.log.lock().unwrap().clone()
    }
}

/// [`HookHandler`] trait 实现
///
/// 为 [`CommandLoggerHook`] 实现钩子处理接口，使其能够接入
/// VibeWindow 的钩子系统。使用条件编译属性以同时支持
/// WASM 和非 WASM 目标平台。
///
/// # 平台兼容性
///
/// - `wasm32` 目标：使用 `?Send` 标记，允许非 Send future
/// - 其他目标：标准 `async_trait` 实现
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl HookHandler for CommandLoggerHook {
    /// 返回钩子名称标识
    ///
    /// 该名称用于在日志和调试输出中标识此钩子实例。
    ///
    /// # 返回值
    ///
    /// 返回静态字符串 `"command-logger"`。
    fn name(&self) -> &str {
        "command-logger"
    }

    /// 返回钩子执行优先级
    ///
    /// 优先级为 `-50`，属于较低优先级。这确保该日志钩子
    /// 在其他业务逻辑钩子执行完毕后才执行，避免影响主流程。
    ///
    /// # 优先级规则
    ///
    /// - 数值越大，优先级越高，越先执行
    /// - 负值表示低优先级，后执行
    ///
    /// # 返回值
    ///
    /// 返回 `-50`。
    fn priority(&self) -> i32 {
        -50
    }

    /// 工具调用完成后的回调处理
    ///
    /// 当工具执行完成后，此方法被调用以记录执行信息。
    /// 日志条目包含时间戳、工具名称、执行时长和成功状态。
    ///
    /// # 参数
    ///
    /// - `tool`: 工具名称标识符
    /// - `result`: 工具执行结果，包含成功状态和输出数据
    /// - `duration`: 工具执行耗时
    ///
    /// # 行为说明
    ///
    /// 1. 格式化日志条目：`[HH:MM:SS] tool_name (XXXms) success=true/false`
    /// 2. 使用 tracing 输出 INFO 级别日志
    /// 3. 将条目追加到内存缓冲区
    ///
    /// # 示例输出
    ///
    /// ```text
    /// [14:30:45] shell (150ms) success=true
    /// ```
    async fn on_after_tool_call(&self, tool: &str, result: &ToolResult, duration: Duration) {
        // 构建日志条目：时间戳 + 工具名 + 耗时 + 成功状态
        let entry = format!(
            "[{}] {} ({}ms) success={}",
            chrono::Utc::now().format("%H:%M:%S"),
            tool,
            duration.as_millis(),
            result.success,
        );

        // 输出到 tracing 系统（INFO 级别）
        tracing::info!(hook = "command-logger", "{}", entry);

        // 追加到内存缓冲区
        self.log.lock().unwrap().push(entry);
    }
}

/// 单元测试模块
///
/// 测试文件位于同目录下的 `tests.rs`，通过 `#[path]` 属性指定路径。
#[cfg(test)]
#[path = "tests.rs"]
mod tests;
