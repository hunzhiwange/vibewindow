//! 处理聊天流式会话事件。
//! 本模块把网关轮询和流式增量落到会话状态，避免 UI 层理解传输细节。

use super::{
    load_session_or_default, now_ms, save_session_task, session_directory_for_save, start_next,
};
use crate::app::message::ProjectMessage;
use crate::app::message::chat::scroll_chat_to_bottom_task;
use crate::app::message::chat::session as chat_session;
use crate::app::state::ChatSendBehavior;
use crate::app::ui::chat;
use crate::app::{App, Message, QueueItem, models};
use iced::Task;
use tracing::info;
use vw_shared::message::types as message;
use vw_shared::session::info as session_info;

fn update_think_timing_from_delta(message: &mut models::ChatMessage, delta: &str, now: u64) {
    if delta.is_empty() {
        return;
    }
    let mut combined = String::with_capacity(message.content.len() + delta.len());
    combined.push_str(&message.content);
    combined.push_str(delta);

    let (thinks, _visible, thinking_open) = chat::split_think(&combined);
    let think_count = thinks.len();

    if message.think_timing.len() < think_count {
        for _ in message.think_timing.len()..think_count {
            message.think_timing.push(models::ThinkTiming {
                start_ms: now,
                end_ms: None,
                last_update_ms: now,
            });
        }
    }

    if think_count == 0 {
        return;
    }

    if thinking_open {
        let last_index = message.think_timing.len().saturating_sub(1);
        for (idx, entry) in message.think_timing.iter_mut().enumerate() {
            if idx == last_index {
                if entry.end_ms.is_none() {
                    entry.last_update_ms = now;
                }
            } else if entry.end_ms.is_none() {
                entry.end_ms = Some(now);
                entry.last_update_ms = now;
            }
        }
    } else {
        for entry in message.think_timing.iter_mut() {
            if entry.end_ms.is_none() {
                entry.end_ms = Some(now);
                entry.last_update_ms = now;
            }
        }
    }
}

fn apply_stream_done_message_ids(
    chat: &[models::ChatMessage],
    message_ids: &mut Vec<Option<String>>,
    assistant_message_id: Option<String>,
    parent_message_id: Option<String>,
) {
    if message_ids.len() < chat.len() {
        message_ids.resize(chat.len(), None);
    }

    let assistant_idx =
        chat.iter().rposition(|message| matches!(message.role, models::ChatRole::Assistant));
    if let Some(idx) = assistant_idx
        && let Some(message_id) = assistant_message_id
    {
        message_ids[idx] = Some(message_id);
    }

    let user_search_end = assistant_idx.unwrap_or(chat.len());
    let user_idx = chat[..user_search_end]
        .iter()
        .rposition(|message| matches!(message.role, models::ChatRole::User));
    if let Some(idx) = user_idx
        && let Some(message_id) = parent_message_id
    {
        message_ids[idx] = Some(message_id);
    }
}

fn message_ids_tail(message_ids: &[Option<String>]) -> Vec<Option<String>> {
    let tail_len = message_ids.len().min(4);
    message_ids[message_ids.len().saturating_sub(tail_len)..].to_vec()
}

fn leading_guide_count(queue: &[QueueItem]) -> usize {
    queue.iter().take_while(|item| item.send_behavior == ChatSendBehavior::Guide).count()
}

fn continuation_label_from_history(history: &[models::ChatMessage]) -> String {
    history
        .iter()
        .rev()
        .find(|message| matches!(message.role, models::ChatRole::User))
        .map(|message| message.content.clone())
        .unwrap_or_default()
}

/// 模块内可见函数，执行 handle_agent_stream_delta 对应的应用流程。
/// 返回值表达处理结果；失败通过错误值、日志或任务消息显式传递。
pub(super) fn handle_agent_stream_delta(app: &mut App, id: u64, delta: String) -> Task<Message> {
    let Some(session_id) = app.find_session_by_request_id(id) else {
        return Task::none();
    };

    let now = now_ms();
    let is_active = app.active_session_id.as_ref() == Some(&session_id);
    let mut live_chat: Vec<models::ChatMessage> = if is_active {
        app.chat.clone()
    } else if let Some(cached) = app.session_chat_cache.get(&session_id).cloned() {
        cached.iter().cloned().collect::<Vec<models::ChatMessage>>()
    } else {
        load_session_or_default(app, session_id.clone()).messages
    };
    let mut live_ids = if is_active {
        app.chat_message_ids.clone()
    } else {
        app.session_chat_message_id_cache
            .get(&session_id)
            .cloned()
            .unwrap_or_else(|| vec![None; live_chat.len()])
    };

    if let Some(last) = live_chat.last_mut()
        && last.role == models::ChatRole::Assistant
    {
        update_think_timing_from_delta(last, &delta, now);
        last.content.push_str(&delta);
    } else {
        live_chat.push(models::ChatMessage {
            role: models::ChatRole::Assistant,
            content: delta.clone(),
            think_timing: Vec::new(),
        });
        live_ids.push(None);
        if let Some(last) = live_chat.last_mut() {
            update_think_timing_from_delta(last, &delta, now);
        }
    }

    app.store_session_chat_snapshot(
        session_id.clone(),
        crate::app::session::shared_chat_messages(live_chat.clone()),
        live_ids.clone(),
    );

    let mut session = load_session_or_default(app, session_id.clone());
    session.messages = live_chat.clone();
    session.updated_ms = now;
    let save_task = save_session_task(session, session_directory_for_save(app, &session_id));

    if !is_active {
        app.sync_task_pet_from_runtime();
        return save_task;
    }

    app.chat = live_chat;
    app.chat_message_ids = live_ids;
    app.sync_task_pet_from_runtime();
    app.sync_chat_message_estimated_heights_len();
    let mut tail_prewarm_task = Task::none();
    let mut should_snap_to_bottom = false;
    if let Some(last_idx) = app.chat.len().checked_sub(1) {
        app.refine_chat_message_estimated_heights(last_idx, last_idx + 1);
        let tail_chunk_start = crate::app::session::chat_ui_chunk_start_idx(last_idx);
        let tail_is_visible = {
            let (visible_start_idx, visible_end_idx) = app.visible_chat_message_window();
            last_idx >= visible_start_idx && last_idx < visible_end_idx
        };
        should_snap_to_bottom = app.chat_auto_scroll && !tail_is_visible;
        if (app.chat_auto_scroll || tail_is_visible)
            && !app.active_session_view_state.preparing_chat_ui_chunks.contains(&tail_chunk_start)
        {
            app.mark_chat_ui_chunks_preparing(&[tail_chunk_start]);
            tail_prewarm_task = crate::app::message::project::prepare_session_ui_task(
                session_id.clone(),
                app.active_shared_chat_messages(),
                tail_chunk_start,
                false,
            );
        }
    }
    app.active_session_view_state.updated_ms = now;
    app.rebuild_active_session_message_meta();
    if should_snap_to_bottom {
        Task::batch(vec![scroll_chat_to_bottom_task(app), tail_prewarm_task, save_task])
    } else {
        Task::batch(vec![tail_prewarm_task, save_task])
    }
}

/// 模块内可见函数，执行 handle_agent_stream_done 对应的应用流程。
/// 返回值表达处理结果；失败通过错误值、日志或任务消息显式传递。
pub(super) fn handle_agent_stream_done(
    app: &mut App,
    id: u64,
    usage: models::TokenUsage,
    assistant_message_id: Option<String>,
    parent_message_id: Option<String>,
) -> Task<Message> {
    let Some(session_id) = app.find_session_by_request_id(id) else {
        return Task::none();
    };
    crate::app::state::clear_pending_guide_handoff(id);
    let is_active_session = app.active_session_id.as_ref() == Some(&session_id);
    let has_pending_queue = !app.get_session_runtime(&session_id).queue.is_empty();
    let resume_history_only = app
        .get_session_runtime(&session_id)
        .active_agent_request
        .as_ref()
        .map(|request| request.resume_history_only)
        .unwrap_or(false);

    let persisted_chat: Vec<models::ChatMessage> = if is_active_session {
        app.chat.clone()
    } else if let Some(cached) = app.session_chat_cache.get(&session_id).cloned() {
        cached.iter().cloned().collect::<Vec<models::ChatMessage>>()
    } else {
        load_session_or_default(app, session_id.clone()).messages
    };
    let mut persisted_ids = if is_active_session {
        app.chat_message_ids.clone()
    } else {
        app.session_chat_message_id_cache
            .get(&session_id)
            .cloned()
            .unwrap_or_else(|| vec![None; persisted_chat.len()])
    };
    apply_stream_done_message_ids(
        &persisted_chat,
        &mut persisted_ids,
        assistant_message_id.clone(),
        parent_message_id.clone(),
    );
    info!(
        target: "vw_desktop",
        request_id = id,
        session_id = %session_id,
        is_active_session,
        assistant_message_id = ?assistant_message_id,
        parent_message_id = ?parent_message_id,
        message_ids_tail = ?message_ids_tail(&persisted_ids),
        "desktop applied stream message ids"
    );

    if is_active_session {
        app.chat_message_ids = persisted_ids.clone();
        app.usage = usage;
        app.last_call_log_path = None;
    }

    let mut local_session = load_session_or_default(app, session_id.clone());
    local_session.messages = persisted_chat.clone();
    local_session.message_ids = persisted_ids.clone();
    local_session.updated_ms = now_ms();
    let save_task = save_session_task(local_session, session_directory_for_save(app, &session_id));
    app.store_session_chat_snapshot(
        session_id.clone(),
        crate::app::session::shared_chat_messages(persisted_chat),
        persisted_ids,
    );
    info!(
        target: "vw_desktop",
        request_id = id,
        session_id = %session_id,
        cached_message_ids_tail = ?app
            .session_chat_message_id_cache
            .get(&session_id)
            .map(|ids| message_ids_tail(ids))
            .unwrap_or_default(),
        "desktop persisted stream message ids"
    );

    {
        let runtime = app.get_session_runtime_mut(&session_id);
        runtime.is_requesting = false;
        runtime.has_unseen_success = !is_active_session;
        runtime.active_agent_request = None;
    }
    app.sync_task_pet_from_runtime();
    app.show_send_mode_popover = false;

    if is_active_session {
        app.sync_active_session_from_chat();
    }

    let next = start_next(app, &session_id);
    let refresh_message_ids = if is_active_session && !has_pending_queue && !resume_history_only {
        app.sessions
            .iter()
            .find(|entry| entry.id == session_id)
            .map(|entry| entry.directory.clone())
            .or_else(|| app.project_path.clone())
            .map(|project_path| {
                let session_id = session_id.clone();
                Task::perform(
                    async move {
                        let client = match crate::app::gateway_client() {
                            Ok(client) => client,
                            Err(err) => return Err(err),
                        };
                        let msgs = client
                            .session_messages::<Vec<message::WithParts>>(
                                &session_id,
                                Some(&project_path),
                            )
                            .await;
                        let info = client
                            .session_get::<session_info::Info>(&session_id, Some(&project_path))
                            .await;
                        match (msgs, info) {
                            (Ok(msgs), Ok(_)) => {
                                let mut usage = crate::app::models::TokenUsage::default();
                                for message in &msgs {
                                    if let message::Info::Assistant(assistant) = &message.info {
                                        usage.input_tokens += assistant.tokens.input;
                                        usage.output_tokens += assistant.tokens.output;
                                        usage.cached_tokens += assistant.tokens.cache.read
                                            + assistant.tokens.cache.write;
                                        usage.reasoning_tokens += assistant.tokens.reasoning;
                                    }
                                }
                                Ok((session_id, msgs, usage))
                            }
                            (Err(err), _) => Err(err),
                            (_, Err(err)) => Err(err),
                        }
                    },
                    |res| Message::Project(ProjectMessage::SessionMessagesLoaded(res)),
                )
            })
            .unwrap_or_else(Task::none)
    } else {
        Task::none()
    };
    let session_todos_task = if is_active_session {
        super::todos::load_input_panel_todos_task(session_id.clone())
    } else {
        Task::none()
    };

    if is_active_session && app.chat_auto_scroll {
        scroll_chat_to_bottom_task(app)
    } else {
        Task::none()
    }
    .chain(session_todos_task)
    .chain(refresh_message_ids)
    .chain(save_task)
    .chain(next)
}

/// 模块内可见函数，执行 handle_agent_post_tool_round 对应的应用流程。
/// 返回值表达处理结果；失败通过错误值、日志或任务消息显式传递。
pub(super) fn handle_agent_post_tool_round(
    app: &mut App,
    id: u64,
    session_id: String,
    step_index: u32,
) -> Task<Message> {
    let Some(found_session_id) = app.find_session_by_request_id(id) else {
        return Task::none();
    };
    if found_session_id != session_id || app.active_session_id.as_ref() != Some(&session_id) {
        return Task::none();
    }

    let runtime = app.get_session_runtime(&session_id);
    let Some(active_request) = runtime.active_agent_request.clone() else {
        return Task::none();
    };
    let insert_idx = leading_guide_count(&runtime.queue);
    let resume_history = app.chat.clone();
    let continuation = QueueItem {
        created_ms: now_ms(),
        query: continuation_label_from_history(&resume_history),
        attachments: Vec::new(),
        root: active_request.root.clone(),
        model: active_request.model.clone(),
        acp_test: active_request.acp_test,
        acp_agent: active_request.acp_agent.clone(),
        agent: active_request.agent.clone(),
        allowed_tools: active_request.allowed_tools.clone(),
        acp_force_new_session: false,
        acp_history_mode: active_request.acp_history_mode,
        acp_recent_count: active_request.acp_recent_count,
        full_access_enabled: active_request.full_access_enabled,
        send_behavior: ChatSendBehavior::Queue,
        request_history_override: Some(resume_history),
        resume_history_only: true,
    };
    let next_item = {
        let runtime = app.get_session_runtime_mut(&session_id);
        runtime.is_requesting = false;
        runtime.submit_anim = 0;
        runtime.has_unseen_success = false;
        runtime.active_agent_request = None;
        runtime.queue.insert(insert_idx, continuation);
        runtime.queue.remove(0)
    };

    app.sync_task_pet_from_runtime();
    app.show_send_mode_popover = false;
    app.sync_active_session_from_chat();
    info!(
        target: "vw_desktop",
        request_id = id,
        session_id = %session_id,
        step_index,
        queued_after_handoff = app.get_session_runtime(&session_id).queue.len(),
        "desktop yielded active request after tool round to prioritize guided queue item"
    );

    chat_session::start(app, next_item, true)
}

/// 模块内可见函数，执行 handle_agent_stream_error 对应的应用流程。
/// 返回值表达处理结果；失败通过错误值、日志或任务消息显式传递。
pub(super) fn handle_agent_stream_error(app: &mut App, id: u64, err: String) -> Task<Message> {
    let Some(session_id) = app.find_session_by_request_id(id) else {
        return Task::none();
    };
    crate::app::state::clear_pending_guide_handoff(id);

    {
        let runtime = app.get_session_runtime_mut(&session_id);
        runtime.is_requesting = false;
        runtime.has_unseen_success = false;
        runtime.active_agent_request = None;
    }
    app.sync_task_pet_from_runtime();
    app.show_send_mode_popover = false;

    if app.active_session_id.as_ref() == Some(&session_id) {
        app.chat.push(models::ChatMessage {
            role: models::ChatRole::System,
            content: err,
            think_timing: Vec::new(),
        });
        app.chat_message_ids.push(None);
        app.store_session_chat_snapshot(
            session_id.clone(),
            crate::app::session::shared_chat_messages(app.chat.clone()),
            app.chat_message_ids.clone(),
        );
        app.session_chat_message_id_cache.insert(session_id.clone(), app.chat_message_ids.clone());
        app.sync_active_session_from_chat();
    }

    let next = start_next(app, &session_id);

    if app.active_session_id.as_ref() == Some(&session_id) && app.chat_auto_scroll {
        scroll_chat_to_bottom_task(app).chain(next)
    } else {
        next
    }
}
#[cfg(test)]
#[path = "stream_events_tests.rs"]
mod stream_events_tests;
