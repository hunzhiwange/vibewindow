//! 用量视图的数据与展示行为测试，覆盖聚合结果和边界输入。

use super::data::UsageData;
use crate::app::App;
use crate::app::models::{ChatMessage, ChatRole};

#[test]
fn usage_data_counts_tool_messages_separately() {
    let (mut app, _task) = App::new();
    app.chat = vec![
        ChatMessage { role: ChatRole::User, content: "user".to_string(), think_timing: Vec::new() },
        ChatMessage {
            role: ChatRole::Assistant,
            content: "assistant".to_string(),
            think_timing: Vec::new(),
        },
        ChatMessage {
            role: ChatRole::Tool,
            content: "tool grep\n{\"status\":\"completed\",\"output\":\"2 matches\"}\n".to_string(),
            think_timing: Vec::new(),
        },
        ChatMessage {
            role: ChatRole::System,
            content: "system".to_string(),
            think_timing: Vec::new(),
        },
    ];

    let usage = UsageData::from_app(&app);

    assert_eq!(usage.message_count, 4);
    assert_eq!(usage.user_msgs, 1);
    assert_eq!(usage.assistant_msgs, 1);
    assert_eq!(usage.system_msgs, 1);
    assert_eq!(usage.tool_msgs, 1);
}
