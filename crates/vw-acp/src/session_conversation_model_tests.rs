//! 会话对话模型持久化测试。
//!
//! 该文件验证 ACP 会话更新被转换为可持久化的对话 DTO 后，仍满足持久化 key
//! 策略，并保留工具结果的结构化渲染信息。

use agent_client_protocol::SessionNotification;
use serde_json::json;

use super::{create_session_conversation, record_session_update};
use crate::{SessionMessage, SessionToolResultContent, find_persisted_key_policy_violations};

fn tool_call_update_notification(raw_output: serde_json::Value) -> SessionNotification {
    serde_json::from_value(json!({
        "sessionId": "session-1",
        "update": {
            "sessionUpdate": "tool_call_update",
            "toolCallId": "tool-1",
            "kind": "apply_patch",
            "title": "apply_patch",
            "status": "completed",
            "rawOutput": raw_output
        }
    }))
    .expect("valid ACP tool_call_update notification")
}

#[test]
fn record_session_update_persists_tool_result_dto() {
    let mut conversation = create_session_conversation(Some("2026-04-21T00:00:00Z"));
    let notification = tool_call_update_notification(json!({
        "tool_use_id": "tool-1",
        "success": true,
        "content": [
            {
                "type": "structured_patch",
                "hunks": [
                    {
                        "path": "src/main.rs",
                        "header": "@@ -1,1 +1,1 @@",
                        "lines": ["-old", "+new"]
                    }
                ]
            }
        ],
        "data": {
            "changed_files": ["src/main.rs"]
        },
        "model_result": "patched src/main.rs",
        "render_hint": {
            "kind": "structured_patch",
            "summary": "Updated 1 file",
            "metadata": {
                "outputPath": "src/main.rs"
            }
        }
    }));

    let _state =
        record_session_update(&mut conversation, None, &notification, Some("2026-04-21T00:00:01Z"));

    let agent = match conversation.messages.last() {
        Some(SessionMessage::Agent(agent)) => agent,
        other => panic!("expected trailing agent message, got {other:?}"),
    };
    let tool_result = agent.tool_results.get("tool-1").expect("tool result persisted");

    assert_eq!(tool_result.tool_use_id, "tool-1");
    assert_eq!(tool_result.tool_name, "apply_patch");
    assert!(!tool_result.is_error);
    assert!(matches!(
        &tool_result.content,
        SessionToolResultContent::Text(text) if text == "Updated 1 file"
    ));

    let stored = tool_result.result.as_ref().expect("structured dto persisted");
    assert_eq!(stored.tool_use_id.as_deref(), Some("tool-1"));
    assert_eq!(stored.tool_id.as_ref().map(|tool_id| tool_id.as_ref()), Some("apply_patch"));
    assert_eq!(
        stored.render_hint.as_ref().and_then(|hint| hint.summary.as_deref()),
        Some("Updated 1 file")
    );

    let serialized = serde_json::to_value(&conversation).expect("conversation serializes");
    let violations = find_persisted_key_policy_violations(&serialized);
    assert!(violations.is_empty(), "unexpected persisted key policy violations: {violations:?}");
}
