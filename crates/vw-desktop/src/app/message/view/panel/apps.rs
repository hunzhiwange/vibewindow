use super::ViewMessage;
use crate::app::{App, Message, Screen, state::UsageModelInfo};
use iced::Task;

pub fn update(app: &mut App, message: ViewMessage) -> Task<Message> {
    match message {
        ViewMessage::OpenApps => {
            let id = "apps".to_string();
            if !app.open_tabs.iter().any(|t| t.id == id) {
                app.open_tabs.push(crate::app::AppTab {
                    id: id.clone(),
                    title: "应用".to_string(),
                    screen: Screen::Apps,
                    project_path: None,
                });
            }
            app.active_tab_id = Some(id);
            app.screen = Screen::Apps;
            Task::none()
        }
        ViewMessage::OpenDesign => {
            let id = "design".to_string();

            if let Some(pos) = app.open_tabs.iter().position(|t| t.id == "apps") {
                app.open_tabs.remove(pos);
            }

            if !app.open_tabs.iter().any(|t| t.id == id) {
                app.open_tabs.push(crate::app::AppTab {
                    id: id.clone(),
                    title: "设计".to_string(),
                    screen: Screen::Design,
                    project_path: None,
                });
            }
            app.active_tab_id = Some(id);
            app.screen = Screen::Design;
            Task::none()
        }
        ViewMessage::OpenUsage => {
            let id = "usage".to_string();
            let active_session_id: Option<String> = app.active_session_id.as_ref().cloned();
            let insert_at =
                if app.open_tabs.first().map(|t| t.id.as_str()) == Some("home") { 1 } else { 0 };
            if let Some(pos) = app.open_tabs.iter().position(|t| t.id == id) {
                let tab = app.open_tabs.remove(pos);
                let i = insert_at.min(app.open_tabs.len());
                app.open_tabs.insert(i, tab);
            } else {
                let i = insert_at.min(app.open_tabs.len());
                app.open_tabs.insert(
                    i,
                    crate::app::AppTab {
                        id: id.clone(),
                        title: "用量".to_string(),
                        screen: Screen::Usage,
                        project_path: None,
                    },
                );
            }
            app.active_tab_id = Some(id);
            app.screen = Screen::Usage;
            app.usage_model_info = None;
            app.usage_session_file_path = None;
            let runtime = app.current_session_runtime();
            let model = if runtime.auto_model { None } else { Some(runtime.model) };
            let model_task = Task::perform(
                async move {
                    use crate::app::provider::provider;
                    fn pick_provider_id(candidates: &[String], model_id: &str) -> Option<String> {
                        if candidates.is_empty() {
                            return None;
                        }
                        let first_segment = model_id.split('/').next().unwrap_or_default();
                        let first_segment_lower = first_segment.to_ascii_lowercase();
                        if !first_segment.is_empty() {
                            if let Some(v) =
                                candidates.iter().find(|id| id.as_str() == first_segment)
                            {
                                return Some(v.clone());
                            }
                            if let Some(v) = candidates
                                .iter()
                                .find(|id| id.to_ascii_lowercase().contains(&first_segment_lower))
                            {
                                return Some(v.clone());
                            }
                        }

                        let model_lower = model_id.to_ascii_lowercase();
                        let prefer =
                            if model_lower.starts_with("gpt-") || model_lower.starts_with('o') {
                                Some("openai")
                            } else if model_lower.contains("claude") {
                                Some("anthropic")
                            } else if model_lower.contains("deepseek") {
                                Some("deepseek")
                            } else {
                                None
                            };
                        if let Some(substr) = prefer
                            && let Some(v) = candidates
                                .iter()
                                .find(|id| id.to_ascii_lowercase().contains(substr))
                        {
                            return Some(v.clone());
                        }
                        if model_lower.contains("deepseek")
                            && let Some(v) = candidates
                                .iter()
                                .find(|id| id.to_ascii_lowercase().contains("openrouter"))
                        {
                            return Some(v.clone());
                        }
                        Some(candidates[0].clone())
                    }

                    let parsed = match model {
                        Some(s) => {
                            let s: String = s;
                            if s.contains('/') {
                                let parsed = provider::parse_model(&s);
                                if provider::get_model(&parsed.provider_id, &parsed.model_id)
                                    .await
                                    .is_ok()
                                {
                                    Some(parsed)
                                } else {
                                    let providers = provider::list().await;
                                    let mut candidates = Vec::<String>::new();
                                    for (provider_id, info) in providers {
                                        if info.models.contains_key(&s) {
                                            candidates.push(provider_id);
                                        }
                                    }
                                    pick_provider_id(&candidates, &s).map(|provider_id| {
                                        provider::ParsedModelRef {
                                            provider_id,
                                            model_id: s.clone(),
                                        }
                                    })
                                }
                            } else {
                                let providers = provider::list().await;
                                let mut candidates = Vec::<String>::new();
                                for (provider_id, info) in providers {
                                    if info.models.contains_key(&s) {
                                        candidates.push(provider_id);
                                    }
                                }
                                pick_provider_id(&candidates, &s).map(|provider_id| {
                                    provider::ParsedModelRef { provider_id, model_id: s.clone() }
                                })
                            }
                        }
                        None => provider::default_model().await.ok(),
                    };
                    let Some(parsed) = parsed else {
                        return None;
                    };
                    let m =
                        provider::get_model(&parsed.provider_id, &parsed.model_id).await.ok()?;
                    let provider_name = provider::get_provider(&parsed.provider_id)
                        .await
                        .map(|p| p.name)
                        .unwrap_or_else(|| parsed.provider_id.clone());
                    Some(UsageModelInfo {
                        provider_id: parsed.provider_id,
                        provider_name,
                        model_id: m.id.clone(),
                        model_name: m.name,
                        context_limit: m.limit.context,
                        output_limit: m.limit.output,
                        cost_input_per_million: m.cost.input,
                        cost_output_per_million: m.cost.output,
                        cost_cache_read_per_million: m.cost.cache.read,
                        cost_cache_write_per_million: m.cost.cache.write,
                    })
                },
                |info| Message::View(ViewMessage::UsageModelInfoLoaded(info)),
            );
            let file_path_task = if let Some(session_id) = active_session_id {
                Task::perform(
                    {
                        let session_id = session_id.as_str().to_owned();
                        async move {
                            let result =
                                crate::app::session_gateway::gateway_session_file_path_async(
                                    &session_id,
                                )
                                .await;
                            (session_id, result)
                        }
                    },
                    |(session_id, result)| {
                        Message::View(ViewMessage::UsageSessionFilePathLoaded(session_id, result))
                    },
                )
            } else {
                Task::none()
            };
            Task::batch(vec![model_task, file_path_task])
        }
        ViewMessage::OpenJsonTool => {
            let id = "json_tool".to_string();
            // Close Apps middle page tab if present
            if let Some(pos) = app.open_tabs.iter().position(|t| t.id == "apps") {
                app.open_tabs.remove(pos);
            }
            if !app.open_tabs.iter().any(|t| t.id == id) {
                app.open_tabs.push(crate::app::AppTab {
                    id: id.clone(),
                    title: "JSON工具".to_string(),
                    screen: Screen::JsonTool,
                    project_path: None,
                });
            }
            app.active_tab_id = Some(id);
            app.screen = Screen::JsonTool;
            Task::none()
        }
        ViewMessage::OpenJsonYamlTool => {
            let id = "json_yaml_tool".to_string();
            if let Some(pos) = app.open_tabs.iter().position(|t| t.id == "apps") {
                app.open_tabs.remove(pos);
            }
            if !app.open_tabs.iter().any(|t| t.id == id) {
                app.open_tabs.push(crate::app::AppTab {
                    id: id.clone(),
                    title: "JSON/YAML互转工具".to_string(),
                    screen: Screen::JsonYamlTool,
                    project_path: None,
                });
            }
            app.active_tab_id = Some(id);
            app.screen = Screen::JsonYamlTool;
            Task::none()
        }
        ViewMessage::OpenSqlTool => {
            let id = "sql_tool".to_string();
            if let Some(pos) = app.open_tabs.iter().position(|t| t.id == "apps") {
                app.open_tabs.remove(pos);
            }
            if !app.open_tabs.iter().any(|t| t.id == id) {
                app.open_tabs.push(crate::app::AppTab {
                    id: id.clone(),
                    title: "SQL美化工具".to_string(),
                    screen: Screen::SqlTool,
                    project_path: None,
                });
            }
            app.active_tab_id = Some(id);
            app.screen = Screen::SqlTool;
            Task::none()
        }
        ViewMessage::OpenRedisTool => {
            let id = "redis_tool".to_string();
            if let Some(pos) = app.open_tabs.iter().position(|t| t.id == "apps") {
                app.open_tabs.remove(pos);
            }
            if !app.open_tabs.iter().any(|t| t.id == id) {
                app.open_tabs.push(crate::app::AppTab {
                    id: id.clone(),
                    title: "Redis客户端".to_string(),
                    screen: Screen::RedisTool,
                    project_path: None,
                });
            }
            app.active_tab_id = Some(id);
            app.screen = Screen::RedisTool;
            if let Some(selected_id) = app.redis_tool.selected_connection_id.clone()
                && !app.redis_tool.has_detail_tab_data_for_selected(app.redis_tool.detail_tab)
            {
                Task::done(Message::RedisTool(
                    crate::app::message::RedisToolMessage::SelectConnection(selected_id),
                ))
            } else {
                Task::none()
            }
        }
        ViewMessage::OpenHtmlTool => {
            let id = "html_tool".to_string();
            if let Some(pos) = app.open_tabs.iter().position(|t| t.id == "apps") {
                app.open_tabs.remove(pos);
            }
            if !app.open_tabs.iter().any(|t| t.id == id) {
                app.open_tabs.push(crate::app::AppTab {
                    id: id.clone(),
                    title: "HTML美化工具".to_string(),
                    screen: Screen::HtmlTool,
                    project_path: None,
                });
            }
            app.active_tab_id = Some(id);
            app.screen = Screen::HtmlTool;
            Task::none()
        }
        ViewMessage::OpenJsonDiffTool => {
            let id = "json_diff_tool".to_string();
            if let Some(pos) = app.open_tabs.iter().position(|t| t.id == "apps") {
                app.open_tabs.remove(pos);
            }
            if !app.open_tabs.iter().any(|t| t.id == id) {
                app.open_tabs.push(crate::app::AppTab {
                    id: id.clone(),
                    title: "JSON比对工具".to_string(),
                    screen: Screen::JsonDiffTool,
                    project_path: None,
                });
            }
            app.active_tab_id = Some(id);
            app.screen = Screen::JsonDiffTool;
            Task::none()
        }
        ViewMessage::OpenMarkdownTool => {
            let id = "markdown_tool".to_string();
            if let Some(pos) = app.open_tabs.iter().position(|t| t.id == "apps") {
                app.open_tabs.remove(pos);
            }
            if !app.open_tabs.iter().any(|t| t.id == id) {
                app.open_tabs.push(crate::app::AppTab {
                    id: id.clone(),
                    title: "Markdown编辑器".to_string(),
                    screen: Screen::MarkdownTool,
                    project_path: None,
                });
            }
            app.active_tab_id = Some(id);
            app.screen = Screen::MarkdownTool;
            Task::none()
        }
        ViewMessage::OpenWorkflowTool => {
            let id = crate::apps::workflow::WORKFLOW_TOOL_TAB_ID.to_string();
            if let Some(pos) = app.open_tabs.iter().position(|t| t.id == "apps") {
                app.open_tabs.remove(pos);
            }
            if !app.open_tabs.iter().any(|t| t.id == id) {
                app.open_tabs.push(crate::app::AppTab {
                    id: id.clone(),
                    title: "Dify工作流".to_string(),
                    screen: Screen::WorkflowTool,
                    project_path: None,
                });
            }
            app.active_tab_id = Some(id);
            app.screen = Screen::WorkflowTool;
            crate::apps::workflow::ensure_initialized(app)
        }
        ViewMessage::OpenMindMapTool => {
            if let Some(pos) = app.open_tabs.iter().position(|t| t.id == "apps") {
                app.open_tabs.remove(pos);
            }
            app.screen = Screen::MindMapTool;
            crate::apps::mindmap::ensure_initialized(app)
        }
        ViewMessage::OpenPasswordTool => {
            let id = "password_tool".to_string();
            if let Some(pos) = app.open_tabs.iter().position(|t| t.id == "apps") {
                app.open_tabs.remove(pos);
            }
            if !app.open_tabs.iter().any(|t| t.id == id) {
                app.open_tabs.push(crate::app::AppTab {
                    id: id.clone(),
                    title: "随机密码生成器".to_string(),
                    screen: Screen::PasswordTool,
                    project_path: None,
                });
            }
            app.active_tab_id = Some(id);
            app.screen = Screen::PasswordTool;
            Task::none()
        }
        ViewMessage::OpenBaseTool => {
            let id = "base_tool".to_string();
            if let Some(pos) = app.open_tabs.iter().position(|t| t.id == "apps") {
                app.open_tabs.remove(pos);
            }
            if !app.open_tabs.iter().any(|t| t.id == id) {
                app.open_tabs.push(crate::app::AppTab {
                    id: id.clone(),
                    title: "进制转换器".to_string(),
                    screen: Screen::BaseTool,
                    project_path: None,
                });
            }
            app.active_tab_id = Some(id);
            app.screen = Screen::BaseTool;
            Task::none()
        }
        ViewMessage::OpenTimestampTool => {
            let id = "timestamp_tool".to_string();
            if let Some(pos) = app.open_tabs.iter().position(|t| t.id == "apps") {
                app.open_tabs.remove(pos);
            }
            if !app.open_tabs.iter().any(|t| t.id == id) {
                app.open_tabs.push(crate::app::AppTab {
                    id: id.clone(),
                    title: "时间戳转换器".to_string(),
                    screen: Screen::TimestampTool,
                    project_path: None,
                });
            }
            app.active_tab_id = Some(id);
            app.screen = Screen::TimestampTool;
            Task::perform(async {}, |_| {
                Message::TimestampTool(
                    crate::app::message::timestamp_tool::TimestampToolMessage::RefreshNow,
                )
            })
        }
        ViewMessage::OpenQrTool => {
            let id = "qr_tool".to_string();
            if let Some(pos) = app.open_tabs.iter().position(|t| t.id == "apps") {
                app.open_tabs.remove(pos);
            }
            if !app.open_tabs.iter().any(|t| t.id == id) {
                app.open_tabs.push(crate::app::AppTab {
                    id: id.clone(),
                    title: "二维码生成器".to_string(),
                    screen: Screen::QrTool,
                    project_path: None,
                });
            }
            app.active_tab_id = Some(id);
            app.screen = Screen::QrTool;
            Task::none()
        }
        ViewMessage::OpenColorTool => {
            let id = "color_tool".to_string();
            if let Some(pos) = app.open_tabs.iter().position(|t| t.id == "apps") {
                app.open_tabs.remove(pos);
            }
            if !app.open_tabs.iter().any(|t| t.id == id) {
                app.open_tabs.push(crate::app::AppTab {
                    id: id.clone(),
                    title: "颜色转换工具".to_string(),
                    screen: Screen::ColorTool,
                    project_path: None,
                });
            }
            app.active_tab_id = Some(id);
            app.screen = Screen::ColorTool;
            Task::none()
        }
        ViewMessage::OpenCleanerTool => {
            let id = "cleaner_tool".to_string();
            if let Some(pos) = app.open_tabs.iter().position(|t| t.id == "apps") {
                app.open_tabs.remove(pos);
            }
            if !app.open_tabs.iter().any(|t| t.id == id) {
                app.open_tabs.push(crate::app::AppTab {
                    id: id.clone(),
                    title: "电脑垃圾清理工具".to_string(),
                    screen: Screen::CleanerTool,
                    project_path: None,
                });
            }
            app.active_tab_id = Some(id);
            app.screen = Screen::CleanerTool;
            Task::none()
        }
        ViewMessage::OpenLargeFileTool => {
            let id = "large_file_tool".to_string();
            if let Some(pos) = app.open_tabs.iter().position(|t| t.id == "apps") {
                app.open_tabs.remove(pos);
            }
            if !app.open_tabs.iter().any(|t| t.id == id) {
                app.open_tabs.push(crate::app::AppTab {
                    id: id.clone(),
                    title: "大文件查找工具".to_string(),
                    screen: Screen::LargeFileTool,
                    project_path: None,
                });
            }
            app.active_tab_id = Some(id);
            app.screen = Screen::LargeFileTool;
            Task::none()
        }
        ViewMessage::AppsOpenMostRecent => {
            if let Some(path) = app.recent_projects.first().cloned() {
                // Close apps tab immediately
                if let Some(pos) = app.open_tabs.iter().position(|t| t.id == "apps") {
                    app.open_tabs.remove(pos);
                }
                app.active_tab_id = app.open_tabs.last().map(|t| t.id.clone());

                return Task::perform(async {}, move |_| {
                    Message::Project(
                        crate::app::message::project::ProjectMessage::OpenRecentPressed(path),
                    )
                });
            }
            Task::none()
        }
        ViewMessage::AppsSearchChanged(v) => {
            app.apps_search_query = v;
            Task::none()
        }
        _ => Task::none(),
    }
}
#[cfg(test)]
#[path = "apps_tests.rs"]
mod apps_tests;
