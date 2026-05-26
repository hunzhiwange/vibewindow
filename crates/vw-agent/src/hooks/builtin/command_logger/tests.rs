//! 命令日志钩子测试模块
//!
//! 本模块包含 `CommandLoggerHook` 的单元测试，用于验证钩子是否正确记录工具调用信息。
//!
//! # 测试覆盖
//!
//! - 工具调用日志记录功能
//! - 日志条目格式验证（工具名称、执行时间、成功状态）

use super::*;

/// 命令日志钩子测试集
///
/// 包含验证命令日志钩子功能的测试用例。
#[allow(dead_code)]
mod tests {
    use super::*;

    /// 测试工具调用日志记录功能
    ///
    /// # 测试目的
    ///
    /// 验证 `CommandLoggerHook` 是否正确记录工具调用的日志信息，包括：
    /// - 工具名称
    /// - 执行耗时
    /// - 执行结果状态
    ///
    /// # 测试步骤
    ///
    /// 1. 创建一个新的命令日志钩子实例
    /// 2. 构造一个成功的工具执行结果
    /// 3. 调用钩子的 `on_after_tool_call` 方法记录日志
    /// 4. 验证日志条目数量为 1
    /// 5. 验证日志条目包含工具名称 "shell"
    /// 6. 验证日志条目包含执行时间 "42ms"
    /// 7. 验证日志条目包含成功状态 "success=true"
    ///
    /// # 示例
    ///
    /// ```ignore
    /// let hook = CommandLoggerHook::new();
    /// let result = ToolResult { success: true, output: "ok".into(), error: None };
    /// hook.on_after_tool_call("shell", &result, Duration::from_millis(42)).await;
    /// assert_eq!(hook.entries().len(), 1);
    /// ```
    #[tokio::test]
    async fn logs_tool_calls() {
        // 创建命令日志钩子实例
        let hook = CommandLoggerHook::new();

        // 构造一个成功的工具执行结果
        let result = ToolResult { success: true, output: "ok".into(), error: None };

        // 记录工具调用日志，工具名称为 "shell"，执行时间为 42 毫秒
        hook.on_after_tool_call("shell", &result, Duration::from_millis(42)).await;

        // 获取所有日志条目
        let entries = hook.entries();

        // 验证：应该只有一条日志记录
        assert_eq!(entries.len(), 1);

        // 验证：日志条目应包含工具名称
        assert!(entries[0].contains("shell"));

        // 验证：日志条目应包含执行时间
        assert!(entries[0].contains("42ms"));

        // 验证：日志条目应包含成功状态
        assert!(entries[0].contains("success=true"));
    }
}
