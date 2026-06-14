use super::*;

#[test]
fn new_hook_has_expected_identity_priority_and_empty_log() {
    let hook = CommandLoggerHook::new();

    assert_eq!(hook.name(), "command-logger");
    assert_eq!(hook.priority(), -50);
    assert!(hook.entries().is_empty());
}

#[tokio::test]
async fn on_after_tool_call_records_success_and_failure_entries_in_order() {
    let hook = CommandLoggerHook::new();
    let success = ToolResult { success: true, output: "ok".into(), error: None };
    let failure =
        ToolResult { success: false, output: String::new(), error: Some("failed".into()) };

    hook.on_after_tool_call("shell", &success, Duration::from_millis(42)).await;
    hook.on_after_tool_call("git", &failure, Duration::from_micros(1_500)).await;

    let entries = hook.entries();
    assert_eq!(entries.len(), 2);
    assert!(entries[0].contains("shell"));
    assert!(entries[0].contains("42ms"));
    assert!(entries[0].contains("success=true"));
    assert!(entries[1].contains("git"));
    assert!(entries[1].contains("1ms"));
    assert!(entries[1].contains("success=false"));
}

#[tokio::test]
async fn on_after_tool_call_records_zero_duration_without_dropping_tool_name() {
    let hook = CommandLoggerHook::new();
    let result = ToolResult::default();

    hook.on_after_tool_call("noop", &result, Duration::ZERO).await;

    let entries = hook.entries();
    assert_eq!(entries.len(), 1);
    assert!(entries[0].contains("noop"));
    assert!(entries[0].contains("0ms"));
}
