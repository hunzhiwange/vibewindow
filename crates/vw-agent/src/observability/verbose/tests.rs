//! verbose 观测器的基础回归测试。
//!
//! 这些测试只验证观测器的稳定外部行为：名称保持不变，并且各类事件
//! 都能被记录而不触发 panic。输出内容由观测器实现负责，这里避免把
//! 测试耦合到日志格式。

use super::*;
use std::time::Duration;

#[test]
fn verbose_name() {
    assert_eq!(VerboseObserver::new().name(), "verbose");
}

#[test]
fn verbose_events_do_not_panic() {
    let obs = VerboseObserver::new();
    // 覆盖 LLM、工具调用与回合结束事件，确保新增事件字段时仍保持宽容处理。
    obs.record_event(&ObserverEvent::LlmRequest {
        provider: "openrouter".into(),
        model: "claude".into(),
        messages_count: 3,
    });
    obs.record_event(&ObserverEvent::LlmResponse {
        provider: "openrouter".into(),
        model: "claude".into(),
        duration: Duration::from_millis(12),
        success: true,
        error_message: None,
        input_tokens: Some(50),
        output_tokens: Some(25),
        cached_tokens: Some(5),
        reasoning_tokens: Some(2),
    });
    obs.record_event(&ObserverEvent::ToolCallStart { tool: "shell".into() });
    obs.record_event(&ObserverEvent::ToolCall {
        tool: "shell".into(),
        duration: Duration::from_millis(2),
        success: true,
    });
    obs.record_event(&ObserverEvent::TurnComplete);
}
