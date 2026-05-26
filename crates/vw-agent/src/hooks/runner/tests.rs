//! HookRunner 测试模块
//!
//! 本模块提供 HookRunner 的单元测试，验证钩子系统的核心功能：
//! - 钩子注册与优先级排序
//! - 无返回值钩子的触发机制
//! - 修改型钩子的管道式处理
//! - 取消信号的传播行为

use super::*;

#[allow(dead_code)]
mod tests {
    use super::*;

    use async_trait::async_trait;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicU32, Ordering};

    /// 计数钩子 - 用于追踪无返回值事件触发次数
    ///
    /// 该钩子在心跳事件触发时递增计数器，主要用于测试钩子是否被正确调用。
    struct CountingHook {
        /// 钩子名称
        name: String,
        /// 优先级（数值越大越先执行）
        priority: i32,
        /// 触发计数器（原子引用计数，支持跨线程共享）
        fire_count: Arc<AtomicU32>,
    }

    impl CountingHook {
        /// 创建新的计数钩子实例
        ///
        /// # 参数
        /// - `name`: 钩子名称标识
        /// - `priority`: 执行优先级
        ///
        /// # 返回值
        /// 返回元组 (钩子实例, 可共享的计数器引用)
        fn new(name: &str, priority: i32) -> (Self, Arc<AtomicU32>) {
            let count = Arc::new(AtomicU32::new(0));
            (Self { name: name.to_string(), priority, fire_count: count.clone() }, count)
        }
    }

    #[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
    #[cfg_attr(not(target_arch = "wasm32"), async_trait)]
    impl HookHandler for CountingHook {
        fn name(&self) -> &str {
            &self.name
        }
        fn priority(&self) -> i32 {
            self.priority
        }
        /// 心跳事件处理 - 递增触发计数
        async fn on_heartbeat_tick(&self) {
            self.fire_count.fetch_add(1, Ordering::SeqCst);
        }
    }

    /// 大写转换钩子 - 将提示词转换为大写
    ///
    /// 该钩子实现修改型钩子行为，在提示词构建前将其转换为大写形式。
    struct UppercasePromptHook {
        /// 钩子名称
        name: String,
        /// 优先级
        priority: i32,
    }

    #[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
    #[cfg_attr(not(target_arch = "wasm32"), async_trait)]
    impl HookHandler for UppercasePromptHook {
        fn name(&self) -> &str {
            &self.name
        }
        fn priority(&self) -> i32 {
            self.priority
        }
        /// 提示词构建前置处理 - 将提示词转为大写
        async fn before_prompt_build(&self, prompt: String) -> HookResult<String> {
            HookResult::Continue(prompt.to_uppercase())
        }
    }

    /// 取消钩子 - 阻止提示词构建流程
    ///
    /// 该钩子用于测试取消信号传播机制，任何经过此钩子的提示词构建都会被取消。
    struct CancelPromptHook {
        /// 钩子名称
        name: String,
        /// 优先级
        priority: i32,
    }

    #[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
    #[cfg_attr(not(target_arch = "wasm32"), async_trait)]
    impl HookHandler for CancelPromptHook {
        fn name(&self) -> &str {
            &self.name
        }
        fn priority(&self) -> i32 {
            self.priority
        }
        /// 提示词构建前置处理 - 返回取消信号
        async fn before_prompt_build(&self, _prompt: String) -> HookResult<String> {
            HookResult::Cancel("blocked by policy".into())
        }
    }

    /// 后缀添加钩子 - 为提示词追加后缀
    ///
    /// 该钩子在提示词末尾添加指定后缀，用于测试修改型钩子的链式处理。
    struct SuffixPromptHook {
        /// 钩子名称
        name: String,
        /// 优先级
        priority: i32,
        /// 要追加的后缀内容
        suffix: String,
    }

    #[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
    #[cfg_attr(not(target_arch = "wasm32"), async_trait)]
    impl HookHandler for SuffixPromptHook {
        fn name(&self) -> &str {
            &self.name
        }
        fn priority(&self) -> i32 {
            self.priority
        }
        /// 提示词构建前置处理 - 追加后缀
        async fn before_prompt_build(&self, prompt: String) -> HookResult<String> {
            HookResult::Continue(format!("{}{}", prompt, self.suffix))
        }
    }

    /// 测试钩子注册与优先级排序
    ///
    /// 验证钩子按优先级从高到低排序：
    /// - 优先级 10 的钩子应排在第一位
    /// - 优先级 5 的钩子应排在第二位
    /// - 优先级 1 的钩子应排在第三位
    #[test]
    fn register_and_sort_by_priority() {
        let mut runner = HookRunner::new();

        // 创建三个不同优先级的钩子
        let (low, _) = CountingHook::new("low", 1);
        let (high, _) = CountingHook::new("high", 10);
        let (mid, _) = CountingHook::new("mid", 5);

        // 按随机顺序注册钩子
        runner.register(Box::new(low));
        runner.register(Box::new(high));
        runner.register(Box::new(mid));

        // 验证内部存储顺序：优先级从高到低
        let names: Vec<&str> = runner.handlers.iter().map(|h| h.name()).collect();
        assert_eq!(names, vec!["high", "mid", "low"]);
    }

    /// 测试无返回值钩子的全部触发机制
    ///
    /// 验证所有已注册钩子在心跳事件时都会被调用，
    /// 不受优先级影响（因为无返回值钩子不产生取消信号）。
    #[tokio::test]
    async fn void_hooks_fire_all_handlers() {
        let mut runner = HookRunner::new();

        // 注册两个计数钩子
        let (h1, c1) = CountingHook::new("hook_a", 0);
        let (h2, c2) = CountingHook::new("hook_b", 0);

        runner.register(Box::new(h1));
        runner.register(Box::new(h2));

        // 触发心跳事件
        runner.fire_heartbeat_tick().await;

        // 验证两个钩子的计数器都递增了
        assert_eq!(c1.load(Ordering::SeqCst), 1);
        assert_eq!(c2.load(Ordering::SeqCst), 1);
    }

    /// 测试修改型钩子的取消传播
    ///
    /// 验证高优先级钩子返回 Cancel 时，
    /// 后续钩子不会被调用，整个管道立即终止。
    #[tokio::test]
    async fn modifying_hook_can_cancel() {
        let mut runner = HookRunner::new();

        // 注册高优先级的取消钩子（优先级 10）
        runner.register(Box::new(CancelPromptHook { name: "blocker".into(), priority: 10 }));
        // 注册低优先级的大写钩子（优先级 0）
        runner.register(Box::new(UppercasePromptHook { name: "upper".into(), priority: 0 }));

        // 执行提示词构建管道
        let result = runner.run_before_prompt_build("hello".into()).await;

        // 验证结果为取消状态
        assert!(result.is_cancel());
    }

    /// 测试修改型钩子的管道式数据传递
    ///
    /// 验证多个修改型钩子按优先级顺序执行，
    /// 前一个钩子的输出作为下一个钩子的输入。
    #[tokio::test]
    async fn modifying_hook_pipelines_data() {
        let mut runner = HookRunner::new();

        // 优先级 10 先执行：将 "hello" 转为 "HELLO"
        runner.register(Box::new(UppercasePromptHook { name: "upper".into(), priority: 10 }));

        // 优先级 0 后执行：将 "HELLO" 追加后缀变为 "HELLO_done"
        runner.register(Box::new(SuffixPromptHook {
            name: "suffix".into(),
            priority: 0,
            suffix: "_done".into(),
        }));

        // 执行管道处理
        match runner.run_before_prompt_build("hello".into()).await {
            HookResult::Continue(result) => assert_eq!(result, "HELLO_done"),
            HookResult::Cancel(_) => panic!("should not cancel"),
        }
    }
}
