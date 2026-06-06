//! Redis 工具详情模块，负责连接、命令、键空间和运行时信息面板。
//!
//! 本模块只负责视图组合与样式适配，不持有业务状态，也不扩大外部能力边界。

use crate::app::components::system_settings_common::{
    primary_action_btn_style, rounded_action_btn_style, settings_divider,
    settings_muted_text_style, settings_panel, settings_panel_style, settings_value_badge,
};
use crate::app::message::RedisToolMessage;
use crate::app::state::{RedisDetailTab, RedisRuntimeOverview};
use crate::app::{App, Message};
use iced::widget::{Space, button, column, container, row, text};
use iced::{Alignment, Element, Length};

use super::common::{
    advanced_execution_note, build_detail_action_button, connection_mode_label, current_load_count,
    enabled_feature_summary, masked_connection_preview, overview_row,
};

mod analysis;
mod command;
mod connection;
mod keys;
mod runtime;

use analysis::{build_key_analysis_empty_state, build_key_analysis_panel};
use command::build_command_panel;
use connection::{build_active_tab, build_tab_bar};
use keys::build_key_tree_panel;
use runtime::{
    build_info_panel, build_keyspace_panel, build_runtime_empty_state, build_runtime_overview_cards,
};

/// 构建对应界面片段。
///
/// # 参数
/// - `app`: 当前视图构建所需的状态、配置或消息。
/// - `compact`: 当前视图构建所需的状态、配置或消息。
///
/// # 返回
/// 返回可直接嵌入父级视图的 Iced 控件或样式闭包。
///
/// # 错误
/// 此函数不返回 `Result`；不可用状态会通过空视图、禁用控件或回退文案表达。
pub(super) fn build_detail_panel<'a>(app: &'a App, compact: bool) -> Element<'a, Message> {
    let is_busy = app.redis_tool.is_gateway_loading();
    let title = if app.redis_tool.selected_connection_id.is_none() {
        "未选择连接".to_string()
    } else if app.redis_tool.draft.name.trim().is_empty() {
        "连接详情".to_string()
    } else {
        app.redis_tool.draft.name.clone()
    };

    let subtitle = if app.redis_tool.selected_connection_id.is_none() {
        "从左侧选择已保存连接，或打开弹窗新建一个 Redis 连接。".to_string()
    } else {
        "已保存连接可在右侧标签切换键树、内容分析、命令、连接配置和运行态信息。".to_string()
    };

    let actions = row![
        build_detail_action_button(
            if app.redis_tool.selected_connection_id.is_some() {
                "编辑配置"
            } else {
                "新建连接"
            },
            if app.redis_tool.selected_connection_id.is_some() {
                Message::RedisTool(RedisToolMessage::OpenConnectionModal)
            } else {
                Message::RedisTool(RedisToolMessage::NewConnection)
            },
            true,
            !is_busy,
        ),
        if app.redis_tool.selected_connection_id.is_some() {
            build_detail_action_button(
                "复制URI",
                Message::RedisTool(RedisToolMessage::CopySelectedUri),
                false,
                !is_busy,
            )
        } else {
            Space::new().width(Length::Shrink).into()
        },
        if app.redis_tool.selected_connection_id.is_some() {
            build_detail_action_button(
                "测试连接",
                Message::RedisTool(RedisToolMessage::TestSelected),
                false,
                !is_busy,
            )
        } else {
            Space::new().width(Length::Shrink).into()
        },
        if app.redis_tool.selected_connection_id.is_some() {
            build_detail_action_button(
                "刷新信息",
                Message::RedisTool(RedisToolMessage::RefreshSelectedRuntime),
                false,
                !is_busy,
            )
        } else {
            Space::new().width(Length::Shrink).into()
        },
        if app.redis_tool.selected_connection_id.is_some() {
            build_detail_action_button(
                "删除连接",
                Message::RedisTool(RedisToolMessage::DeleteSelected),
                false,
                !is_busy,
            )
        } else {
            Space::new().width(Length::Shrink).into()
        },
    ]
    .spacing(10)
    .align_y(Alignment::Center);

    column![
        container(
            row![
                column![
                    text(title).size(18),
                    text(subtitle).size(12).style(settings_muted_text_style),
                ]
                .spacing(4),
                Space::new().width(Length::Fill),
                actions,
            ]
            .spacing(16)
            .align_y(Alignment::Center),
        )
        .padding([18, 20])
        .style(settings_panel_style),
        build_detail_workspace(app, compact, is_busy),
    ]
    .spacing(12)
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

fn build_connection_workspace<'a>(
    app: &'a App,
    compact: bool,
    is_busy: bool,
) -> Element<'a, Message> {
    let action_label = if app.redis_tool.selected_connection_id.is_some() {
        "编辑配置"
    } else {
        "新建连接"
    };
    let action_message = if app.redis_tool.selected_connection_id.is_some() {
        RedisToolMessage::OpenConnectionModal
    } else {
        RedisToolMessage::NewConnection
    };
    let mut workspace_content = column![
        row![
            column![
                text("工作区预览").size(14),
                text("连接参数从弹窗维护，当前页只保留摘要与运行入口。")
                    .size(12)
                    .style(settings_muted_text_style),
            ]
            .spacing(4),
            Space::new().width(Length::Fill),
            settings_value_badge(format!("默认加载 {} 项", current_load_count(app))),
            button(text(action_label).size(13))
                .on_press_maybe((!is_busy).then_some(Message::RedisTool(action_message)))
                .padding([10, 14])
                .style(primary_action_btn_style),
        ]
        .align_y(Alignment::Center)
        .spacing(12),
        settings_divider(),
        overview_row("连接预览", masked_connection_preview(app)),
        overview_row(
            "当前模式",
            if app.redis_tool.draft_is_new { "新建连接" } else { "已保存连接" },
        ),
        overview_row("连接拓扑", connection_mode_label(&app.redis_tool.draft)),
        overview_row("启用特性", enabled_feature_summary(&app.redis_tool.draft)),
        overview_row(
            "运行态",
            if app.redis_tool.has_runtime_for_selected() {
                "已加载"
            } else if app.redis_tool.selected_connection_id.is_some() {
                "待加载"
            } else {
                "未选择连接"
            },
        ),
        overview_row("默认读取策略", "后续键列表默认按加载数量限制首屏读取。"),
    ]
    .spacing(12);

    if let Some(note) = advanced_execution_note(&app.redis_tool.draft) {
        workspace_content = workspace_content.push(overview_row("执行限制", note));
    }

    let workspace = settings_panel(workspace_content);

    container(workspace)
        .width(Length::Fill)
        .height(if compact { Length::Shrink } else { Length::Fill })
        .into()
}

pub(super) fn build_connection_form_panel<'a>(
    app: &'a App,
    compact: bool,
    is_busy: bool,
) -> Element<'a, Message> {
    settings_panel(
        column![
            container(
                column![
                    row![
                        text("连接配置").size(14),
                        Space::new().width(Length::Fill),
                        settings_value_badge(connection_mode_label(&app.redis_tool.draft)),
                    ]
                    .align_y(Alignment::Center),
                    text("通过页签维护基础参数与高级连接能力。当前仅 SSH 隧道仍未接入测试链路。")
                        .size(12)
                        .style(settings_muted_text_style),
                ]
                .spacing(4),
            )
            .padding([4, 0]),
            settings_divider(),
            build_tab_bar(app, is_busy),
            settings_divider(),
            build_active_tab(app, compact),
        ]
        .spacing(0),
    )
    .into()
}

fn build_detail_workspace<'a>(app: &'a App, compact: bool, is_busy: bool) -> Element<'a, Message> {
    let selected_id = app.redis_tool.selected_connection_id.as_deref();
    let runtime = app
        .redis_tool
        .runtime_overview
        .as_ref()
        .filter(|overview| selected_id == Some(overview.connection_id.as_str()));
    let connection_label = if let Some(current_runtime) = runtime {
        current_runtime.connection_label.clone()
    } else if app.redis_tool.draft.name.trim().is_empty() {
        masked_connection_preview(app)
    } else {
        app.redis_tool.draft.name.clone()
    };

    column![
        row![
            text("连接工作区").size(14),
            Space::new().width(Length::Fill),
            settings_value_badge(connection_label),
            settings_value_badge(app.redis_tool.detail_tab.title()),
        ]
        .spacing(12)
        .align_y(Alignment::Center),
        settings_divider(),
        build_detail_tab_bar(app, is_busy),
        container(build_active_detail_tab(app, compact, is_busy, runtime))
            .width(Length::Fill)
            .height(Length::Fill),
    ]
    .spacing(12)
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

fn build_detail_tab_bar<'a>(app: &'a App, is_busy: bool) -> Element<'a, Message> {
    let mut tabs = row![].spacing(8).align_y(Alignment::Center);
    for tab in [
        RedisDetailTab::Keys,
        RedisDetailTab::Analysis,
        RedisDetailTab::Command,
        RedisDetailTab::Connection,
        RedisDetailTab::Overview,
        RedisDetailTab::Info,
    ] {
        let active = app.redis_tool.detail_tab == tab;
        let tab_button: Element<'a, Message> = if active {
            button(text(tab.title()).size(13))
                .on_press_maybe(
                    (!is_busy)
                        .then_some(Message::RedisTool(RedisToolMessage::DetailTabChanged(tab))),
                )
                .padding([8, 12])
                .style(primary_action_btn_style)
                .into()
        } else {
            button(text(tab.title()).size(13))
                .on_press_maybe(
                    (!is_busy)
                        .then_some(Message::RedisTool(RedisToolMessage::DetailTabChanged(tab))),
                )
                .padding([8, 12])
                .style(rounded_action_btn_style)
                .into()
        };
        tabs = tabs.push(tab_button);
    }

    container(tabs).padding([12, 0]).into()
}

fn build_active_detail_tab<'a>(
    app: &'a App,
    compact: bool,
    is_busy: bool,
    runtime: Option<&'a RedisRuntimeOverview>,
) -> Element<'a, Message> {
    match app.redis_tool.detail_tab {
        RedisDetailTab::Connection => build_connection_workspace(app, compact, is_busy),
        RedisDetailTab::Keys => build_key_tree_panel(app, is_busy),
        RedisDetailTab::Analysis => {
            if let Some(analysis) = app.redis_tool.key_analysis.as_ref().filter(|analysis| {
                app.redis_tool.selected_connection_id.as_deref()
                    == Some(analysis.connection_id.as_str())
                    && app.redis_tool.selected_key.as_deref() == Some(analysis.key.as_str())
            }) {
                build_key_analysis_panel(app, analysis, is_busy)
            } else if app.redis_tool.selected_key.is_some()
                && app.redis_tool.selected_connection_id.is_some()
            {
                build_key_analysis_empty_state(
                    app,
                    "内容分析尚未加载",
                    "点击“刷新内容”后，可查看当前 Key 的类型、TTL、内存占用和值预览。",
                    is_busy,
                )
            } else if app.redis_tool.selected_connection_id.is_some() {
                build_detail_hint_state(
                    "尚未选择 Key",
                    "先在“键树”标签中点选具体 Key，或使用“新增 Key”创建后自动进入详情。",
                )
            } else {
                build_detail_hint_state(
                    "尚未选择连接",
                    "内容分析依赖已保存连接；先在左侧选择连接或保存当前草稿。",
                )
            }
        }
        RedisDetailTab::Command => build_command_panel(app, is_busy),
        RedisDetailTab::Overview => {
            if let Some(runtime) = runtime {
                column![
                    build_runtime_overview_cards(runtime, compact),
                    build_keyspace_panel(runtime),
                ]
                .spacing(12)
                .into()
            } else if app.redis_tool.selected_connection_id.is_some() {
                build_runtime_empty_state(
                    app,
                    "运行态概览尚未加载",
                    "点击“刷新信息”后，即可查看服务器、内存和键值统计。",
                    is_busy,
                )
            } else {
                build_detail_hint_state(
                    "尚未选择连接",
                    "先从左侧选择一个已保存连接，或在“连接配置”标签中创建新连接。",
                )
            }
        }
        RedisDetailTab::Info => {
            if let Some(runtime) = runtime {
                build_info_panel(app, runtime)
            } else if app.redis_tool.selected_connection_id.is_some() {
                build_runtime_empty_state(
                    app,
                    "INFO 全量字段尚未加载",
                    "点击“刷新信息”后，即可查看完整 INFO 键值并在本地过滤。",
                    is_busy,
                )
            } else {
                build_detail_hint_state(
                    "尚未选择连接",
                    "INFO 标签仅对已保存连接生效；先选择连接或切回“连接配置”继续编辑。",
                )
            }
        }
    }
}

fn build_detail_hint_state<'a>(title: &'a str, description: &'a str) -> Element<'a, Message> {
    settings_panel(
        column![text(title).size(14), text(description).size(12).style(settings_muted_text_style),]
            .spacing(8),
    )
    .into()
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
