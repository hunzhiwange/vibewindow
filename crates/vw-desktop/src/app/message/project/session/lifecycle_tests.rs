//! 覆盖项目会话生命周期逻辑，验证打开、加载和重置流程。
//!
//! 注释聚焦模块职责、消息边界和失败处理方式，帮助维护者在不改变逻辑的前提下理解代码。

use super::lifecycle::loaded_chat_from_gateway_messages;
use crate::app::models::ChatRole;
use serde_json::{Map, Value, json};
use vw_shared::message::types as agent_message;

fn assistant_info(id: &str) -> agent_message::Info {
    agent_message::Info::Assistant(Box::new(agent_message::AssistantInfo {
        id: id.to_string(),
        session_id: "session-1".to_string(),
        time: agent_message::AssistantTime {
            created: 10,
            completed: Some(20),
        },
        error: None,
        parent_id: "user-1".to_string(),
        model_id: "model-a".to_string(),
        provider_id: "provider-a".to_string(),
        mode: "chat".to_string(),
        agent: "default".to_string(),
        path: agent_message::PathInfo {
            cwd: "/tmp".to_string(),
            root: "/tmp".to_string(),
        },
        summary: None,
        cost: 0.0,
        tokens: agent_message::TokenInfo {
            total: None,
            input: 0,
            output: 0,
            reasoning: 0,
            cache: agent_message::TokenCacheInfo { read: 0, write: 0 },
        },
        variant: None,
        finish: None,
    }))
}

fn part_base(part_id: &str, message_id: &str) -> agent_message::PartBase {
    agent_message::PartBase {
        id: part_id.to_string(),
        session_id: "session-1".to_string(),
        message_id: message_id.to_string(),
    }
}

fn tool_payload(raw: &str) -> Value {
    let (_, payload) = raw.split_once('\n').expect("tool payload should contain a newline");
    serde_json::from_str(payload.trim()).expect("tool payload should be valid json")
}

#[test]
fn loaded_chat_from_gateway_messages_splits_tool_parts_into_tool_rows() {
    let mut metadata = Map::new();
    metadata.insert("summary".to_string(), Value::String("2 次匹配".to_string()));

    let messages = vec![agent_message::WithParts {
        info: assistant_info("assistant-1"),
        parts: vec![
            agent_message::Part::Reasoning(agent_message::ReasoningPart {
                base: part_base("part-r1", "assistant-1"),
                text: "先缩小范围".to_string(),
                metadata: None,
                time: agent_message::PartTime {
                    start: 100,
                    end: Some(140),
                },
            }),
            agent_message::Part::Text(agent_message::TextPart {
                base: part_base("part-t1", "assistant-1"),
                text: "先看搜索结果。".to_string(),
                synthetic: None,
                ignored: None,
                time: None,
                metadata: None,
            }),
            agent_message::Part::Tool(agent_message::ToolPart {
                base: part_base("part-tool-1", "assistant-1"),
                call_id: "call-1".to_string(),
                tool: "grep".to_string(),
                state: agent_message::ToolState::Completed(agent_message::ToolStateCompleted {
                    input: Map::from_iter([(
                        "pattern".to_string(),
                        Value::String("foo".to_string()),
                    )]),
                    output: "2 matches".to_string(),
                    title: "搜索 foo".to_string(),
                    metadata,
                    time: agent_message::ToolStateCompletedTime {
                        start: 150,
                        end: 180,
                        compacted: None,
                    },
                    attachments: None,
                }),
                metadata: None,
            }),
            agent_message::Part::Text(agent_message::TextPart {
                base: part_base("part-t2", "assistant-1"),
                text: "最终答案。".to_string(),
                synthetic: None,
                ignored: None,
                time: None,
                metadata: None,
            }),
        ],
    }];

    let (chat, message_ids) = loaded_chat_from_gateway_messages(messages);

    assert_eq!(chat.len(), 3);
    assert_eq!(chat[0].role, ChatRole::Assistant);
    assert_eq!(chat[0].content, "<think>先缩小范围</think>先看搜索结果。");
    assert_eq!(chat[0].think_timing.len(), 1);
    assert_eq!(chat[0].think_timing[0].start_ms, 100);
    assert_eq!(chat[0].think_timing[0].end_ms, Some(140));

    assert_eq!(chat[1].role, ChatRole::Tool);
    let payload = tool_payload(&chat[1].content);
    assert_eq!(payload.get("status"), Some(&json!("completed")));
    assert_eq!(payload.get("output"), Some(&json!("2 matches")));
    assert_eq!(payload.get("callID"), Some(&json!("call-1")));
    assert_eq!(payload.get("summary"), Some(&json!("2 次匹配")));

    assert_eq!(chat[2].role, ChatRole::Assistant);
    assert_eq!(chat[2].content, "最终答案。");
    assert_eq!(message_ids, vec![
        Some("assistant-1".to_string()),
        Some("assistant-1".to_string()),
        Some("assistant-1".to_string()),
    ]);
}

#[test]
fn loaded_chat_from_gateway_messages_skips_empty_assistant_shell_for_tool_only_messages() {
    let messages = vec![agent_message::WithParts {
        info: assistant_info("assistant-2"),
        parts: vec![agent_message::Part::Tool(agent_message::ToolPart {
            base: part_base("part-tool-2", "assistant-2"),
            call_id: "call-2".to_string(),
            tool: "question".to_string(),
            state: agent_message::ToolState::Running(agent_message::ToolStateRunning {
                input: Map::from_iter([(
                    "questions".to_string(),
                    json!([{"header": "确认", "question": "继续吗？"}]),
                )]),
                title: Some("等待确认".to_string()),
                metadata: None,
                time: agent_message::PartTime {
                    start: 200,
                    end: None,
                },
            }),
            metadata: None,
        })],
    }];

    let (chat, message_ids) = loaded_chat_from_gateway_messages(messages);

    assert_eq!(chat.len(), 1);
    assert_eq!(chat[0].role, ChatRole::Tool);
    let payload = tool_payload(&chat[0].content);
    assert_eq!(payload.get("status"), Some(&json!("running")));
    assert_eq!(payload.get("callID"), Some(&json!("call-2")));
    assert_eq!(message_ids, vec![Some("assistant-2".to_string())]);
}