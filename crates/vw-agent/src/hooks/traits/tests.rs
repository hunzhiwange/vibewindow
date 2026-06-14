//! 钩子 trait 单元测试模块
//!
//! 本模块包含对 `HookHandler` trait 及相关功能的单元测试，验证钩子系统的核心行为：
//! - 钩子结果状态判断（取消/继续）
//! - 默认优先级机制
//! - 默认钩子行为（透传）
//!
//! ## 测试覆盖
//!
//! - `hook_result_is_cancel`: 测试 `HookResult::is_cancel()` 方法的正确性
//! - `default_priority_is_zero`: 验证未覆盖 `priority()` 时的默认值为 0
//! - `default_modifying_hooks_pass_through`: 验证未覆盖的钩子方法默认透传输入

use super::*;

#[allow(dead_code)]
mod tests {
    use super::*;

    /// 测试钩子结构体
    ///
    /// 用于单元测试的简单钩子实现，包含可配置的名称和优先级。
    /// 实现了 `HookHandler` trait 以便在测试中验证钩子行为。
    struct TestHook {
        /// 钩子名称标识符
        name: String,
        /// 钩子执行优先级（数值越小优先级越高）
        priority: i32,
    }

    impl TestHook {
        /// 创建新的测试钩子实例
        ///
        /// # 参数
        ///
        /// - `name`: 钩子名称
        /// - `priority`: 钩子优先级
        ///
        /// # 返回值
        ///
        /// 返回配置好的 `TestHook` 实例
        fn new(name: &str, priority: i32) -> Self {
            Self { name: name.to_string(), priority }
        }
    }

    #[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
    #[cfg_attr(not(target_arch = "wasm32"), async_trait)]
    impl HookHandler for TestHook {
        /// 返回钩子名称
        fn name(&self) -> &str {
            &self.name
        }

        /// 返回钩子优先级
        fn priority(&self) -> i32 {
            self.priority
        }
    }

    /// 测试 `HookResult::is_cancel()` 方法的行为
    ///
    /// 验证：
    /// - `HookResult::Continue` 变体的 `is_cancel()` 应返回 `false`
    /// - `HookResult::Cancel` 变体的 `is_cancel()` 应返回 `true`
    #[test]
    fn hook_result_is_cancel() {
        // Continue 变体不应被判定为取消
        let ok: HookResult<String> = HookResult::Continue("hi".into());
        assert!(!ok.is_cancel());

        // Cancel 变体应被判定为取消
        let cancel: HookResult<String> = HookResult::Cancel("blocked".into());
        assert!(cancel.is_cancel());
    }

    /// 测试默认优先级为零
    ///
    /// 验证：当 `HookHandler` trait 实现未覆盖 `priority()` 方法时，
    /// 默认实现应返回 0。
    #[test]
    fn default_priority_is_zero() {
        /// 最小化钩子结构体（仅实现必需的 name 方法）
        struct MinimalHook;

        #[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
        #[cfg_attr(not(target_arch = "wasm32"), async_trait)]
        impl HookHandler for MinimalHook {
            fn name(&self) -> &str {
                "minimal"
            }
        }

        // 未覆盖 priority() 时应返回默认值 0
        assert_eq!(MinimalHook.priority(), 0);
    }

    fn expect_continue<T>(result: HookResult<T>) -> T {
        match result {
            HookResult::Continue(value) => value,
            HookResult::Cancel(reason) => panic!("unexpected cancel: {reason}"),
        }
    }

    fn channel_message(content: &str) -> ChannelMessage {
        ChannelMessage {
            id: "msg-1".to_string(),
            sender: "sender".to_string(),
            reply_target: "reply".to_string(),
            content: content.to_string(),
            channel: "cli".to_string(),
            timestamp: 7,
            thread_ts: None,
        }
    }

    /// 测试默认空钩子是 no-op。
    ///
    /// 所有 void hook 默认实现都应可被安全调用，不产生 panic 或额外约束。
    #[tokio::test]
    async fn default_void_hooks_are_noops() {
        let hook = TestHook::new("test", 0);
        let messages = vec![ChatMessage::user("hello")];
        let response = ChatResponse {
            text: Some("ok".to_string()),
            tool_calls: Vec::new(),
            usage: None,
            reasoning_content: None,
        };
        let result = ToolResult { success: true, output: "done".to_string(), error: None };

        hook.on_gateway_start("localhost", 3000).await;
        hook.on_gateway_stop().await;
        hook.on_session_start("session", "telegram").await;
        hook.on_session_end("session", "telegram").await;
        hook.on_llm_input(&messages, "model").await;
        hook.on_llm_output(&response).await;
        hook.on_after_tool_call("shell", &result, std::time::Duration::from_millis(3)).await;
        hook.on_message_sent("cli", "user", "hello").await;
        hook.on_heartbeat_tick().await;

        assert_eq!(hook.name(), "test");
    }

    /// 测试默认修改钩子透传行为
    ///
    /// 验证：当 `HookHandler` 未覆盖修改型钩子方法时，
    /// 默认实现应透传输入（不修改、不取消）。
    #[tokio::test]
    async fn default_modifying_hooks_pass_through_all_inputs() {
        let hook = TestHook::new("test", 0);

        assert_eq!(
            expect_continue(hook.before_model_resolve("openai".into(), "gpt".into()).await),
            ("openai".to_string(), "gpt".to_string())
        );
        assert_eq!(expect_continue(hook.before_prompt_build("prompt".into()).await), "prompt");

        let (messages, model) =
            expect_continue(hook.before_llm_call(vec![ChatMessage::user("hi")], "m".into()).await);
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].content, "hi");
        assert_eq!(model, "m");

        let (name, args) = expect_continue(
            hook.before_tool_call("shell".into(), serde_json::json!({"cmd": "ls"})).await,
        );
        assert_eq!(name, "shell");
        assert_eq!(args["cmd"], "ls");

        let received = expect_continue(hook.on_message_received(channel_message("body")).await);
        assert_eq!(received.content, "body");

        assert_eq!(
            expect_continue(
                hook.on_message_sending("cli".into(), "user".into(), "hello".into()).await
            ),
            ("cli".to_string(), "user".to_string(), "hello".to_string())
        );
    }
}
