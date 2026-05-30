//! 会话 Gateway 处理器的持久化行为测试。
//!
//! 这些测试覆盖流式聊天落盘时的模型标识拆分、用户/助手消息生成以及预分配
//! 消息 id 的复用，确保 HTTP 流式响应和本地会话存储之间保持一致。

use super::stream::{StreamTurnMessageIds, persist_stream_chat_turn, split_stream_model_ref};

use crate::app::agent::project;
use crate::app::agent::session as agent_session;
use crate::session::ui_types as ui_models;

#[test]
fn split_stream_model_ref_uses_model_id_when_provider_is_missing() {
    let model_ref = split_stream_model_ref(Some("claude-sonnet-4"));

    assert_eq!(model_ref.provider_id, "");
    assert_eq!(model_ref.model_id, "claude-sonnet-4");
}

#[tokio::test]
async fn persist_stream_chat_turn_creates_user_and_assistant_messages() {
    let temp = tempfile::tempdir().expect("tempdir");
    let session_id = format!("ses_stream_test_{}", uuid::Uuid::new_v4());

    // 流式持久化依赖当前项目实例目录；测试用临时目录隔离真实用户数据。
    let (assistant_id, user_id, stored) =
        project::instance::provide(temp.path(), None, move || {
            Box::pin(async move {
                let usage = ui_models::TokenUsage {
                    input_tokens: 12,
                    output_tokens: 34,
                    cached_tokens: 5,
                    reasoning_tokens: 6,
                };
                let (assistant_id, user_id) = persist_stream_chat_turn(
                    &session_id,
                    "hello from user",
                    "hello from assistant",
                    Some("anthropic/claude-sonnet-4"),
                    &usage,
                    Some("stop"),
                    None,
                )
                .await
                .expect("stream turn should persist");
                let stored = agent_session::message::messages(&session_id, None)
                    .await
                    .expect("stored messages should load");
                Ok::<_, project::Error>((assistant_id, user_id, stored))
            })
        })
        .await
        .expect("instance context should be provided")
        .expect("stream turn should return stored messages");

    assert_eq!(stored.len(), 2);

    // 用户消息也携带模型引用，后续 UI 和审计视图需要用它重建上下文。
    let user_message = stored
        .iter()
        .find(|message| message.info.id() == user_id)
        .expect("user message should be present");
    match &user_message.info {
        agent_session::message::Info::User(info) => {
            assert_eq!(info.model.provider_id, "anthropic");
            assert_eq!(info.model.model_id, "claude-sonnet-4");
        }
        _ => panic!("expected stored user message"),
    }
    let user_text = user_message
        .parts
        .iter()
        .find_map(|part| match part {
            agent_session::message::Part::Text(text) => Some(text.text.as_str()),
            _ => None,
        })
        .expect("user text part should be present");
    assert_eq!(user_text, "hello from user");

    // 助手消息必须挂到用户消息下面，避免会话树在回放时丢失父子关系。
    let assistant_message = stored
        .iter()
        .find(|message| message.info.id() == assistant_id)
        .expect("assistant message should be present");
    match &assistant_message.info {
        agent_session::message::Info::Assistant(info) => {
            assert_eq!(info.parent_id, user_id);
            assert_eq!(info.provider_id, "anthropic");
            assert_eq!(info.model_id, "claude-sonnet-4");
            assert_eq!(info.finish.as_deref(), Some("stop"));
            assert_eq!(info.tokens.input, 12);
            assert_eq!(info.tokens.output, 34);
            assert_eq!(info.tokens.reasoning, 6);
            assert_eq!(info.tokens.cache.read, 5);
        }
        _ => panic!("expected stored assistant message"),
    }
    let assistant_text = assistant_message
        .parts
        .iter()
        .find_map(|part| match part {
            agent_session::message::Part::Text(text) => Some(text.text.as_str()),
            _ => None,
        })
        .expect("assistant text part should be present");
    assert_eq!(assistant_text, "hello from assistant");
}

#[tokio::test]
async fn persist_stream_chat_turn_reuses_preallocated_message_ids() {
    let temp = tempfile::tempdir().expect("tempdir");
    let session_id = "ses_stream_preallocated";
    // 预分配 id 来自流式响应开始阶段，落盘必须复用同一组 id 才能和前端事件对齐。
    let preallocated =
        StreamTurnMessageIds::new("msg_assistant_preallocated", "msg_user_preallocated");

    let (assistant_id, user_id, stored) =
        project::instance::provide(temp.path(), None, move || {
            let preallocated = preallocated.clone();
            Box::pin(async move {
                let usage = ui_models::TokenUsage::default();
                let (assistant_id, user_id) = persist_stream_chat_turn(
                    session_id,
                    "hello from user",
                    "hello from assistant",
                    Some("anthropic/claude-sonnet-4"),
                    &usage,
                    Some("stop"),
                    Some(&preallocated),
                )
                .await
                .expect("stream turn should persist");
                let stored = agent_session::message::messages(session_id, None)
                    .await
                    .expect("stored messages should load");
                Ok::<_, project::Error>((assistant_id, user_id, stored))
            })
        })
        .await
        .expect("instance context should be provided")
        .expect("stream turn should return stored messages");

    assert_eq!(assistant_id, "msg_assistant_preallocated");
    assert_eq!(user_id, "msg_user_preallocated");
    assert!(stored.iter().any(|message| message.info.id() == assistant_id));
    assert!(stored.iter().any(|message| message.info.id() == user_id));
}
