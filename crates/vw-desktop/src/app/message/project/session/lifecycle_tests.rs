//! 覆盖项目会话生命周期逻辑，验证打开、加载和重置流程。
//!
//! 注释聚焦模块职责、消息边界和失败处理方式，帮助维护者在不改变逻辑的前提下理解代码。

use super::lifecycle::loaded_chat_from_gateway_messages;
use crate::app::models::ChatRole;
use serde_json::{Map, Value, json};
use vw_shared::message::types as agent_message;

fn app() -> crate::app::App {
    crate::app::App::new().0
}

fn session_info(id: &str, title: &str, directory: &str) -> vw_shared::session::info::Info {
    vw_shared::session::info::Info {
        id: id.to_string(),
        slug: id.to_string(),
        project_id: "project".to_string(),
        directory: directory.to_string(),
        parent_id: None,
        summary: None,
        share: None,
        title: title.to_string(),
        version: "1".to_string(),
        time: vw_shared::session::info::TimeInfo {
            created: 1,
            updated: 1,
            compacting: None,
            archived: None,
        },
        permission: None,
        revert: None,
    }
}

fn user_info(id: &str) -> agent_message::Info {
    agent_message::Info::User(Box::new(agent_message::UserInfo {
        id: id.to_string(),
        session_id: "session-1".to_string(),
        time: agent_message::UserTime { created: 5 },
        summary: None,
        agent: "default".to_string(),
        model: agent_message::ModelRef {
            provider_id: "provider-a".to_string(),
            model_id: "model-a".to_string(),
        },
        system: None,
        tools: None,
        variant: None,
    }))
}

fn assistant_info(id: &str) -> agent_message::Info {
    agent_message::Info::Assistant(Box::new(agent_message::AssistantInfo {
        id: id.to_string(),
        session_id: "session-1".to_string(),
        time: agent_message::AssistantTime { created: 10, completed: Some(20) },
        error: None,
        parent_id: "user-1".to_string(),
        model_id: "model-a".to_string(),
        provider_id: "provider-a".to_string(),
        mode: "chat".to_string(),
        agent: "default".to_string(),
        path: agent_message::PathInfo { cwd: "/tmp".to_string(), root: "/tmp".to_string() },
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

fn tool_message(id: &str, state: agent_message::ToolState) -> agent_message::WithParts {
    agent_message::WithParts {
        info: assistant_info(id),
        parts: vec![agent_message::Part::Tool(agent_message::ToolPart {
            base: part_base(&format!("part-{id}"), id),
            call_id: format!("call-{id}"),
            tool: "bash".to_string(),
            state,
            metadata: None,
        })],
    }
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
                time: agent_message::PartTime { start: 100, end: Some(140) },
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
    assert_eq!(
        message_ids,
        vec![
            Some("assistant-1".to_string()),
            Some("assistant-1".to_string()),
            Some("assistant-1".to_string()),
        ]
    );
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
                time: agent_message::PartTime { start: 200, end: None },
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

#[test]
fn loaded_chat_from_gateway_messages_keeps_user_text_file_and_message_ids() {
    let messages = vec![agent_message::WithParts {
        info: user_info("user-1"),
        parts: vec![
            agent_message::Part::Text(agent_message::TextPart {
                base: part_base("part-u1", "user-1"),
                text: "看这个文件".to_string(),
                synthetic: None,
                ignored: None,
                time: None,
                metadata: None,
            }),
            agent_message::Part::File(agent_message::FilePart {
                base: part_base("part-f1", "user-1"),
                url: "file:///tmp/a.rs".to_string(),
                mime: "text/rust".to_string(),
                filename: Some("a.rs".to_string()),
                source: None,
            }),
        ],
    }];

    let (chat, ids) = loaded_chat_from_gateway_messages(messages);

    assert_eq!(chat.len(), 1);
    assert_eq!(chat[0].role, ChatRole::User);
    assert!(chat[0].content.contains("看这个文件"));
    assert!(chat[0].content.contains("[File: file:///tmp/a.rs]"));
    assert_eq!(ids, vec![Some("user-1".to_string())]);
}

#[test]
fn loaded_chat_from_gateway_messages_covers_pending_running_completed_and_error_payloads() {
    let mut running_metadata = Map::new();
    running_metadata.insert("permissionRequest".to_string(), json!({"id": "perm-1"}));
    running_metadata.insert("render_hint".to_string(), json!("compact"));

    let mut completed_metadata = Map::new();
    completed_metadata.insert("toolCallId".to_string(), json!("explicit-call"));

    let messages = vec![
        tool_message(
            "pending",
            agent_message::ToolState::Pending(agent_message::ToolStatePending {
                input: Map::from_iter([("arg".to_string(), json!("value"))]),
                raw: " raw input ".to_string(),
            }),
        ),
        tool_message(
            "running",
            agent_message::ToolState::Running(agent_message::ToolStateRunning {
                input: Map::new(),
                title: Some("Running".to_string()),
                metadata: Some(running_metadata),
                time: agent_message::PartTime { start: 1, end: Some(2) },
            }),
        ),
        tool_message(
            "completed",
            agent_message::ToolState::Completed(agent_message::ToolStateCompleted {
                input: Map::from_iter([("path".to_string(), json!("/tmp/a"))]),
                output: "done".to_string(),
                title: "Done".to_string(),
                metadata: completed_metadata,
                time: agent_message::ToolStateCompletedTime {
                    start: 3,
                    end: 4,
                    compacted: Some(5),
                },
                attachments: Some(vec![agent_message::ToolAttachment {
                    url: "file:///tmp/out.txt".to_string(),
                    mime: "text/plain".to_string(),
                }]),
            }),
        ),
        tool_message(
            "denied",
            agent_message::ToolState::Error(agent_message::ToolStateError {
                input: Map::new(),
                error: "Approval required".to_string(),
                metadata: None,
                time: agent_message::PartTime { start: 6, end: None },
            }),
        ),
        tool_message(
            "error",
            agent_message::ToolState::Error(agent_message::ToolStateError {
                input: Map::from_iter([("cmd".to_string(), json!("false"))]),
                error: "process failed".to_string(),
                metadata: Some(Map::from_iter([("summary".to_string(), json!("失败"))])),
                time: agent_message::PartTime { start: 7, end: Some(8) },
            }),
        ),
    ];

    let (chat, ids) = loaded_chat_from_gateway_messages(messages);

    assert_eq!(chat.len(), 5);
    assert_eq!(ids.len(), 5);

    let pending = tool_payload(&chat[0].content);
    assert_eq!(pending.get("status"), Some(&json!("pending")));
    assert_eq!(pending.get("input"), Some(&json!(" raw input ")));

    let running = tool_payload(&chat[1].content);
    assert_eq!(running.get("status"), Some(&json!("running")));
    assert_eq!(running.get("title"), Some(&json!("Running")));
    assert_eq!(running.get("permission_request"), Some(&json!({"id": "perm-1"})));
    assert_eq!(running.get("renderHint"), Some(&json!("compact")));

    let completed = tool_payload(&chat[2].content);
    assert_eq!(completed.get("status"), Some(&json!("completed")));
    assert_eq!(completed.get("toolCallId"), Some(&json!("call-completed")));
    assert_eq!(completed.pointer("/metadata/toolCallId"), Some(&json!("explicit-call")));
    assert!(completed.get("attachments").is_some());

    let denied = tool_payload(&chat[3].content);
    assert_eq!(denied.get("status"), Some(&json!("denied")));

    let error = tool_payload(&chat[4].content);
    assert_eq!(error.get("status"), Some(&json!("error")));
    assert_eq!(error.get("summary"), Some(&json!("失败")));
}

#[test]
fn session_context_menu_and_rename_state_are_updated_synchronously() {
    let mut app = app();
    app.sessions = vec![session_info("s1", "First", "/tmp/project")];
    app.project_sessions
        .insert("/tmp/other".to_string(), vec![session_info("s2", "Second", "/tmp/other")]);

    let _ = super::lifecycle::handle(
        &mut app,
        crate::app::message::project::ProjectMessage::SessionRightClicked(
            "s1".to_string(),
            10.0,
            20.0,
        ),
    );
    assert_eq!(app.session_menu_id.as_deref(), Some("s1"));
    assert_eq!(app.session_menu_anchor.expect("anchor").x, 10.0);

    let _ = super::lifecycle::handle(
        &mut app,
        crate::app::message::project::ProjectMessage::SessionMenuClose,
    );
    assert!(app.session_menu_id.is_none());
    assert!(app.session_menu_anchor.is_none());

    let _ = super::lifecycle::handle(
        &mut app,
        crate::app::message::project::ProjectMessage::SessionRenamePressed("s2".to_string()),
    );
    assert_eq!(app.session_rename_id.as_deref(), Some("s2"));
    assert_eq!(app.session_rename_value, "Second");

    let _ = super::lifecycle::handle(
        &mut app,
        crate::app::message::project::ProjectMessage::SessionRenameChanged(" Renamed ".to_string()),
    );
    assert_eq!(app.session_rename_value, " Renamed ");

    let _ = super::lifecycle::handle(
        &mut app,
        crate::app::message::project::ProjectMessage::SessionRenameCancel,
    );
    assert!(app.session_rename_id.is_none());
    assert!(app.session_rename_value.is_empty());
}

#[test]
fn session_rename_save_ignores_missing_or_empty_title_and_updates_local_titles() {
    let mut app = app();
    app.sessions = vec![session_info("s1", "First", "/tmp/project")];
    app.project_sessions
        .insert("/tmp/project".to_string(), vec![session_info("s1", "First", "/tmp/project")]);

    let _ = super::lifecycle::handle(
        &mut app,
        crate::app::message::project::ProjectMessage::SessionRenameSave,
    );
    assert_eq!(app.sessions[0].title, "First");

    app.session_rename_id = Some("s1".to_string());
    app.session_rename_value = "   ".to_string();
    let _ = super::lifecycle::handle(
        &mut app,
        crate::app::message::project::ProjectMessage::SessionRenameSave,
    );
    assert_eq!(app.sessions[0].title, "First");
    assert_eq!(app.session_rename_id.as_deref(), Some("s1"));

    app.session_rename_value = "Updated".to_string();
    let task = super::lifecycle::handle(
        &mut app,
        crate::app::message::project::ProjectMessage::SessionRenameSave,
    );

    assert!(task.is_some());
    assert!(app.session_rename_id.is_none());
    assert!(app.session_rename_value.is_empty());
    assert_eq!(app.sessions[0].title, "Updated");
    assert_eq!(app.project_sessions.get("/tmp/project").unwrap()[0].title, "Updated");
}

#[test]
fn session_created_inserts_unique_session_and_resets_active_chat_state() {
    let mut app = app();
    let info = session_info("s1", "First", "/tmp/project");
    app.project_path = Some("/tmp/project".to_string());
    app.sessions = vec![info.clone()];
    app.chat.push(crate::app::models::ChatMessage {
        role: ChatRole::User,
        content: "old".to_string(),
        think_timing: Vec::new(),
    });
    app.chat_message_ids = vec![Some("old".to_string())];

    let task = super::lifecycle::handle(
        &mut app,
        crate::app::message::project::ProjectMessage::SessionCreated(info.clone()),
    );

    assert!(task.is_some());
    assert_eq!(app.new_session_last_directory.as_deref(), Some("/tmp/project"));
    assert_eq!(app.sessions.iter().filter(|s| s.id == "s1").count(), 1);
    assert_eq!(app.project_sessions.get("/tmp/project").unwrap()[0].id, "s1");
    assert_eq!(app.active_session_id.as_deref(), Some("s1"));
    assert!(app.chat.is_empty());
    assert!(app.chat_message_ids.is_empty());
    assert_eq!(app.usage, crate::app::models::TokenUsage::default());
}

#[test]
fn session_copied_adds_to_current_and_project_session_lists_without_duplicates() {
    let mut app = app();
    let info = session_info("s1", "Copy", "/tmp/project");
    app.project_path = Some("/tmp/project".to_string());
    app.sessions = vec![info.clone()];

    let _ = super::lifecycle::handle(
        &mut app,
        crate::app::message::project::ProjectMessage::SessionCopied(info.clone()),
    );
    assert_eq!(app.sessions.iter().filter(|s| s.id == "s1").count(), 1);
    assert_eq!(app.project_sessions.get("/tmp/project").unwrap().len(), 1);

    let second = session_info("s2", "Copy 2", "/tmp/project");
    let _ = super::lifecycle::handle(
        &mut app,
        crate::app::message::project::ProjectMessage::SessionCopied(second),
    );
    assert_eq!(app.sessions[0].id, "s2");
    assert_eq!(app.project_sessions.get("/tmp/project").unwrap()[0].id, "s2");
}
