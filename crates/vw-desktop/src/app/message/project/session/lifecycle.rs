//! 处理项目会话生命周期，负责加载历史、重置视图和切换活动会话。
//!
//! 注释聚焦模块职责、消息边界和失败处理方式，帮助维护者在不改变逻辑的前提下理解代码。

use super::common::save_config_field_task;
use crate::app::message::project::ProjectMessage;
use crate::app::message::project::helpers::{
    find_session_project_path, now_ms, prepare_session_ui_chunks_task, prepare_session_ui_task,
};
use crate::app::{App, Message, models, state::ActiveSessionViewState};
use serde_json::{Map, Value};
use vw_shared::message::types as agent_message;

fn push_loaded_chat_message(
    chat: &mut Vec<models::ChatMessage>,
    message_ids: &mut Vec<Option<String>>,
    role: models::ChatRole,
    content: String,
    think_timing: Vec<models::ThinkTiming>,
    message_id: Option<String>,
) {
    if content.trim().is_empty() && think_timing.is_empty() {
        return;
    }

    chat.push(models::ChatMessage { role, content, think_timing });
    message_ids.push(message_id);
}

fn flush_loaded_text_message(
    chat: &mut Vec<models::ChatMessage>,
    message_ids: &mut Vec<Option<String>>,
    role: models::ChatRole,
    content: &mut String,
    think_timing: &mut Vec<models::ThinkTiming>,
    message_id: &Option<String>,
) {
    push_loaded_chat_message(
        chat,
        message_ids,
        role,
        std::mem::take(content),
        std::mem::take(think_timing),
        message_id.clone(),
    );
}

fn tool_input_text(input: &Map<String, Value>) -> Option<String> {
    if input.is_empty() {
        return None;
    }

    serde_json::to_string(input).ok().filter(|value| !value.trim().is_empty())
}

fn tool_error_is_denied(error: &str, metadata: Option<&Map<String, Value>>) -> bool {
    if metadata.is_some_and(|value| {
        value.contains_key("permission_request") || value.contains_key("permissionRequest")
    }) {
        return true;
    }

    let lowered = error.to_ascii_lowercase();
    lowered.contains("approval required")
        || lowered.contains("requires approval")
        || lowered.contains("denied")
        || lowered.contains("forbidden")
        || lowered.contains("not allowed")
        || lowered.contains("permission")
}

fn merged_tool_metadata(
    metadata: Option<&Map<String, Value>>,
    call_id: &str,
) -> Option<Map<String, Value>> {
    let mut merged = metadata.cloned().unwrap_or_default();
    if !merged.contains_key("callID")
        && !merged.contains_key("toolCallId")
        && !merged.contains_key("call_id")
        && !merged.contains_key("callId")
    {
        merged.insert("callID".to_string(), Value::String(call_id.to_string()));
    }

    if merged.is_empty() { None } else { Some(merged) }
}

fn lift_tool_metadata(payload: &mut Map<String, Value>, metadata: &Map<String, Value>) {
    if let Some(value) = metadata
        .get("permission_request")
        .cloned()
        .or_else(|| metadata.get("permissionRequest").cloned())
    {
        payload.entry("permission_request".to_string()).or_insert(value);
    }

    if let Some(value) = metadata.get("summary").cloned() {
        payload.entry("summary".to_string()).or_insert(value);
    }

    if let Some(value) =
        metadata.get("renderHint").cloned().or_else(|| metadata.get("render_hint").cloned())
    {
        payload.entry("renderHint".to_string()).or_insert(value);
    }
}

fn insert_string_field(payload: &mut Map<String, Value>, key: &str, value: Option<String>) {
    if let Some(value) = value.filter(|value| !value.trim().is_empty()) {
        payload.insert(key.to_string(), Value::String(value));
    }
}

fn insert_value_field(payload: &mut Map<String, Value>, key: &str, value: Option<Value>) {
    if let Some(value) = value {
        payload.insert(key.to_string(), value);
    }
}

fn tool_part_payload(part: &agent_message::ToolPart) -> Value {
    let mut payload = Map::new();
    payload.insert("callID".to_string(), Value::String(part.call_id.clone()));
    payload.insert("toolCallId".to_string(), Value::String(part.call_id.clone()));

    match &part.state {
        agent_message::ToolState::Pending(state) => {
            payload.insert("status".to_string(), Value::String("pending".to_string()));
            insert_string_field(
                &mut payload,
                "input",
                if state.raw.trim().is_empty() {
                    tool_input_text(&state.input)
                } else {
                    Some(state.raw.clone())
                },
            );
        }
        agent_message::ToolState::Running(state) => {
            payload.insert("status".to_string(), Value::String("running".to_string()));
            insert_string_field(&mut payload, "input", tool_input_text(&state.input));
            insert_string_field(&mut payload, "title", state.title.clone());
            insert_value_field(&mut payload, "time", serde_json::to_value(&state.time).ok());

            if let Some(metadata) = merged_tool_metadata(state.metadata.as_ref(), &part.call_id) {
                lift_tool_metadata(&mut payload, &metadata);
                payload.insert("metadata".to_string(), Value::Object(metadata));
            }
        }
        agent_message::ToolState::Completed(state) => {
            payload.insert("status".to_string(), Value::String("completed".to_string()));
            insert_string_field(&mut payload, "input", tool_input_text(&state.input));
            insert_string_field(&mut payload, "title", Some(state.title.clone()));
            insert_string_field(&mut payload, "output", Some(state.output.clone()));
            insert_value_field(&mut payload, "time", serde_json::to_value(&state.time).ok());
            insert_value_field(
                &mut payload,
                "attachments",
                state
                    .attachments
                    .as_ref()
                    .filter(|attachments| !attachments.is_empty())
                    .and_then(|attachments| serde_json::to_value(attachments).ok()),
            );

            if let Some(metadata) = merged_tool_metadata(Some(&state.metadata), &part.call_id) {
                lift_tool_metadata(&mut payload, &metadata);
                payload.insert("metadata".to_string(), Value::Object(metadata));
            }
        }
        agent_message::ToolState::Error(state) => {
            let status = if tool_error_is_denied(&state.error, state.metadata.as_ref()) {
                "denied"
            } else {
                "error"
            };
            payload.insert("status".to_string(), Value::String(status.to_string()));
            insert_string_field(&mut payload, "input", tool_input_text(&state.input));
            insert_string_field(&mut payload, "error", Some(state.error.clone()));
            insert_value_field(&mut payload, "time", serde_json::to_value(&state.time).ok());

            if let Some(metadata) = merged_tool_metadata(state.metadata.as_ref(), &part.call_id) {
                lift_tool_metadata(&mut payload, &metadata);
                payload.insert("metadata".to_string(), Value::Object(metadata));
            }
        }
    }

    Value::Object(payload)
}

fn tool_part_message_content(part: &agent_message::ToolPart) -> String {
    format!("tool {}\n{}\n", part.tool, tool_part_payload(part))
}

/// loaded_chat_from_gateway_messages 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 变更范围限制在当前消息处理路径内，不引入额外的流程分支。
pub(super) fn loaded_chat_from_gateway_messages(
    msgs: Vec<agent_message::WithParts>,
) -> (Vec<models::ChatMessage>, Vec<Option<String>>) {
    let mut chat = Vec::new();
    let mut message_ids = Vec::new();

    for message in msgs {
        let role = match &message.info {
            agent_message::Info::User(_) => models::ChatRole::User,
            agent_message::Info::Assistant(_) => models::ChatRole::Assistant,
        };
        let message_id = Some(message.info.id().to_string());
        let mut content = String::new();
        let mut think_timing = Vec::new();

        // 网关消息按片段到达，这里保持原顺序合并，避免工具卡片和正文错位。

        for part in message.parts {
            match part {
                agent_message::Part::Text(part) => content.push_str(&part.text),
                agent_message::Part::Reasoning(part) => {
                    content.push_str("<think>");
                    content.push_str(&part.text);
                    content.push_str("</think>");
                    think_timing.push(models::ThinkTiming {
                        start_ms: part.time.start,
                        end_ms: part.time.end,
                        last_update_ms: part.time.end.unwrap_or(part.time.start),
                    });
                }
                agent_message::Part::Tool(part) => {
                    flush_loaded_text_message(
                        &mut chat,
                        &mut message_ids,
                        role.clone(),
                        &mut content,
                        &mut think_timing,
                        &message_id,
                    );
                    push_loaded_chat_message(
                        &mut chat,
                        &mut message_ids,
                        models::ChatRole::Tool,
                        tool_part_message_content(&part),
                        Vec::new(),
                        message_id.clone(),
                    );
                }
                agent_message::Part::File(part) => {
                    content.push_str(&format!("\n[File: {}]\n", part.url));
                }
                _ => {}
            }
        }

        flush_loaded_text_message(
            &mut chat,
            &mut message_ids,
            role,
            &mut content,
            &mut think_timing,
            &message_id,
        );
    }

    (chat, message_ids)
}

/// handle 处理当前模块对应的消息或状态转换。
///
/// 参数由调用方提供应用状态、用户输入或后台任务结果；返回值会交给上层消息循环继续处理。
/// 失败会被转换为界面可展示的状态或消息，避免在处理链路中静默丢失。
pub(crate) fn handle(app: &mut App, message: ProjectMessage) -> Option<iced::Task<Message>> {
    // 所有界面事件在一个入口显式匹配，方便审计状态变更和异步任务边界。
    match message {
        ProjectMessage::SessionRightClicked(id, x, y) => {
            app.session_menu_id = Some(id);
            app.session_menu_anchor = Some(iced::Point::new(x, y));
            Some(iced::Task::none())
        }
        ProjectMessage::SessionMenuClose => {
            app.session_menu_id = None;
            app.session_menu_anchor = None;
            Some(iced::Task::none())
        }
        ProjectMessage::SessionTitleClicked(id) => {
            let now = web_time::Instant::now();
            let is_double_click = if let Some(last) = app.session_title_last_click {
                now.duration_since(last).as_millis() < 400
            } else {
                false
            };
            app.session_title_last_click = Some(now);
            if is_double_click {
                app.session_menu_id = None;
                app.session_menu_anchor = None;
                app.show_session_actions_popover = false;
                app.session_rename_id = Some(id.clone());
                if let Some(s) = app.sessions.iter().find(|s| s.id == id) {
                    app.session_rename_value = s.title.clone();
                }
            }
            Some(iced::Task::none())
        }
        ProjectMessage::SessionRenamePressed(id) => {
            app.session_menu_id = None;
            app.session_menu_anchor = None;
            app.show_session_actions_popover = false;
            app.session_rename_id = Some(id.clone());
            if let Some(s) = app.sessions.iter().find(|s| s.id == id) {
                app.session_rename_value = s.title.clone();
            } else if let Some(s) = app
                .project_sessions
                .values()
                .find_map(|sessions| sessions.iter().find(|s| s.id == id))
            {
                app.session_rename_value = s.title.clone();
            }
            Some(iced::Task::none())
        }
        ProjectMessage::SessionRenameChanged(v) => {
            app.session_rename_value = v;
            Some(iced::Task::none())
        }
        ProjectMessage::SessionRenameCancel => {
            app.session_rename_id = None;
            app.session_rename_value.clear();
            Some(iced::Task::none())
        }
        ProjectMessage::SessionRenameSave => {
            let Some(id) = app.session_rename_id.clone() else {
                return Some(iced::Task::none());
            };
            let title = app.session_rename_value.trim().to_string();
            if title.is_empty() {
                return Some(iced::Task::none());
            }

            app.session_rename_id = None;
            app.session_rename_value.clear();

            if let Some(s) = app.sessions.iter_mut().find(|s| s.id == id) {
                s.title = title.clone();
            }
            for sessions in app.project_sessions.values_mut() {
                if let Some(s) = sessions.iter_mut().find(|s| s.id == id) {
                    s.title = title.clone();
                }
            }
            app.refresh_task_pet_session_title(&id, &title);

            let project_path = find_session_project_path(app, &id);
            Some(iced::Task::perform(
                async move {
                    let client = crate::app::gateway_client().map_err(|err| err.to_string())?;
                    client
                        .session_update::<vw_shared::session::info::Info>(
                            &id,
                            project_path.as_deref(),
                            &vw_gateway_client::GatewaySessionPatchBody {
                                title: Some(title),
                                time: None,
                            },
                        )
                        .await
                },
                |_| Message::None,
            ))
        }
        ProjectMessage::SessionArchivePressed(id) => {
            app.session_menu_id = None;
            app.session_menu_anchor = None;
            app.show_session_actions_popover = false;

            if let Some(s) = app.sessions.iter_mut().find(|s| s.id == id) {
                s.time.archived = Some(now_ms());
            }
            for sessions in app.project_sessions.values_mut() {
                if let Some(s) = sessions.iter_mut().find(|s| s.id == id) {
                    s.time.archived = Some(now_ms());
                }
            }

            let mut switch_task = iced::Task::none();
            if app.active_session_id.as_ref() == Some(&id) {
                let next = app
                    .sessions
                    .iter()
                    .find(|s| s.time.archived.is_none() && s.id != id)
                    .map(|s| s.id.clone());

                if let Some(next_id) = next {
                    app.active_session_id = Some(next_id.clone());
                    app.mark_active_session_viewed();
                    app.sync_active_session_preferences();
                    switch_task = iced::Task::perform(
                        async move {
                            let client = match crate::app::gateway_client() {
                                Ok(client) => client,
                                Err(err) => return Err(err),
                            };
                            let msgs = client
                                .session_messages::<Vec<agent_message::WithParts>>(&next_id, None)
                                .await;
                            let info = client
                                .session_get::<vw_shared::session::info::Info>(&next_id, None)
                                .await;
                            match (msgs, info) {
                                (Ok(msgs), Ok(_info)) => {
                                    let mut usage = models::TokenUsage::default();
                                    for m in &msgs {
                                        if let agent_message::Info::Assistant(a) = &m.info {
                                            usage.input_tokens += a.tokens.input;
                                            usage.output_tokens += a.tokens.output;
                                            usage.cached_tokens +=
                                                a.tokens.cache.read + a.tokens.cache.write;
                                            usage.reasoning_tokens += a.tokens.reasoning;
                                        }
                                    }
                                    Ok((next_id, msgs, usage))
                                }
                                (Err(e), _) => Err(e.to_string()),
                                (_, Err(e)) => Err(e.to_string()),
                            }
                        },
                        |res| Message::Project(ProjectMessage::SessionMessagesLoaded(res)),
                    );
                } else {
                    app.active_session_id = None;
                    app.chat.clear();
                    app.usage = models::TokenUsage::default();
                    app.active_session_view_state = ActiveSessionViewState::default();
                    app.invalidate_chat_ui_state();
                }
            }

            let project_path = find_session_project_path(app, &id);
            Some(iced::Task::batch(vec![
                iced::Task::perform(
                    async move {
                        let client = match crate::app::gateway_client() {
                            Ok(client) => client,
                            Err(err) => return Err(err.to_string()),
                        };
                        client
                            .session_update::<vw_shared::session::info::Info>(
                                &id,
                                project_path.as_deref(),
                                &vw_gateway_client::GatewaySessionPatchBody {
                                    title: None,
                                    time: Some(vw_gateway_client::GatewaySessionPatchTime {
                                        archived: Some(vw_shared::time::now_ms()),
                                    }),
                                },
                            )
                            .await
                            .map_err(|err| err.to_string())
                    },
                    |_| Message::None,
                ),
                switch_task,
            ]))
        }
        ProjectMessage::SessionDeletePressed(id) => {
            app.session_menu_id = None;
            app.session_menu_anchor = None;
            app.show_session_actions_popover = false;
            app.session_runtime_states.remove(&id);

            let project_path = find_session_project_path(app, &id);

            if let Some(pos) = app.sessions.iter().position(|s| s.id == id) {
                app.sessions.remove(pos);
            }
            for sessions in app.project_sessions.values_mut() {
                if let Some(pos) = sessions.iter().position(|s| s.id == id) {
                    sessions.remove(pos);
                }
            }

            let mut switch_task = iced::Task::none();

            if app.active_session_id.as_ref() == Some(&id) {
                let next =
                    app.sessions.iter().find(|s| s.time.archived.is_none()).map(|s| s.id.clone());

                if let Some(next_id) = next {
                    app.active_session_id = Some(next_id.clone());
                    app.mark_active_session_viewed();
                    app.sync_active_session_preferences();
                    switch_task = iced::Task::perform(
                        async move {
                            let client = match crate::app::gateway_client() {
                                Ok(client) => client,
                                Err(err) => return Err(err),
                            };
                            let msgs = client
                                .session_messages::<Vec<agent_message::WithParts>>(&next_id, None)
                                .await;
                            match msgs {
                                Ok(msgs) => {
                                    let mut usage = models::TokenUsage::default();
                                    for m in &msgs {
                                        if let agent_message::Info::Assistant(a) = &m.info {
                                            usage.input_tokens += a.tokens.input;
                                            usage.output_tokens += a.tokens.output;
                                            usage.cached_tokens +=
                                                a.tokens.cache.read + a.tokens.cache.write;
                                            usage.reasoning_tokens += a.tokens.reasoning;
                                        }
                                    }
                                    Ok((next_id, msgs, usage))
                                }
                                Err(e) => Err(e.to_string()),
                            }
                        },
                        |res| Message::Project(ProjectMessage::SessionMessagesLoaded(res)),
                    );
                } else {
                    app.active_session_id = None;
                    app.chat.clear();
                    app.usage = models::TokenUsage::default();
                    app.active_session_view_state = ActiveSessionViewState::default();
                    app.invalidate_chat_ui_state();
                }
            }

            Some(iced::Task::batch(vec![
                iced::Task::perform(
                    async move {
                        let client = crate::app::gateway_client().map_err(|err| err.to_string())?;
                        client.session_delete(&id, project_path.as_deref()).await
                    },
                    |_| Message::None,
                ),
                switch_task,
            ]))
        }
        ProjectMessage::SessionCopyPressed(id) => {
            let project_path = find_session_project_path(app, &id);
            Some(iced::Task::perform(
                async move {
                    let client = crate::app::gateway_client().map_err(|err| err.to_string())?;
                    client
                        .session_fork::<vw_shared::session::info::Info>(
                            &id,
                            project_path.as_deref(),
                            &None,
                        )
                        .await
                },
                |res| match res {
                    Ok(info) => Message::Project(ProjectMessage::SessionCopied(info)),
                    Err(e) => {
                        eprintln!("Fork failed: {}", e);
                        Message::None
                    }
                },
            ))
        }
        ProjectMessage::SessionCopied(info) => {
            if app.project_path.as_ref() == Some(&info.directory)
                && !app.sessions.iter().any(|s| s.id == info.id)
            {
                app.sessions.insert(0, info.clone());
            }
            let sessions_key = app.project_path.clone().unwrap_or_else(|| info.directory.clone());
            if let Some(list) = app.project_sessions.get_mut(&sessions_key) {
                if !list.iter().any(|s| s.id == info.id) {
                    list.insert(0, info.clone());
                }
            } else {
                app.project_sessions.insert(sessions_key, vec![info.clone()]);
            }
            Some(iced::Task::none())
        }
        ProjectMessage::SessionCreated(info) => {
            app.new_session_last_directory = Some(info.directory.clone());
            let save_last_directory_task = save_config_field_task(
                "new_session_last_directory",
                serde_json::Value::String(info.directory.clone()),
            );
            app.cache_active_session_chat();
            if !app.sessions.iter().any(|s| s.id == info.id) {
                app.sessions.insert(0, info.clone());
            }
            let sessions_key = app.project_path.clone().unwrap_or_else(|| info.directory.clone());
            if let Some(list) = app.project_sessions.get_mut(&sessions_key) {
                if !list.iter().any(|s| s.id == info.id) {
                    list.insert(0, info.clone());
                }
            } else {
                app.project_sessions.insert(sessions_key, vec![info.clone()]);
            }
            app.active_session_id = Some(info.id.clone());
            app.mark_active_session_viewed();
            app.chat.clear();
            app.chat_message_ids.clear();
            app.store_session_chat_snapshot(
                info.id.clone(),
                crate::app::session::shared_chat_messages(app.chat.clone()),
                app.chat_message_ids.clone(),
            );
            app.usage = models::TokenUsage::default();
            app.active_session_view_state = ActiveSessionViewState::default();
            app.invalidate_chat_ui_state();
            app.sync_active_session_preferences();
            Some(save_last_directory_task)
        }
        ProjectMessage::SessionMessagesLoaded(res) => {
            match res {
                Ok((id, msgs, usage)) => {
                    tracing::info!(
                        target: "vw_desktop",
                        session_id = %id,
                        message_count = msgs.len(),
                        active_session_matches = app.active_session_id.as_ref() == Some(&id),
                        "received session messages load result"
                    );
                    if app.active_session_id.as_ref() == Some(&id) {
                        let local_messages = app
                            .cached_chat_messages(&id)
                            .map(|chat| chat.iter().cloned().collect::<Vec<_>>());
                        let local_message_ids = app.cached_chat_message_ids(&id);
                        let (next_chat, next_ids) = if msgs.is_empty()
                            && let Some(local) = local_messages.clone()
                            && !local.is_empty()
                        {
                            let ids = local_message_ids.unwrap_or_else(|| vec![None; local.len()]);
                            (local, ids)
                        } else {
                            let (converted, converted_ids) =
                                loaded_chat_from_gateway_messages(msgs);

                            if !converted.is_empty() {
                                (converted, converted_ids)
                            } else if let Some(local) = local_messages
                                && !local.is_empty()
                            {
                                let ids =
                                    local_message_ids.unwrap_or_else(|| vec![None; local.len()]);
                                (local, ids)
                            } else {
                                (Vec::new(), Vec::new())
                            }
                        };

                        let shared_chat = crate::app::session::shared_chat_messages(next_chat);
                        app.chat = shared_chat.iter().cloned().collect();
                        app.chat_message_ids = if app.chat.len() == next_ids.len() {
                            next_ids
                        } else {
                            vec![None; app.chat.len()]
                        };
                        app.store_session_chat_snapshot(
                            id.clone(),
                            shared_chat.clone(),
                            app.chat_message_ids.clone(),
                        );
                        app.usage = usage;
                        app.active_session_view_state.updated_ms = 0;
                        app.clear_active_session_steps();
                        app.invalidate_chat_ui_state();
                        if app.chat.is_empty() {
                            tracing::warn!(
                                target: "vw_desktop",
                                session_id = %id,
                                "session message load completed but chat remained empty"
                            );
                        } else {
                            tracing::info!(
                                target: "vw_desktop",
                                session_id = %id,
                                chat_messages = app.chat.len(),
                                "applied loaded session messages to active chat"
                            );
                        }

                        if app.chat.is_empty() {
                            app.active_session_view_state.ui_preparing = false;
                            app.active_session_view_state.base_ready = true;
                            app.pin_chat_ui_chunk(None);
                        } else {
                            app.active_session_view_state.ui_preparing = true;
                            app.active_session_view_state.base_ready = false;
                            let base_chunk_start = app.preferred_base_chat_ui_chunk_start();
                            app.pin_chat_ui_chunk(Some(base_chunk_start));
                            app.mark_chat_ui_chunks_preparing(&[base_chunk_start]);
                            return Some(prepare_session_ui_task(
                                id,
                                shared_chat,
                                base_chunk_start,
                                true,
                                app.dialogue_flow_show_reasoning_summary,
                            ));
                        }
                    } else {
                        tracing::warn!(
                            target: "vw_desktop",
                            session_id = %id,
                            active_session_id = app.active_session_id.as_deref().unwrap_or("<none>"),
                            "discarded session messages because active session changed"
                        );
                    }
                }
                Err(e) => {
                    tracing::warn!(
                        target: "vw_desktop",
                        active_session_id = app.active_session_id.as_deref().unwrap_or("<none>"),
                        error = %e,
                        "failed to load session messages"
                    );
                    if let Some(id) = app.active_session_id.clone()
                        && let Some(local) = app.cached_chat_messages(&id)
                        && !local.is_empty()
                    {
                        let local_messages = local.iter().cloned().collect::<Vec<_>>();
                        let local_message_ids = if let Some(ids) = app.cached_chat_message_ids(&id)
                            && ids.len() == local_messages.len()
                        {
                            ids
                        } else {
                            vec![None; local_messages.len()]
                        };
                        let shared_chat = crate::app::session::shared_chat_messages(local_messages);
                        app.chat = shared_chat.iter().cloned().collect();
                        app.chat_message_ids = local_message_ids;
                        app.store_session_chat_snapshot(
                            id.clone(),
                            shared_chat.clone(),
                            app.chat_message_ids.clone(),
                        );
                        app.clear_active_session_steps();
                        app.active_session_view_state.ui_preparing = true;
                        app.active_session_view_state.base_ready = false;
                        app.invalidate_chat_ui_state();
                        let base_chunk_start = app.preferred_base_chat_ui_chunk_start();
                        app.pin_chat_ui_chunk(Some(base_chunk_start));
                        app.mark_chat_ui_chunks_preparing(&[base_chunk_start]);
                        return Some(prepare_session_ui_task(
                            id,
                            shared_chat,
                            base_chunk_start,
                            true,
                            app.dialogue_flow_show_reasoning_summary,
                        ));
                    } else {
                        app.error_message = Some(format!("Failed to load session messages: {}", e));
                    }
                }
            }
            Some(iced::Task::none())
        }
        ProjectMessage::SessionUiPrepared { session_id, phase } => {
            if app.active_session_id.as_ref() == Some(&session_id) {
                let is_base_phase =
                    matches!(phase, crate::app::session::PreparedChatUiPhase::Base(_));
                app.apply_prepared_chat_ui_phase(phase);
                if is_base_phase {
                    app.active_session_view_state.base_ready = true;
                    app.rebuild_active_session_message_meta();
                    app.active_session_view_state.ui_preparing = false;
                    let (visible_start_idx, visible_end_idx) = app.visible_chat_message_window();
                    let nearby_chunk_starts =
                        app.pending_chat_ui_chunk_starts(visible_start_idx, visible_end_idx, true);
                    if !nearby_chunk_starts.is_empty() {
                        app.mark_chat_ui_chunks_preparing(&nearby_chunk_starts);
                        return Some(prepare_session_ui_chunks_task(
                            session_id,
                            app.active_shared_chat_messages(),
                            nearby_chunk_starts,
                            None,
                            app.dialogue_flow_show_reasoning_summary,
                        ));
                    }
                }
            }
            Some(iced::Task::none())
        }
        _ => None,
    }
}
