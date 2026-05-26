//! 工具执行器历史消息构造的测试。
//!
//! 当前覆盖原生 tool message ID 存在时，历史输出应保留为 tool 角色消息。

use super::*;

#[test]
fn build_tool_result_history_messages_uses_native_tool_messages_when_ids_exist() {
    let messages = build_tool_result_history_messages(
        &[],
        &[ToolResultHistoryEntry {
            tool_name: "file_read".to_string(),
            tool_call_id: Some("call_1".to_string()),
            output: "hello".to_string(),
        }],
        true,
    );

    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0].role, "tool");
    assert!(messages[0].content.contains("call_1"));
    assert!(messages[0].content.contains("hello"));
}
