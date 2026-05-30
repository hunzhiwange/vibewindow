//! # 系统设置模块
//!
//! 本模块提供系统设置面板的 UI 组件，用于展示和管理 VibeWindow 应用的各类配置选项。
//!
//! ## 主要功能
//!
//! - 提供统一的系统设置入口界面，包含多个配置分类标签页
//! - 支持通过标签页切换浏览不同类别的设置项
//! - 以模态对话框形式展示，支持点击外部区域关闭
//!
//! ## 配置分类
//!
//! 系统设置面板包含以下配置类别：
//!
//! - **常规设置 (General)**：应用基础配置
//! - **对话流 (DialogueFlow)**：对话流程相关配置
//! - **编辑器 (Editor)**：代码编辑器设置
//! - **项目 (Projects)**：项目管理配置
//! - **提供商 (Providers)**：AI 服务提供商配置
//! - **模型 (Models)**：AI 模型配置
//! - **嵌入路由 (EmbeddingRoutes)**：嵌入模型路由配置
//! - **模型路由 (ModelRoutes)**：模式到模型的路由配置
//! - **查询分类 (QueryClassification)**：查询分类规则配置
//! - **目标循环 (GoalLoop)**：自治目标循环设置
//! - **心跳配置 (Heartbeat)**：心跳检测设置
//! - **定时任务配置 (Cron)**：定时任务配置
//! - **标准流程配置 (Sop)**：标准操作流程目录、执行模式与队列限制
//! - **调度配置 (Scheduler)**：任务调度设置
//! - **委托代理配置 (Agents)**：委托代理 researcher/coder/reviewer 配置
//! - **代理通信配置 (AgentsIpc)**：代理间通信配置
//! - **协调配置 (Coordination)**：多代理协调设置
//! - **可靠性配置 (Reliability)**：系统可靠性配置
//! - **通道配置 (Channels)**：CLI 与多通道消息集成配置
//! - **记忆配置 (Memory)**：记忆后端、缓存和嵌入检索配置
//! - **运行时配置 (Runtime)**：native/docker/wasm 运行时配置
//! - **自治配置 (Autonomy)**：代理自治能力设置
//! - **安全配置 (Security)**：安全策略配置
//! - **网关配置 (Gateway)**：HTTP 网关、配对与 node-control 配置
//! - **可观测性配置 (Observability)**：日志与监控配置
//! - **存储配置 (Storage)**：持久化存储 provider 与连接参数
//! - **代理配置 (Proxy)**：网络代理设置
//! - **隧道配置 (Tunnel)**：网关公网暴露隧道配置
//! - **Composio 集成配置 (Composio)**：Composio OAuth 工具集成设置
//! - **技能配置 (Skills)**：代理技能配置
//! - **钩子配置 (Hooks)**：运行时钩子配置
//! - **研究配置 (Research)**：研究功能配置
//! - **网页搜索配置 (WebSearch)**：Web 搜索工具 provider 与请求参数配置
//! - **网络请求配置 (HttpRequest)**：HTTP 请求工具配置
//! - **浏览器配置 (Browser)**：浏览器工具与自动化后端配置
//! - **多模态配置 (Multimodal)**：图像输入数量、大小限制与远程抓取配置
//! - **网络请求配置 (HttpRequest)**：HTTP 请求工具白名单与请求限制
//! - **转录配置 (Transcription)**：语音转录配置

use crate::app::components::system_settings_common::{
    settings_close_button, settings_muted_text_style, settings_panel_style,
    settings_text_input_style,
};
use crate::app::{App, Message, message};
use iced::widget::scrollable::{Direction, Scrollbar};
use iced::widget::{button, column, container, row, scrollable, text, text_input};
use iced::{Alignment, Background, Border, Color, Element, Length, Shadow, Vector};

/// 系统设置标签页枚举
///
/// 定义系统设置面板中所有可用的配置分类标签页。
/// 每个变体对应一个独立的配置模块，用户可通过点击标签页切换查看。
///
/// # 变体说明
///
/// - `General` - 常规设置，包含应用基础配置项
/// - `DialogueFlow` - 对话流设置，控制对话流程行为
/// - `Editor` - 编辑器设置，配置代码编辑器选项
/// - `Projects` - 项目设置，管理项目相关配置
/// - `Providers` - 提供商设置，配置 AI 服务提供商
/// - `Models` - 模型设置，管理可用 AI 模型
/// - `EmbeddingRoutes` - 嵌入路由设置，管理嵌入模型路由规则
/// - `ModelRoutes` - 模型路由设置，配置模式到模型的映射
/// - `QueryClassification` - 查询分类设置，配置匹配规则与分类目标
/// - `GoalLoop` - 目标循环设置，配置自治目标执行节奏与投递目标
/// - `Heartbeat` - 心跳设置，配置心跳检测参数
/// - `Cron` - Cron 设置，配置定时任务
/// - `Sop` - SOP 设置，配置 SOP 目录和执行策略
/// - `Scheduler` - 调度器设置，配置任务调度策略
/// - `Agents` - Agents 设置，配置 researcher / coder / reviewer 等委托代理
/// - `AgentsIpc` - Agents IPC 设置，配置代理间通信
/// - `Coordination` - 协调设置，配置多代理协调策略
/// - `Reliability` - 可靠性设置，配置容错与恢复机制
/// - `Channels` - 通道设置，配置 CLI 与外部消息通道
/// - `Memory` - 记忆设置，配置记忆后端和向量检索参数
/// - `Runtime` - 运行时设置，配置执行环境与隔离选项
/// - `Autonomy` - 自治设置，配置代理自治能力
/// - `Security` - 安全设置，配置安全策略
/// - `Observability` - 可观测性设置，配置日志与监控
/// - `Storage` - 存储设置，配置持久化存储 provider 与连接参数
/// - `Proxy` - 代理设置，配置网络代理
/// - `Tunnel` - 隧道设置，配置网关公网暴露 provider
/// - `Composio` - Composio 设置，配置 OAuth 工具集成
/// - `Skills` - 技能设置，配置代理可用技能
/// - `Hooks` - 钩子设置，配置运行时 hooks 与内置钩子
/// - `Research` - 研究设置，配置研究功能
/// - `WebSearch` - Web 搜索设置，配置搜索 provider、凭据与请求参数
/// - `HttpRequest` - HTTP 请求设置，配置 http_request 工具白名单与网络限制
/// - `Browser` - 浏览器设置，配置 browser/browser_open 工具与后端
/// - `Multimodal` - 多模态设置，配置图像输入限制与远程抓取策略
/// - `HttpRequest` - HTTP 请求设置，配置 http_request 工具白名单与请求限制
/// - `Transcription` - 转录设置，配置语音转录功能
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SystemTab {
    /// 常规设置标签页
    General,
    /// 对话流设置标签页
    DialogueFlow,
    /// 编辑器设置标签页
    Editor,
    /// 项目设置标签页
    Projects,
    /// 提供商设置标签页
    Providers,
    /// 模型设置标签页
    Models,
    /// 嵌入路由设置标签页
    EmbeddingRoutes,
    /// 模型路由设置标签页
    ModelRoutes,
    /// 查询分类设置标签页
    QueryClassification,
    /// 目标循环配置标签页
    GoalLoop,
    /// 心跳配置标签页
    Heartbeat,
    /// 定时任务配置标签页
    Cron,
    /// 标准流程配置标签页
    Sop,
    /// 调度配置标签页
    Scheduler,
    /// 委托代理配置标签页
    Agents,
    /// 代理通信配置标签页
    AgentsIpc,
    /// 协调配置标签页
    Coordination,
    /// 可靠性配置标签页
    Reliability,
    /// 通道配置标签页
    Channels,
    /// 记忆配置标签页
    Memory,
    /// 运行时配置标签页
    Runtime,
    /// 自治配置标签页
    Autonomy,
    /// 安全配置标签页
    Security,
    /// 客户端网关连接标签页
    GatewayClient,
    /// 网关配置标签页
    Gateway,
    /// 可观测性配置标签页
    Observability,
    /// 存储配置标签页
    Storage,
    /// 代理配置标签页
    Proxy,
    /// 隧道配置标签页
    Tunnel,
    /// Composio 集成配置标签页
    Composio,
    /// 技能配置标签页
    Skills,
    /// 钩子配置标签页
    Hooks,
    /// 研究配置标签页
    Research,
    /// 网页搜索配置标签页
    WebSearch,
    /// 网络请求配置标签页
    HttpRequest,
    /// 浏览器配置标签页
    Browser,
    /// 多模态配置标签页
    Multimodal,
    /// 转录配置标签页
    Transcription,
}

impl SystemTab {
    /// 返回所有系统设置标签页的数组
    ///
    /// # 返回值
    ///
    /// 包含所有 `SystemTab` 变体的固定大小数组，按显示顺序排列。
    ///
    /// # 示例
    ///
    /// ```ignore
    /// let all_tabs = SystemTab::all();
    /// for tab in all_tabs.iter() {
    ///     println!("{}", tab);
    /// }
    /// ```
    fn all() -> [SystemTab; 38] {
        [
            SystemTab::General,
            SystemTab::DialogueFlow,
            SystemTab::Editor,
            SystemTab::Projects,
            SystemTab::Providers,
            SystemTab::Models,
            SystemTab::EmbeddingRoutes,
            SystemTab::Memory,
            SystemTab::ModelRoutes,
            SystemTab::QueryClassification,
            SystemTab::GoalLoop,
            SystemTab::Heartbeat,
            SystemTab::Cron,
            SystemTab::Sop,
            SystemTab::Scheduler,
            SystemTab::Agents,
            SystemTab::AgentsIpc,
            SystemTab::Coordination,
            SystemTab::Reliability,
            SystemTab::Channels,
            SystemTab::Runtime,
            SystemTab::Autonomy,
            SystemTab::Security,
            SystemTab::GatewayClient,
            SystemTab::Gateway,
            SystemTab::Observability,
            SystemTab::Storage,
            SystemTab::Proxy,
            SystemTab::Tunnel,
            SystemTab::Composio,
            SystemTab::Skills,
            SystemTab::Hooks,
            SystemTab::Research,
            SystemTab::WebSearch,
            SystemTab::HttpRequest,
            SystemTab::Browser,
            SystemTab::Multimodal,
            SystemTab::Transcription,
        ]
    }
}

impl std::fmt::Display for SystemTab {
    /// 格式化标签页为中文名称
    ///
    /// 将每个标签页变体转换为对应的中文显示名称，
    /// 用于在 UI 界面中渲染标签页标题。
    ///
    /// # 参数
    ///
    /// - `f` - 格式化器引用
    ///
    /// # 返回值
    ///
    /// 返回 `fmt::Result`，表示格式化操作的结果。
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SystemTab::General => write!(f, "常规设置"),
            SystemTab::DialogueFlow => write!(f, "对话流"),
            SystemTab::Editor => write!(f, "编辑器"),
            SystemTab::Projects => write!(f, "项目"),
            SystemTab::Providers => write!(f, "提供商"),
            SystemTab::Models => write!(f, "模型"),
            SystemTab::EmbeddingRoutes => write!(f, "嵌入路由"),
            SystemTab::ModelRoutes => write!(f, "模型路由"),
            SystemTab::QueryClassification => write!(f, "查询分类"),
            SystemTab::GoalLoop => write!(f, "目标循环配置"),
            SystemTab::Heartbeat => write!(f, "心跳配置"),
            SystemTab::Cron => write!(f, "定时任务配置"),
            SystemTab::Sop => write!(f, "标准流程配置"),
            SystemTab::Scheduler => write!(f, "调度配置"),
            SystemTab::Agents => write!(f, "委托代理配置"),
            SystemTab::AgentsIpc => write!(f, "代理通信配置"),
            SystemTab::Coordination => write!(f, "协调配置"),
            SystemTab::Reliability => write!(f, "可靠性配置"),
            SystemTab::Channels => write!(f, "通道配置"),
            SystemTab::Memory => write!(f, "记忆配置"),
            SystemTab::Runtime => write!(f, "运行时配置"),
            SystemTab::Autonomy => write!(f, "自治配置"),
            SystemTab::Security => write!(f, "安全配置"),
            SystemTab::GatewayClient => write!(f, "客户端网关"),
            SystemTab::Gateway => write!(f, "服务端网关"),
            SystemTab::Observability => write!(f, "可观测性配置"),
            SystemTab::Storage => write!(f, "存储配置"),
            SystemTab::Proxy => write!(f, "代理配置"),
            SystemTab::Tunnel => write!(f, "隧道配置"),
            SystemTab::Composio => write!(f, "Composio 集成配置"),
            SystemTab::Skills => write!(f, "技能配置"),
            SystemTab::Hooks => write!(f, "钩子配置"),
            SystemTab::Research => write!(f, "研究配置"),
            SystemTab::WebSearch => write!(f, "网页搜索配置"),
            SystemTab::HttpRequest => write!(f, "网络请求配置"),
            SystemTab::Browser => write!(f, "浏览器配置"),
            SystemTab::Multimodal => write!(f, "多模态配置"),
            SystemTab::Transcription => write!(f, "转录配置"),
        }
    }
}

fn system_tab_search_text(tab: SystemTab) -> &'static str {
    match tab {
        SystemTab::General => "常规设置 general app basic 基础",
        SystemTab::DialogueFlow => "对话流 dialogue flow permission 对话 权限",
        SystemTab::Editor => "编辑器 editor code 编辑",
        SystemTab::Projects => "项目 projects workspace 工作区",
        SystemTab::Providers => "提供商 providers api key model 模型",
        SystemTab::Models => "模型 models llm ai provider",
        SystemTab::EmbeddingRoutes => "嵌入路由 embedding routes vector model 模型",
        SystemTab::ModelRoutes => "模型路由 model routes routing 模型",
        SystemTab::QueryClassification => "查询分类 query classification rules 规则",
        SystemTab::GoalLoop => "目标循环配置 goal loop run automation 运行 自动化",
        SystemTab::Heartbeat => "心跳配置 heartbeat monitor follow up 自动化",
        SystemTab::Cron => "定时任务配置 cron schedule task 自动化",
        SystemTab::Sop => "标准流程配置 sop workflow run 运行",
        SystemTab::Scheduler => "调度配置 scheduler tasks run 运行",
        SystemTab::Agents => "委托代理配置 agents subagent model tools 模型 工具",
        SystemTab::AgentsIpc => "代理通信配置 agents ipc communication",
        SystemTab::Coordination => "协调配置 coordination agents",
        SystemTab::Reliability => "可靠性配置 reliability retry run 运行",
        SystemTab::Channels => "通道配置 channels cli message",
        SystemTab::Memory => "记忆配置 memory embedding vector 模型",
        SystemTab::Runtime => "运行时配置 runtime native docker wasm run 运行",
        SystemTab::Autonomy => "自治配置 autonomy agent run 运行",
        SystemTab::Security => "安全配置 security policy permission",
        SystemTab::GatewayClient => "客户端网关 gateway client connection",
        SystemTab::Gateway => "服务端网关 gateway server api",
        SystemTab::Observability => "可观测性配置 observability log monitor",
        SystemTab::Storage => "存储配置 storage database",
        SystemTab::Proxy => "代理配置 proxy network",
        SystemTab::Tunnel => "隧道配置 tunnel public gateway",
        SystemTab::Composio => "composio 集成配置 oauth tools 工具",
        SystemTab::Skills => "技能配置 skills tools plugins 工具",
        SystemTab::Hooks => "钩子配置 hooks tool command 工具",
        SystemTab::Research => "研究配置 research search tool 工具",
        SystemTab::WebSearch => "网页搜索配置 web search brave tool 工具 搜索",
        SystemTab::HttpRequest => "网络请求配置 http request tool allowlist 工具",
        SystemTab::Browser => "浏览器配置 browser computer use tool 工具",
        SystemTab::Multimodal => "多模态配置 multimodal image vision model 模型",
        SystemTab::Transcription => "转录配置 transcription audio speech model 模型",
    }
}

fn system_tab_matches_query(tab: SystemTab, query: &str) -> bool {
    let query = query.trim();
    if query.is_empty() {
        return true;
    }

    let needle = query.to_lowercase();
    tab.to_string().to_lowercase().contains(&needle)
        || system_tab_search_text(tab).to_lowercase().contains(&needle)
}

fn active_tab_help_modal_open(app: &App, active_tab: SystemTab) -> bool {
    crate::app::components::system_settings_help::help_open_for_tab(
        app.system_settings_help_tab,
        active_tab,
    ) || match active_tab {
        SystemTab::Heartbeat => app.heartbeat_settings.show_help_modal,
        SystemTab::Cron => app.cron_settings.show_help_modal,
        SystemTab::Scheduler => app.scheduler_settings.show_help_modal,
        SystemTab::AgentsIpc => app.agents_ipc_settings.show_help_modal,
        SystemTab::Coordination => app.coordination_settings.show_help_modal,
        SystemTab::Reliability => app.reliability_settings.show_help_modal,
        SystemTab::Autonomy => app.autonomy_settings.show_help_modal,
        SystemTab::Security => app.security_settings.show_help_modal,
        SystemTab::GatewayClient => app.gateway_client_settings.show_help_modal,
        SystemTab::Gateway => app.gateway_settings.show_help_modal,
        SystemTab::Observability => app.observability_settings.show_help_modal,
        SystemTab::Proxy => app.proxy_settings.show_help_modal,
        SystemTab::Skills => app.skills_settings.show_help_modal,
        SystemTab::Research => app.research_settings.show_help_modal,
        SystemTab::WebSearch => app.web_search_settings.show_help_modal,
        SystemTab::Transcription => app.transcription_settings.show_help_modal,
        _ => false,
    }
}

/// 渲染系统设置面板视图
///
/// 构建并返回系统设置模态对话框的 UI 元素。该视图包含：
///
/// - 半透明遮罩层（点击可关闭设置面板）
/// - 模态对话框容器
/// - 左侧标签页导航栏
/// - 右侧配置内容区域
/// - 关闭按钮
///
/// # 参数
///
/// - `app` - 应用状态引用，包含当前激活的标签页和显示状态等信息
///
/// # 返回值
///
/// 返回 `Element<'_, Message>` 类型的 UI 元素，可直接集成到 Iced 应用中。
///
/// # 行为说明
///
/// 1. 如果 `app.show_system_settings` 为 `false`，返回空容器
/// 2. 根据当前激活标签页加载对应的配置模块视图
/// 3. 对于 Providers 和 Models 标签页，额外添加浮层支持
/// 4. 点击遮罩层时触发关闭设置面板的消息
///
/// # 布局结构
///
/// ```text
/// ┌─────────────────────────────────────────────────┐
/// │ 半透明遮罩 (点击关闭)                              │
/// │  ┌───────────────────────────────────────────┐  │
/// │  │ 系统配置                          [×]     │  │
/// │  ├─────────────┬─────────────────────────────┤  │
/// │  │ 标签页列表   │ 配置内容区域                 │  │
/// │  │ · 常规设置  │                             │  │
/// │  │ · 对话流    │ (根据选中标签页显示)          │  │
/// │  │ · 编辑器    │                             │  │
/// │  │ · ...      │                             │  │
/// │  │            │                             │  │
/// │  └─────────────┴─────────────────────────────┘  │
/// └─────────────────────────────────────────────────┘
/// ```
///
/// # 示例
///
/// ```ignore
/// let settings_view = view(&app);
/// // 将 settings_view 添加到应用的视图层级中
/// ```
pub fn view(app: &App) -> Element<'_, Message> {
    // 如果设置面板未显示，返回空容器
    if !app.show_system_settings {
        return container(column![]).into();
    }

    // 获取当前激活的标签页
    let active_tab = app.system_settings_tab;
    let filtered_tabs = SystemTab::all()
        .iter()
        .copied()
        .filter(|tab| system_tab_matches_query(*tab, &app.system_settings_query))
        .collect::<Vec<_>>();
    let query_input = text_input("搜索配置", &app.system_settings_query)
        .on_input(|value| Message::Settings(message::SettingsMessage::SystemTabQueryChanged(value)))
        .padding([8, 10])
        .size(13)
        .style(settings_text_input_style);

    // 构建左侧标签页导航栏
    // 为每个标签页创建按钮，根据激活状态和悬停状态应用不同样式
    let tabs_bar: Element<'_, Message> = if filtered_tabs.is_empty() {
        container(text("没有匹配的配置").size(12).style(settings_muted_text_style))
            .padding([10, 12])
            .width(Length::Fill)
            .into()
    } else {
        column(
            filtered_tabs
                .iter()
                .map(|tab| {
                    // 判断当前标签页是否处于激活状态
                    let is_active = *tab == active_tab;
                    // 创建标签页文本标签
                    let label = text(tab.to_string()).size(13);

                    // 构建标签页按钮
                    let btn = button(container(label).width(Length::Fill).padding([8, 12]))
                        .width(Length::Fill)
                        .on_press(Message::Settings(message::SettingsMessage::SystemTabSelected(
                            *tab,
                        )))
                        .style(move |theme: &iced::Theme, status| {
                            let palette = theme.extended_palette();
                            let is_dark = theme.palette().background.r
                                + theme.palette().background.g
                                + theme.palette().background.b
                                < 1.5;
                            let bg = if is_active {
                                Some(Background::Color(if is_dark {
                                    theme.palette().primary.scale_alpha(0.18)
                                } else {
                                    theme.palette().primary.scale_alpha(0.08)
                                }))
                            } else {
                                match status {
                                    iced::widget::button::Status::Hovered => {
                                        Some(Background::Color(if is_dark {
                                            palette.background.weak.color.scale_alpha(0.72)
                                        } else {
                                            Color::WHITE.scale_alpha(0.78)
                                        }))
                                    }
                                    iced::widget::button::Status::Pressed => {
                                        Some(Background::Color(if is_dark {
                                            palette.background.strong.color.scale_alpha(0.86)
                                        } else {
                                            palette.background.weak.color.scale_alpha(0.92)
                                        }))
                                    }
                                    _ => None,
                                }
                            };

                            iced::widget::button::Style {
                                background: bg,
                                text_color: if is_active {
                                    theme.palette().primary.scale_alpha(0.96)
                                } else {
                                    theme.palette().text.scale_alpha(0.92)
                                },
                                border: Border {
                                    radius: 14.0.into(),
                                    width: 1.0,
                                    color: if is_active {
                                        theme.palette().primary.scale_alpha(0.24)
                                    } else if is_dark {
                                        palette.background.strong.color.scale_alpha(0.78)
                                    } else {
                                        Color::from_rgba8(15, 23, 42, 0.06)
                                    },
                                },
                                shadow: if is_active {
                                    Shadow {
                                        color: Color::BLACK.scale_alpha(if is_dark {
                                            0.18
                                        } else {
                                            0.06
                                        }),
                                        offset: Vector::new(0.0, 6.0),
                                        blur_radius: 14.0,
                                    }
                                } else {
                                    Shadow::default()
                                },
                                ..Default::default()
                            }
                        });

                    container(btn).width(Length::Fill).into()
                })
                .collect::<Vec<_>>(),
        )
        .spacing(4)
        .padding([2, 2])
        .into()
    };

    // 根据当前激活的标签页加载对应的配置内容视图
    let content: Element<'_, Message> = match active_tab {
        SystemTab::General => crate::app::components::system_settings_general::view(app),
        SystemTab::DialogueFlow => crate::app::components::system_settings_dialogue_flow::view(app),
        SystemTab::Editor => crate::app::components::system_settings_editor::view(app),
        SystemTab::Projects => crate::app::components::system_settings_projects::view(app),
        SystemTab::Providers => {
            crate::app::components::system_settings_providers::connected::view(app)
        }
        SystemTab::Models => crate::app::components::system_settings_models::main_view(app),
        SystemTab::EmbeddingRoutes => {
            crate::app::components::system_settings_embedding_routes::view(app)
        }
        SystemTab::ModelRoutes => crate::app::components::system_settings_model_routes::view(app),
        SystemTab::QueryClassification => {
            crate::app::components::system_settings_query_classification::view(app)
        }
        SystemTab::GoalLoop => crate::app::components::system_settings_goal_loop::view(app),
        SystemTab::Heartbeat => crate::app::components::system_settings_heartbeat::view(app),
        SystemTab::Cron => crate::app::components::system_settings_cron::view(app),
        SystemTab::Sop => crate::app::components::system_settings_sop::view(app),
        SystemTab::Scheduler => crate::app::components::system_settings_scheduler::view(app),
        SystemTab::Agents => crate::app::components::system_settings_agents::view(app),
        SystemTab::AgentsIpc => crate::app::components::system_settings_agents_ipc::view(app),
        SystemTab::Coordination => crate::app::components::system_settings_coordination::view(app),
        SystemTab::Reliability => crate::app::components::system_settings_reliability::view(app),
        SystemTab::Channels => crate::app::components::system_settings_channels::view(app),
        SystemTab::Memory => crate::app::components::system_settings_memory::view(app),
        SystemTab::Runtime => crate::app::components::system_settings_runtime::view(app),
        SystemTab::Autonomy => crate::app::components::system_settings_autonomy::view(app),
        SystemTab::Security => crate::app::components::system_settings_security::view(app),
        SystemTab::GatewayClient => {
            crate::app::components::system_settings_gateway_client::view(app)
        }
        SystemTab::Gateway => crate::app::components::system_settings_gateway::view(app),
        SystemTab::Observability => {
            crate::app::components::system_settings_observability::view(app)
        }
        SystemTab::Storage => crate::app::components::system_settings_storage::view(app),
        SystemTab::Proxy => crate::app::components::system_settings_proxy::view(app),
        SystemTab::Tunnel => crate::app::components::system_settings_tunnel::view(app),
        SystemTab::Composio => crate::app::components::system_settings_composio::view(app),
        SystemTab::Skills => crate::app::components::system_settings_skills::view(app),
        SystemTab::Hooks => crate::app::components::system_settings_hooks::view(app),
        SystemTab::Research => crate::app::components::system_settings_research::view(app),
        SystemTab::WebSearch => crate::app::components::system_settings_web_search::view(app),
        SystemTab::HttpRequest => crate::app::components::system_settings_http_request::view(app),
        SystemTab::Browser => crate::app::components::system_settings_browser::view(app),
        SystemTab::Multimodal => crate::app::components::system_settings_multimodal::view(app),
        SystemTab::Transcription => {
            crate::app::components::system_settings_transcription::view(app)
        }
    };

    // 创建关闭按钮（右上角 × 符号）
    let close_btn =
        settings_close_button(Message::View(message::ViewMessage::ToggleSystemSettings));

    // 构建全屏设置面板主体内容
    // 面板占满内容区，不再以小尺寸模态框居中显示
    let modal_content = container(
        column![
            container(
                row![
                    column![
                        text("配置").size(22),
                        text("管理运行、工具、模型与连接设置。")
                            .size(12)
                            .style(settings_muted_text_style),
                    ]
                    .spacing(4),
                    container(text(" ")).width(Length::Fill),
                    close_btn,
                ]
                .align_y(Alignment::Start)
            )
            .padding(iced::Padding::default().top(18).right(18).bottom(14).left(18)),
            row![
                container(
                    column![
                        text("分类").size(11).style(settings_muted_text_style),
                        query_input,
                        scrollable(
                            container(tabs_bar).padding(iced::Padding::default().right(10.0))
                        )
                        .direction(Direction::Vertical(Scrollbar::new().width(4).scroller_width(4)))
                        .height(Length::Fill)
                    ]
                    .spacing(8)
                    .height(Length::Fill),
                )
                .width(Length::Fixed(272.0))
                .height(Length::Fill)
                .padding([16, 14])
                .style(|theme: &iced::Theme| {
                    let extended = theme.extended_palette();
                    let is_dark = theme.palette().background.r
                        + theme.palette().background.g
                        + theme.palette().background.b
                        < 1.5;
                    iced::widget::container::Style {
                        background: Some(Background::Color(if is_dark {
                            extended.background.base.color.scale_alpha(0.66)
                        } else {
                            Color::WHITE.scale_alpha(0.72)
                        })),
                        border: Border {
                            radius: 20.0.into(),
                            width: 1.0,
                            color: if is_dark {
                                extended.background.strong.color.scale_alpha(0.82)
                            } else {
                                Color::from_rgba8(15, 23, 42, 0.08)
                            },
                        },
                        shadow: Shadow {
                            color: Color::BLACK.scale_alpha(if is_dark { 0.18 } else { 0.08 }),
                            offset: Vector::new(0.0, 10.0),
                            blur_radius: 24.0,
                        },
                        snap: false,
                        ..Default::default()
                    }
                }),
                container({
                    let mut content_with_help = column![].spacing(12).width(Length::Fill);
                    if let Some(help_bar) =
                        crate::app::components::system_settings_help::help_button_bar(active_tab)
                    {
                        content_with_help = content_with_help.push(help_bar);
                    }
                    content_with_help = content_with_help.push(content);

                    let content_panel: Element<'_, Message> = if active_tab == SystemTab::Agents
                        || active_tab_help_modal_open(app, active_tab)
                    {
                        container(content_with_help)
                            .padding([0, 0])
                            .width(Length::Fill)
                            .height(Length::Fill)
                            .into()
                    } else {
                        scrollable(
                            container(content_with_help)
                                .padding(iced::Padding::default().right(10.0)),
                        )
                        .direction(Direction::Vertical(Scrollbar::new().width(4).scroller_width(4)))
                        .into()
                    };
                    container(content_panel)
                        .padding([18, 20])
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .style(settings_panel_style)
                })
                .width(Length::Fill)
                .height(Length::Fill)
                .padding(iced::Padding::default().top(0).right(20).bottom(20).left(0))
            ]
            .spacing(16)
            .height(Length::Fill)
        ]
        .spacing(0)
        .height(Length::Fill),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .style(|theme: &iced::Theme| {
        let palette = theme.extended_palette();
        let is_dark = theme.palette().background.r
            + theme.palette().background.g
            + theme.palette().background.b
            < 1.5;
        iced::widget::container::Style {
            text_color: Some(theme.palette().text),
            background: Some(Background::Color(if is_dark {
                palette.background.weak.color.scale_alpha(0.28)
            } else {
                Color::from_rgba8(248, 250, 252, 0.96)
            })),
            border: Border::default(),
            snap: false,
            ..Default::default()
        }
    });

    // 将模态内容转换为 Element
    let mut dialog: Element<'_, Message> = modal_content.into();

    // 为 Providers 标签页添加额外的浮层视图
    // 这些浮层用于显示自定义提供商、目录、连接等子对话框
    if app.system_settings_tab == SystemTab::Providers {
        dialog = crate::app::components::system_settings_providers::custom_provider::view_overlays(
            app, dialog,
        );
        dialog =
            crate::app::components::system_settings_providers::catalog::view_overlays(app, dialog);
        dialog =
            crate::app::components::system_settings_providers::connect::view_overlays(app, dialog);
        dialog =
            crate::app::components::system_settings_providers::custom_model_modal::view_overlays(
                app, dialog,
            );
    }

    // 为 Models 标签页添加额外的浮层视图
    if app.system_settings_tab == SystemTab::Models {
        dialog = crate::app::components::system_settings_models::view_overlays(app, dialog);
    }

    if app.system_settings_tab == SystemTab::Heartbeat {
        dialog = crate::app::components::system_settings_heartbeat::view_overlays(app, dialog);
    }

    if app.system_settings_tab == SystemTab::Cron {
        dialog = crate::app::components::system_settings_cron::view_overlays(app, dialog);
    }

    if app.system_settings_tab == SystemTab::Scheduler {
        dialog = crate::app::components::system_settings_scheduler::view_overlays(app, dialog);
    }

    if app.system_settings_tab == SystemTab::AgentsIpc {
        dialog = crate::app::components::system_settings_agents_ipc::view_overlays(app, dialog);
    }

    if app.system_settings_tab == SystemTab::Coordination {
        dialog = crate::app::components::system_settings_coordination::view_overlays(app, dialog);
    }

    if app.system_settings_tab == SystemTab::Reliability {
        dialog = crate::app::components::system_settings_reliability::view_overlays(app, dialog);
    }

    if app.system_settings_tab == SystemTab::Autonomy {
        dialog = crate::app::components::system_settings_autonomy::view_overlays(app, dialog);
    }

    if app.system_settings_tab == SystemTab::Security {
        dialog = crate::app::components::system_settings_security::view_overlays(app, dialog);
    }

    if app.system_settings_tab == SystemTab::GatewayClient {
        dialog = crate::app::components::system_settings_gateway_client::view_overlays(app, dialog);
    }

    if app.system_settings_tab == SystemTab::Gateway {
        dialog = crate::app::components::system_settings_gateway::view_overlays(app, dialog);
    }

    if app.system_settings_tab == SystemTab::Observability {
        dialog = crate::app::components::system_settings_observability::view_overlays(app, dialog);
    }

    if app.system_settings_tab == SystemTab::Proxy {
        dialog = crate::app::components::system_settings_proxy::view_overlays(app, dialog);
    }

    if app.system_settings_tab == SystemTab::Skills {
        dialog = crate::app::components::system_settings_skills::view_overlays(app, dialog);
    }

    if app.system_settings_tab == SystemTab::Research {
        dialog = crate::app::components::system_settings_research::view_overlays(app, dialog);
    }

    if app.system_settings_tab == SystemTab::WebSearch {
        dialog = crate::app::components::system_settings_web_search::view_overlays(app, dialog);
    }

    if app.system_settings_tab == SystemTab::Transcription {
        dialog = crate::app::components::system_settings_transcription::view_overlays(app, dialog);
    }

    dialog = crate::app::components::system_settings_help::with_help_modal(
        app,
        dialog,
        app.system_settings_tab,
        app.system_settings_help_tab,
    );

    dialog
}
