//! 处理聊天会话相关应用消息。
//! 本模块协调用户输入、会话缓存和网关持久化。

use super::ChatMessage;
#[cfg(not(target_arch = "wasm32"))]
use crate::app::config::{save_project_chat_preferences, set_config_field};
use crate::app::message::TaskBoardMessage;
use crate::app::message::chat::stream;
use crate::app::message::project::ProjectMessage;
use crate::app::{AgentRequest, App, Message, QueueItem, message, models};
use crate::app::state::{ChatSendBehavior, MAIN_AGENT_KEY, SessionRuntimeState};
use iced::Task;
use std::path::Path;
use std::time::Duration;
use tracing::{debug, info};
use vw_shared::id;
use vw_shared::session::info as session;
use vw_shared::session::session_utils::create_slug;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InlineModeCommand {
    TaskMode,
    ChatMode,
}

fn parse_inline_mode_command(query: &str) -> Option<InlineModeCommand> {
    let command = query.split_whitespace().next()?.to_ascii_lowercase();
    match command.as_str() {
        "/task" => Some(InlineModeCommand::TaskMode),
        "/new" | "/clear" | "/session" => Some(InlineModeCommand::ChatMode),
        _ => None,
    }
}

fn apply_inline_mode_command(app: &mut App, command: InlineModeCommand) -> Task<Message> {
    let runtime_editor = {
        let runtime = app.current_session_runtime_mut();
        runtime.task_mode_enabled = command == InlineModeCommand::TaskMode;
        runtime.input_editor = iced::widget::text_editor::Content::new();
        runtime.input_editor.clone()
    };
    if app.active_session_id.is_none() {
        app.input_editor = runtime_editor;
    }
    Task::none()
}

fn gateway_endpoint(app: &App) -> (String, u16) {
    let host = app.gateway_settings.host_input.trim();
    let host = if host.is_empty() { "127.0.0.1" } else { host };
    (host.to_string(), app.gateway_settings.port)
}

fn current_session_directory(app: &App) -> Option<String> {
    app.active_session_id
        .as_ref()
        .and_then(|id| app.sessions.iter().find(|s| &s.id == id).map(|s| s.directory.clone()))
}

fn default_acp_agent(app: &App) -> Option<String> {
    app.acp_agent.clone()
}

fn resolve_acp_agent(app: &App, selected: Option<String>) -> Option<String> {
    selected.or_else(|| default_acp_agent(app))
}

fn normalize_delegate_agent(selected: Option<String>) -> Option<String> {
    selected.and_then(|value| {
        let trimmed = value.trim();
        (!trimmed.is_empty() && trimmed != MAIN_AGENT_KEY).then(|| trimmed.to_string())
    })
}

fn request_delegate_agent(selected: Option<String>) -> Option<String> {
    normalize_delegate_agent(selected).or_else(|| Some(MAIN_AGENT_KEY.to_string()))
}

fn selected_delegate_allowed_tools(
    app: &App,
    runtime: &SessionRuntimeState,
) -> Option<Vec<String>> {
    let tool_inventory = app.session_tool_inventory(runtime);
    if tool_inventory.effective_tools.is_empty() {
        None
    } else if tool_inventory.static_filtered
        || !runtime.tool_selector.is_all_enabled()
        || tool_inventory.effective_tools.len() != tool_inventory.base_tools.len()
    {
        Some(tool_inventory.effective_tools)
    } else {
        None
    }
}

fn is_supported_image_attachment(path: &str) -> bool {
    Path::new(path)
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| {
            matches!(
                ext.to_ascii_lowercase().as_str(),
                "png" | "jpg" | "jpeg" | "webp" | "gif" | "bmp"
            )
        })
        .unwrap_or(false)
}

fn compose_query_with_attachments(query: &str, attachments: &[String]) -> String {
    let trimmed = query.trim();
    let markers = attachments
        .iter()
        .map(|path| {
            if is_supported_image_attachment(path) {
                format!("[IMAGE:{path}]")
            } else {
                format!("[DOCUMENT:{path}]")
            }
        })
        .collect::<Vec<_>>()
        .join("\n");

    match (trimmed.is_empty(), markers.is_empty()) {
        (true, true) => String::new(),
        (false, true) => trimmed.to_string(),
        (true, false) => markers,
        (false, false) => format!("{trimmed}\n\n{markers}"),
    }
}

fn persist_acp_history_preferences(
    app: &mut App,
    mode: crate::app::state::AcpHistoryReplayMode,
    recent_count: usize,
) -> Task<Message> {
    app.acp_history_mode = mode;
    app.acp_recent_count = recent_count.clamp(1, 20);

    #[cfg(target_arch = "wasm32")]
    {
        return Task::batch([
            save_config_field_task(
                "acp_history_strategy",
                serde_json::Value::String(mode.as_str().to_string()),
            ),
            save_config_field_task(
                "acp_history_recent_count",
                serde_json::Value::Number((app.acp_recent_count as u64).into()),
            ),
        ]);
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        set_config_field(
            "acp_history_strategy",
            serde_json::Value::String(mode.as_str().to_string()),
        );
        set_config_field(
            "acp_history_recent_count",
            serde_json::Value::Number((app.acp_recent_count as u64).into()),
        );
        Task::none()
    }
}

fn persist_chat_preferences(
    app: &App,
    model: &str,
    auto_model: bool,
    acp_agent: Option<&str>,
) -> Task<Message> {
    #[cfg(target_arch = "wasm32")]
    {
        let mut tasks = vec![
            save_config_field_task("model", serde_json::Value::String(model.to_string())),
            save_config_field_task("auto_model", serde_json::Value::Bool(auto_model)),
            save_config_field_task(
                "acp_agent",
                acp_agent
                    .map(|value| serde_json::Value::String(value.to_string()))
                    .unwrap_or(serde_json::Value::Null),
            ),
        ];
        if let Some(project_path) = app.project_path.clone() {
            tasks.push(save_project_chat_preferences_task(
                project_path,
                model.to_string(),
                auto_model,
                acp_agent.map(str::to_string),
            ));
        }
        return Task::batch(tasks);
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        set_config_field("model", serde_json::Value::String(model.to_string()));
        set_config_field("auto_model", serde_json::Value::Bool(auto_model));
        set_config_field(
            "acp_agent",
            acp_agent
                .map(|value| serde_json::Value::String(value.to_string()))
                .unwrap_or(serde_json::Value::Null),
        );
        if let Some(project_path) = app.project_path.clone() {
            save_project_chat_preferences(&project_path, model, auto_model, acp_agent);
        }
        Task::none()
    }
}

#[cfg(target_arch = "wasm32")]
fn save_config_field_task(key: &'static str, value: serde_json::Value) -> Task<Message> {
    let key_owned = key.to_string();
    Task::perform(
        async move {
            let mut cfg = crate::app::config::load_app_config_async().await?;
            if let Some(obj) = cfg.as_object_mut() {
                obj.insert(key_owned.clone(), value);
            } else {
                cfg = serde_json::json!({ key_owned: value });
            }
            crate::app::config::save_app_config_async(cfg).await
        },
        move |result| {
            if let Err(error) = result {
                tracing::warn!(target: "vw_desktop", key = key, error = %error, "failed to save desktop preference field");
            }
            Message::Chat(ChatMessage::SessionSaveAck)
        },
    )
}

#[cfg(target_arch = "wasm32")]
fn save_project_chat_preferences_task(
    project_path: String,
    model: String,
    auto_model: bool,
    acp_agent: Option<String>,
) -> Task<Message> {
    Task::perform(
        async move {
            crate::app::config::save_project_chat_preferences_async(
                &project_path,
                &model,
                auto_model,
                acp_agent.as_deref(),
            )
            .await
        },
        move |result: Result<(), String>| {
            if let Err(error) = result {
                tracing::warn!(target: "vw_desktop", error = %error, "failed to save project chat preferences");
            }
            Message::Chat(ChatMessage::SessionSaveAck)
        },
    )
}

fn message_id_for_index(app: &App, msg_idx: usize) -> Option<String> {
    app.chat_message_ids.get(msg_idx).and_then(|id| id.clone())
}

#[cfg(target_arch = "wasm32")]
fn save_session_task(session: models::ChatSession, directory: Option<String>) -> Task<Message> {
    Task::perform(
        async move {
            crate::app::session_gateway::gateway_save_session_async(&session, directory.as_deref())
                .await
        },
        |result| {
            if let Err(error) = result {
                tracing::warn!(target: "vw_desktop", error = %error, "failed to save session");
            }
            Message::Chat(ChatMessage::SessionSaveAck)
        },
    )
}

#[cfg(not(target_arch = "wasm32"))]
fn save_session_task(session: models::ChatSession, directory: Option<String>) -> Task<Message> {
    let _ = crate::app::session_gateway::gateway_save_session(&session, directory.as_deref());
    Task::none()
}

#[cfg(target_arch = "wasm32")]
fn save_agent_session_scoped_task(info: session::Info, scope: Option<String>) -> Task<Message> {
    Task::perform(
        async move {
            crate::app::session_gateway::gateway_save_agent_session_scoped_async(
                &info,
                scope.as_deref(),
            )
            .await
        },
        |result| {
            if let Err(error) = result {
                tracing::warn!(target: "vw_desktop", error = %error, "failed to save scoped agent session");
            }
            Message::Chat(ChatMessage::SessionSaveAck)
        },
    )
}

#[cfg(not(target_arch = "wasm32"))]
fn save_agent_session_scoped_task(info: session::Info, scope: Option<String>) -> Task<Message> {
    let _ = crate::app::session_gateway::gateway_save_agent_session_scoped(&info, scope.as_deref());
    Task::none()
}

/// 公开函数，执行 update 对应的应用流程。
/// 返回值表达处理结果；失败通过错误值、日志或任务消息显式传递。
pub fn update(app: &mut App, message: ChatMessage) -> Task<Message> {
    match message {
        ChatMessage::SendPressed => {
            let runtime = app.current_session_runtime();
            let query = runtime.input_editor.text().to_string();
            if query.trim().is_empty() && app.files.is_empty() {
                return Task::none();
            }
            let task_mode_enabled = runtime.task_mode_enabled;
            let task_mode_priority = runtime.task_mode_priority.clone();
            let task_mode_model = runtime.task_mode_model.clone();
            let task_mode_subtasks = runtime.task_mode_subtasks.clone();
            if let Some(command) = parse_inline_mode_command(&query) {
                return apply_inline_mode_command(app, command);
            }
            let attachments = std::mem::take(&mut app.files);
            if task_mode_enabled {
                return Task::done(Message::TaskBoard(
                    TaskBoardMessage::AddTaskFromInputWithOptions {
                        content: compose_query_with_attachments(&query, &attachments),
                        priority: task_mode_priority,
                        model: task_mode_model,
                        subtasks: task_mode_subtasks,
                    },
                ));
            }
            let runtime = app.current_session_runtime();
            let root = app
                .active_session_id
                .as_ref()
                .and_then(|id| {
                    app.sessions.iter().find(|s| &s.id == id).map(|s| s.directory.clone())
                })
                .or_else(|| app.project_path.clone());
            let model = if runtime.auto_model { None } else { Some(runtime.model.clone()) };
            let acp_agent = resolve_acp_agent(app, runtime.acp_agent.clone());
            let agent = request_delegate_agent(runtime.agent.clone());
            let allowed_tools = selected_delegate_allowed_tools(app, &runtime);
            let acp_test = acp_agent.is_some();
            let acp_history_mode = runtime.acp_history_mode;
            let acp_recent_count = runtime.acp_recent_count.clamp(1, 20);
            let full_access_enabled = runtime.full_access_enabled;
            let acp_force_new_session = acp_agent.is_some()
                && (runtime.acp_rebuild_required
                    || runtime.last_effective_acp_agent.as_ref() != acp_agent.as_ref());
            let created_ms = crate::app::time::now_ms();
            debug!(
                target: "vw_desktop",
                active_session_id = ?app.active_session_id,
                acp_test,
                acp_agent = ?acp_agent,
                acp_force_new_session,
                acp_history_mode = %acp_history_mode.as_str(),
                acp_recent_count,
                model = ?model,
                agent = ?agent,
                allowed_tools = ?allowed_tools,
                has_root = root.is_some(),
                query_len = query.len(),
                "desktop chat request prepared"
            );
            let item = QueueItem {
                created_ms,
                query,
                attachments,
                root,
                model,
                acp_test,
                acp_agent,
                agent,
                allowed_tools,
                acp_force_new_session,
                acp_history_mode,
                acp_recent_count,
                full_access_enabled,
                send_behavior: app.chat_send_behavior,
                request_history_override: None,
                resume_history_only: false,
            };

            let is_requesting = app.current_session_is_requesting();
            match app.chat_send_behavior {
                ChatSendBehavior::StopAndSend => {
                    if let Some(session_id) = app.active_session_id.clone() {
                        let runtime = app.get_session_runtime_mut(&session_id);
                        if let Some(request_id) = runtime.active_agent_request.as_ref().map(|request| request.id) {
                            crate::app::state::clear_pending_guide_handoff(request_id);
                        }
                        runtime.is_requesting = false;
                        runtime.submit_anim = 0;
                        runtime.active_agent_request = None;
                    }
                    app.show_send_mode_popover = false;
                    start(app, item, false)
                }
                ChatSendBehavior::Guide if is_requesting => {
                    debug!(
                        target: "vw_desktop",
                        active_session_id = ?app.active_session_id,
                        queue_len = app.current_session_runtime().queue.len() + 1,
                        "desktop chat request guided ahead of queued requests"
                    );
                    if let Some(request_id) = app
                        .current_session_runtime()
                        .active_agent_request
                        .as_ref()
                        .map(|request| request.id)
                    {
                        crate::app::state::mark_pending_guide_handoff(request_id);
                    }
                    let runtime = app.current_session_runtime_mut();
                    runtime.queue.insert(0, item);
                    runtime.input_editor = iced::widget::text_editor::Content::new();
                    Task::none()
                }
                ChatSendBehavior::Queue if is_requesting => {
                    debug!(
                        target: "vw_desktop",
                        active_session_id = ?app.active_session_id,
                        queue_len = app.current_session_runtime().queue.len() + 1,
                        "desktop chat request queued while another request is active"
                    );
                    let runtime = app.current_session_runtime_mut();
                    runtime.queue.push(item);
                    runtime.input_editor = iced::widget::text_editor::Content::new();
                    Task::none()
                }
                ChatSendBehavior::Guide | ChatSendBehavior::Queue => start(app, item, false),
            }
        }
        ChatMessage::CancelPressed => {
            let session_id = match &app.active_session_id {
                Some(id) => id.clone(),
                None => return Task::none(),
            };

            let is_requesting = app.current_session_is_requesting();
            if !is_requesting {
                return Task::none();
            }

            {
                let runtime = app.get_session_runtime_mut(&session_id);
                if let Some(request_id) = runtime.active_agent_request.as_ref().map(|request| request.id) {
                    crate::app::state::clear_pending_guide_handoff(request_id);
                }
                runtime.is_requesting = false;
                runtime.submit_anim = 0;
                runtime.active_agent_request = None;
            }
            app.show_send_mode_popover = false;
            app.sync_active_session_from_chat();

            let runtime = app.get_session_runtime(&session_id);
            if runtime.queue.is_empty() {
                Task::none()
            } else {
                let runtime = app.get_session_runtime_mut(&session_id);
                let item = runtime.queue.remove(0);
                start(app, item, true)
            }
        }
        ChatMessage::AutoModelToggled(b) => {
            let (model, auto_model) = {
                let runtime = app.current_session_runtime_mut();
                runtime.auto_model = b;
                if b {
                    runtime.task_mode_model = "auto".to_string();
                }
                (runtime.model.clone(), runtime.auto_model)
            };
            app.auto_model = auto_model;
            persist_chat_preferences(app, &model, auto_model, app.acp_agent.as_deref())
        }
        ChatMessage::AcpAgentSelected(agent) => {
            let (model, auto_model) = {
                let runtime = app.current_session_runtime_mut();
                runtime.acp_agent = agent.clone();
                (runtime.model.clone(), runtime.auto_model)
            };
            app.acp_agent = agent.clone();
            app.show_acp_popover = false;
            info!(
                target: "vw_desktop",
                requested_acp_agent = ?agent,
                selected_acp_agent = ?app.acp_agent,
                is_default = app.acp_agent.is_none(),
                "desktop ACP agent updated"
            );
            persist_chat_preferences(app, &model, auto_model, app.acp_agent.as_deref())
        }
        ChatMessage::SessionAgentSelected(agent) => {
            let normalized_agent = normalize_delegate_agent(agent);
            let base_tools = {
                let mut next_runtime = app.current_session_runtime();
                next_runtime.agent = normalized_agent.clone();
                app.session_tool_inventory(&next_runtime).base_tools
            };
            let switch_to_tools_tab = app.show_session_tool_selector_popover;
            let runtime = app.current_session_runtime_mut();
            runtime.agent = normalized_agent.clone();
            runtime.tool_selector.reconcile_tools(&base_tools);
            if switch_to_tools_tab {
                runtime
                    .tool_selector
                    .select_tab(crate::app::state::SessionToolSelectorTab::Tools);
            }
            info!(
                target: "vw_desktop",
                selected_agent = ?normalized_agent,
                "desktop session delegate agent updated"
            );
            Task::none()
        }
        ChatMessage::AcpHistoryModeSelected(mode) => {
            let recent_count = {
                let runtime = app.current_session_runtime_mut();
                runtime.acp_history_mode = mode;
                runtime.acp_recent_count.clamp(1, 20)
            };
            persist_acp_history_preferences(app, mode, recent_count)
        }
        ChatMessage::AcpHistoryRecentCountChanged(raw) => {
            let parsed = raw.trim().parse::<usize>().ok().unwrap_or(3).clamp(1, 20);
            {
                let runtime = app.current_session_runtime_mut();
                runtime.acp_recent_count = parsed;
            }
            let mode = app.current_session_runtime().acp_history_mode;
            persist_acp_history_preferences(app, mode, parsed)
        }
        ChatMessage::ModelSelected(m) => {
            let (model, auto_model): (String, bool) = {
                let runtime = app.current_session_runtime_mut();
                let trimmed = m.trim();
                runtime.model =
                    if trimmed.is_empty() { "auto".to_string() } else { trimmed.to_string() };
                runtime.auto_model = false;
                runtime.task_mode_model = runtime.model.clone();
                (runtime.model.clone(), runtime.auto_model)
            };
            app.model = model.clone();
            app.auto_model = auto_model;
            app.show_model_popover = false;
            app.show_mode_popover = false;
            persist_chat_preferences(app, &model, auto_model, app.acp_agent.as_deref())
        }
        ChatMessage::ModelInputChanged(m) => {
            let (model, auto_model): (String, bool) = {
                let runtime = app.current_session_runtime_mut();
                let trimmed = m.trim();
                runtime.model =
                    if trimmed.is_empty() { "auto".to_string() } else { trimmed.to_string() };
                runtime.auto_model = runtime.model == "auto";
                runtime.task_mode_model = runtime.model.clone();
                (runtime.model.clone(), runtime.auto_model)
            };
            app.model = model.clone();
            app.auto_model = auto_model;
            persist_chat_preferences(app, &model, auto_model, app.acp_agent.as_deref())
        }
        ChatMessage::SessionToolBucketToggled(bucket) => {
            let runtime = app.current_session_runtime_mut();
            if !runtime.tool_selector.toggle_bucket(bucket) {
                app.push_notification("至少保留一个工具分桶".to_string());
            }
            Task::none()
        }
        ChatMessage::SessionToolSelectorTabSelected(tab) => {
            let runtime = app.current_session_runtime_mut();
            runtime.tool_selector.select_tab(tab);
            Task::none()
        }
        ChatMessage::SessionToolGroupCollapsedToggled(group) => {
            let runtime = app.current_session_runtime_mut();
            runtime.tool_selector.toggle_group_collapsed(group);
            Task::none()
        }
        ChatMessage::SessionToolGroupToolsToggled(group) => {
            let base_tools = app.current_session_tool_inventory().base_tools;
            let runtime = app.current_session_runtime_mut();
            if !runtime.tool_selector.toggle_group_tools(&base_tools, group) {
                app.push_notification("至少保留一个会话工具".to_string());
            }
            Task::none()
        }
        ChatMessage::SessionToolSelectorSelectAll => {
            let base_tools = app.current_session_tool_inventory().base_tools;
            let runtime = app.current_session_runtime_mut();
            runtime.tool_selector.select_all_tools(&base_tools);
            Task::none()
        }
        ChatMessage::SessionToolSelectorInvert => {
            let base_tools = app.current_session_tool_inventory().base_tools;
            let runtime = app.current_session_runtime_mut();
            if !runtime.tool_selector.invert_tools(&base_tools) {
                app.push_notification("当前工具范围无法直接反选".to_string());
            }
            Task::none()
        }
        ChatMessage::SessionToolToggled(tool_id) => {
            let base_tools = app.current_session_tool_inventory().base_tools;
            let runtime = app.current_session_runtime_mut();
            if !runtime.tool_selector.toggle_tool(&base_tools, &tool_id) {
                app.push_notification("至少保留一个会话工具".to_string());
            }
            Task::none()
        }
        ChatMessage::SessionToolSelectorReset => {
            let runtime = app.current_session_runtime_mut();
            runtime.tool_selector.reset();
            Task::none()
        }
        ChatMessage::QueueRemove(i) => {
            let runtime = app.current_session_runtime_mut();
            if i < runtime.queue.len() {
                runtime.queue.remove(i);
            }
            Task::none()
        }
        ChatMessage::QueueUp(i) => {
            let runtime = app.current_session_runtime_mut();
            if i > 0 && i < runtime.queue.len() {
                runtime.queue.swap(i, i - 1);
            }
            Task::none()
        }
        ChatMessage::QueueDown(i) => {
            let runtime = app.current_session_runtime_mut();
            if i + 1 < runtime.queue.len() {
                runtime.queue.swap(i, i + 1);
            }
            Task::none()
        }
        ChatMessage::SubmitTick => {
            let is_requesting = app.current_session_is_requesting();
            if !is_requesting {
                let runtime = app.current_session_runtime_mut();
                runtime.submit_anim = 0;
                return Task::none();
            }
            let runtime = app.current_session_runtime_mut();
            runtime.submit_anim = (runtime.submit_anim + 1) % 3;
            message::after(Duration::from_millis(200), Message::Chat(ChatMessage::SubmitTick))
        }
        ChatMessage::ForkSessionAt(msg_idx) => {
            let Some(session_id) = app.active_session_id.clone() else {
                return Task::none();
            };
            let Some(message_id) = message_id_for_index(app, msg_idx) else {
                app.push_notification("当前消息暂不支持分叉".to_string());
                return Task::none();
            };
            let Some(base_chat) = app.chat.get(..=msg_idx).map(|slice| slice.to_vec()) else {
                return Task::none();
            };
            let Some(base_message_ids) =
                app.chat_message_ids.get(..=msg_idx).map(|slice| slice.to_vec())
            else {
                return Task::none();
            };
            let root = current_session_directory(app).or_else(|| app.project_path.clone());
            let runtime = app.current_session_runtime();
            let model = if runtime.auto_model { None } else { Some(runtime.model.clone()) };
            let endpoint = gateway_endpoint(app);
            Task::perform(
                async move {
                    let result = match vw_gateway_client::GatewayClient::new(
                        vw_gateway_client::GatewayEndpoint::new(endpoint.0, endpoint.1),
                    ) {
                        Ok(client) => {
                            client
                                .session_fork::<session::Info>(
                                    &session_id,
                                    root.as_deref(),
                                    &Some(vw_gateway_client::GatewaySessionForkBody {
                                        message_id: Some(message_id),
                                    }),
                                )
                                .await
                        }
                        Err(err) => Err(err),
                    };
                    ChatMessage::ForkSessionFinished {
                        result,
                        base_chat,
                        base_message_ids,
                        root,
                        model,
                    }
                },
                Message::Chat,
            )
        }
        ChatMessage::ForkSessionFinished { result, base_chat, base_message_ids, root, model } => {
            match result {
                Ok(info) => {
                    app.cache_active_session_chat();
                    if !app.sessions.iter().any(|session| session.id == info.id) {
                        app.sessions.insert(0, info.clone());
                    }
                    if let Some(list) = app.project_sessions.get_mut(&info.directory) {
                        if !list.iter().any(|session| session.id == info.id) {
                            list.insert(0, info.clone());
                        }
                    } else {
                        app.project_sessions.insert(info.directory.clone(), vec![info.clone()]);
                    }
                    app.active_session_id = Some(info.id.clone());
                    app.chat = base_chat;
                    app.chat_message_ids = base_message_ids;
                    app.chat_reset_menu_idx = None;
                    app.invalidate_chat_ui_state();
                    app.sync_active_session_preferences();
                    app.store_session_chat_snapshot(
                        info.id.clone(),
                        crate::app::session::shared_chat_messages(app.chat.clone()),
                        app.chat_message_ids.clone(),
                    );
                    let created_ms = crate::app::time::now_ms();
                    let current_runtime = app.current_session_runtime();
                    let acp_agent =
                        resolve_acp_agent(app, current_runtime.acp_agent.clone());
                    let agent = request_delegate_agent(current_runtime.agent.clone());
                    let allowed_tools = selected_delegate_allowed_tools(app, &current_runtime);
                    let acp_test = acp_agent.is_some();
                    start(
                        app,
                        QueueItem {
                            created_ms,
                            query: String::new(),
                            attachments: Vec::new(),
                            root,
                            model,
                            acp_test,
                            acp_agent,
                            agent,
                            allowed_tools,
                            acp_force_new_session: true,
                            acp_history_mode: current_runtime.acp_history_mode,
                            acp_recent_count: current_runtime.acp_recent_count,
                            full_access_enabled: current_runtime.full_access_enabled,
                            send_behavior: ChatSendBehavior::Queue,
                            request_history_override: None,
                            resume_history_only: false,
                        },
                        true,
                    )
                }
                Err(error) => {
                    app.push_notification(format!("分叉会话失败: {}", error));
                    Task::none()
                }
            }
        }
        ChatMessage::ResetSessionToMessage { msg_idx, revert_code } => {
            let Some(session_id) = app.active_session_id.clone() else {
                return Task::none();
            };
            let Some(message_id) = message_id_for_index(app, msg_idx) else {
                app.push_notification("当前消息暂不支持重置".to_string());
                return Task::none();
            };
            app.chat_reset_menu_idx = None;
            let root = current_session_directory(app).or_else(|| app.project_path.clone());
            let endpoint = gateway_endpoint(app);
            Task::perform(
                async move {
                    let result = match vw_gateway_client::GatewayClient::new(
                        vw_gateway_client::GatewayEndpoint::new(endpoint.0, endpoint.1),
                    ) {
                        Ok(client) => {
                            client
                                .session_reset::<session::Info>(
                                    &session_id,
                                    root.as_deref(),
                                    &vw_gateway_client::GatewaySessionResetBody {
                                        message_id,
                                        revert_code,
                                    },
                                )
                                .await
                        }
                        Err(err) => Err(err),
                    };
                    ChatMessage::ResetSessionFinished { result, session_id }
                },
                Message::Chat,
            )
        }
        ChatMessage::ResetSessionFinished { result, session_id } => match result {
            Ok(info) => {
                if let Some(existing) =
                    app.sessions.iter_mut().find(|session| session.id == info.id)
                {
                    *existing = info.clone();
                }
                for sessions in app.project_sessions.values_mut() {
                    if let Some(existing) =
                        sessions.iter_mut().find(|session| session.id == info.id)
                    {
                        *existing = info.clone();
                    }
                }
                app.mark_session_acp_rebuild_required(&session_id);
                Task::done(Message::Project(ProjectMessage::OpenProjectSessionPressed(
                    info.directory.clone(),
                    session_id,
                )))
            }
            Err(error) => {
                app.push_notification(format!("重置会话失败: {}", error));
                Task::none()
            }
        },
        _ => Task::none(),
    }
}

/// 模块内可见函数，执行 start 对应的应用流程。
/// 返回值表达处理结果；失败通过错误值、日志或任务消息显式传递。
pub(super) fn start(app: &mut App, item: QueueItem, keep: bool) -> Task<Message> {
    app.agent_stream_id = app.agent_stream_id.wrapping_add(1);
    let stream_id = app.agent_stream_id;
    let mut save_tasks = Vec::new();
    let pending_runtime = app.current_session_runtime();

    app.cache_active_session_chat();

    if app.active_session_id.is_none() {
        let id = id::descending(id::Prefix::Session, None)
            .unwrap_or_else(|_| format!("ses_{}", stream_id));
        let title_input = item.query.trim().to_string();
        let title = if title_input.is_empty() { "新会话".to_string() } else { title_input };
        let directory = app.project_path.clone().unwrap_or_default();
        let inferred_project_id = app
            .project_id
            .clone()
            .or_else(|| {
                app.project_sessions
                    .get(&directory)
                    .and_then(|sessions| sessions.first().map(|s| s.project_id.clone()))
            })
            .or_else(|| {
                app.sessions.iter().find(|s| s.directory == directory).map(|s| s.project_id.clone())
            });
        let Some(inferred_project_id) = inferred_project_id else {
            app.error_message = Some("未选择项目，无法创建会话".to_string());
            return Task::none();
        };
        let now = crate::app::time::now_ms();

        let info = session::Info {
            id: id.clone(),
            slug: create_slug(),
            project_id: inferred_project_id,
            directory,
            parent_id: None,
            summary: None,
            share: None,
            title: title.clone(),
            version: "0.0.0".to_string(),
            time: session::TimeInfo {
                created: now,
                updated: now,
                compacting: None,
                archived: None,
            },
            permission: None,
            revert: None,
        };

        save_tasks
            .push(save_agent_session_scoped_task(info.clone(), Some(info.project_id.clone())));

        app.sessions.insert(0, info);
        let info = app.sessions[0].clone();
        if let Some(list) = app.project_sessions.get_mut(&info.directory) {
            if !list.iter().any(|s| s.id == info.id) {
                list.insert(0, info.clone());
            }
        } else {
            app.project_sessions.insert(info.directory.clone(), vec![info.clone()]);
            app.project_session_load_counts.insert(info.directory.clone(), 10);
        }
        app.active_session_id = Some(id.clone());
        let runtime = app.get_session_runtime_mut(&id);
        runtime.full_access_enabled = pending_runtime.full_access_enabled;
    }

    let request_history = item.request_history_override.clone().unwrap_or_else(|| app.chat.clone());
    let request_query = if item.resume_history_only {
        String::new()
    } else {
        compose_query_with_attachments(&item.query, &item.attachments)
    };
    let appended_start_idx = app.chat.len();
    if !request_query.trim().is_empty() {
        app.chat.push(models::ChatMessage {
            role: models::ChatRole::User,
            content: request_query.clone(),
            think_timing: Vec::new(),
        });
        app.chat_message_ids.push(None);
    }
    app.chat.push(models::ChatMessage {
        role: models::ChatRole::Assistant,
        content: String::new(),
        think_timing: Vec::new(),
    });
    app.chat_message_ids.push(None);
    app.invalidate_chat_ui_for_message_idx(
        appended_start_idx.min(app.chat.len().saturating_sub(1)),
    );
    if let Some(last_idx) = app.chat.len().checked_sub(1) {
        app.invalidate_chat_ui_for_message_idx(last_idx);
    }
    app.sync_chat_message_estimated_heights_len();
    app.refine_chat_message_estimated_heights(appended_start_idx, app.chat.len());
    app.active_session_view_state.updated_ms = item.created_ms;
    app.rebuild_active_session_message_meta();
    if let Some(session_id) = app.active_session_id.clone() {
        app.store_session_chat_snapshot(
            session_id,
            crate::app::session::shared_chat_messages(app.chat.clone()),
            app.chat_message_ids.clone(),
        );
    }
    if !keep {
        let is_empty_session = app.active_session_id.is_none();
        let runtime = app.current_session_runtime_mut();
        runtime.input_editor = iced::widget::text_editor::Content::new();
        if is_empty_session {
            app.input_editor = runtime.input_editor.clone();
        }
    }

    if let Some(session_id) = app.active_session_id.clone() {
        let runtime = app.get_session_runtime_mut(&session_id);
        runtime.is_requesting = true;
        runtime.submit_anim = 0;
        runtime.has_unseen_success = false;
        info!(
            target: "vw_desktop",
            request_id = stream_id,
            session_id = %session_id,
            acp_test = item.acp_test,
            acp_agent = ?item.acp_agent,
            acp_allowed_tools = ?Option::<Vec<String>>::None,
            agent = ?item.agent,
            allowed_tools = ?item.allowed_tools,
            acp_force_new_session = item.acp_force_new_session,
            acp_history_mode = %item.acp_history_mode.as_str(),
            acp_recent_count = item.acp_recent_count,
            full_access_enabled = item.full_access_enabled,
            model = ?item.model,
            has_root = item.root.is_some(),
            query_len = request_query.len(),
            history_len = request_history.len(),
            resume_history_only = item.resume_history_only,
            keep_input = keep,
            "desktop agent request started"
        );
        runtime.active_agent_request = Some(AgentRequest {
            id: stream_id,
            session: session_id.clone(),
            query: request_query,
            root: item.root,
            model: item.model,
            acp_test: item.acp_test,
            acp_agent: item.acp_agent,
            acp_allowed_tools: None,
            agent: item.agent,
            allowed_tools: item.allowed_tools,
            acp_force_new_session: item.acp_force_new_session,
            acp_history_mode: item.acp_history_mode,
            acp_recent_count: item.acp_recent_count,
            full_access_enabled: item.full_access_enabled,
            resume_history_only: item.resume_history_only,
            history: request_history,
        });
        if runtime
            .queue
            .first()
            .is_some_and(|queued| queued.send_behavior == ChatSendBehavior::Guide)
        {
            crate::app::state::mark_pending_guide_handoff(stream_id);
        } else {
            crate::app::state::clear_pending_guide_handoff(stream_id);
        }
        if runtime.active_agent_request.as_ref().is_some_and(|request| request.acp_test) {
            runtime.acp_rebuild_required = false;
        }
        runtime.last_effective_acp_agent =
            runtime.active_agent_request.as_ref().and_then(|request| request.acp_agent.clone());

        let now = crate::app::time::now_ms();
        let title = app
            .sessions
            .iter()
            .find(|s| s.id == session_id)
            .map(|s| s.title.clone())
            .unwrap_or_else(|| "新会话".to_string());
        let mut local = models::ChatSession {
            id: session_id.clone(),
            title,
            messages: vec![],
            message_ids: vec![],
            calls: vec![],
            steps: vec![],
            created_ms: now,
            updated_ms: now,
        };
        local.messages = app.chat.clone();
        local.message_ids = app.chat_message_ids.clone();
        local.updated_ms = now;
        let session_directory = current_session_directory(app).or_else(|| app.project_path.clone());
        save_tasks.push(save_session_task(local, session_directory));
    }

    app.sync_active_session_from_chat();
    let submit = message::after(Duration::from_millis(200), Message::Chat(ChatMessage::SubmitTick));
    let title_task = stream::maybe_generate_session_title(app);
    save_tasks.push(submit);
    save_tasks.push(Task::done(Message::Chat(ChatMessage::LoadInputPanelTodos)));
    if app.chat_auto_scroll {
        save_tasks.push(super::scroll_chat_to_bottom_task(app).chain(title_task));
    } else {
        save_tasks.push(title_task);
    }
    Task::batch(save_tasks)
}
#[cfg(test)]
#[path = "session_tests.rs"]
mod session_tests;
