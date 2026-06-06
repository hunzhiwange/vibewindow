//! Redis 工具视图模块，负责连接列表、弹窗、状态徽标和表单控件。
//!
//! 本模块只负责视图组合与样式适配，不持有业务状态，也不扩大外部能力边界。

use crate::app::components::system_settings_common::{
    primary_action_btn_style, rounded_action_btn_style, settings_checkbox_style,
    settings_muted_text_style, settings_panel, settings_pick_list_menu_style,
    settings_pick_list_style, settings_text_input_style, settings_value_badge,
};
use crate::app::message::RedisToolMessage;
use crate::app::state::RedisKeyValueKind;
use crate::app::{App, Message};
use iced::widget::{
    Space, button, checkbox, column, container, pick_list, row, scrollable, text, text_input,
};
use iced::{Alignment, Element, Length};

use super::common::{
    form_row, history_page_label, history_table_header, history_table_row, modal_header,
    modal_shell, redis_scroll_direction,
};
use super::detail::build_connection_form_panel;

const SETTINGS_MODAL_WIDTH: f32 = 600.0;

/// 构建对应界面片段。
///
/// # 参数
/// - `app`: 当前视图构建所需的状态、配置或消息。
///
/// # 返回
/// 返回可直接嵌入父级视图的 Iced 控件或样式闭包。
///
/// # 错误
/// 此函数不返回 `Result`；不可用状态会通过空视图、禁用控件或回退文案表达。
pub(super) fn build_settings_modal<'a>(app: &'a App) -> Element<'a, Message> {
    let is_busy = app.redis_tool.is_gateway_loading();
    let count_stepper = row![
        button(text("-").size(18))
            .on_press_maybe(
                (!is_busy)
                    .then_some(Message::RedisTool(RedisToolMessage::DecreaseDefaultLoadCount,))
            )
            .padding([8, 12])
            .style(rounded_action_btn_style),
        text_input("500", &app.redis_tool.default_load_count_input)
            .on_input(|value| Message::RedisTool(RedisToolMessage::DefaultLoadCountChanged(value)))
            .padding([10, 14])
            .size(14)
            .width(Length::Fixed(96.0))
            .style(settings_text_input_style),
        button(text("+").size(18))
            .on_press_maybe(
                (!is_busy)
                    .then_some(Message::RedisTool(RedisToolMessage::IncreaseDefaultLoadCount,))
            )
            .padding([8, 12])
            .style(rounded_action_btn_style),
        button(text(if is_busy { "保存中..." } else { "保存" }).size(13))
            .on_press_maybe(
                (!is_busy).then_some(Message::RedisTool(RedisToolMessage::SaveDefaultLoadCount,))
            )
            .padding([10, 14])
            .style(primary_action_btn_style),
    ]
    .spacing(8)
    .align_y(Alignment::Center);

    let content = column![
        modal_header("通用", Message::RedisTool(RedisToolMessage::CloseSettingsModal)),
        settings_panel(
            column![form_row(
                "加载数量",
                "首版用于连接工作台的默认读取预算，后续键浏览与预览会直接复用该值。",
                count_stepper.into(),
                false,
            ),]
            .spacing(0),
        ),
        settings_panel(
            column![
                row![
                    column![
                        text("连接配置").size(14),
                        text("导出全部连接配置为 JSON，或从 JSON 文件导入并覆盖当前连接列表。")
                            .size(12)
                            .style(settings_muted_text_style),
                    ]
                    .spacing(4),
                    Space::new().width(Length::Fill),
                    settings_value_badge(format!("已保存 {} 个", app.redis_tool.connections.len())),
                ]
                .align_y(Alignment::Center),
                row![
                    button(text(if is_busy { "处理中..." } else { "导出" }).size(13))
                        .on_press_maybe(
                            (!is_busy)
                                .then_some(Message::RedisTool(RedisToolMessage::ExportConfigs,))
                        )
                        .padding([10, 14])
                        .style(rounded_action_btn_style),
                    button(text(if is_busy { "处理中..." } else { "导入" }).size(13))
                        .on_press_maybe(
                            (!is_busy)
                                .then_some(Message::RedisTool(RedisToolMessage::ImportConfigs,))
                        )
                        .padding([10, 14])
                        .style(primary_action_btn_style),
                ]
                .spacing(10),
            ]
            .spacing(14),
        ),
    ]
    .spacing(14)
    .width(Length::Fixed(SETTINGS_MODAL_WIDTH));

    modal_shell(content.into()).into()
}

/// 构建对应界面片段。
///
/// # 参数
/// - `app`: 当前视图构建所需的状态、配置或消息。
///
/// # 返回
/// 返回可直接嵌入父级视图的 Iced 控件或样式闭包。
///
/// # 错误
/// 此函数不返回 `Result`；不可用状态会通过空视图、禁用控件或回退文案表达。
pub(super) fn build_connection_modal<'a>(app: &'a App) -> Element<'a, Message> {
    let is_busy = app.redis_tool.is_gateway_loading();
    let title = if app.redis_tool.draft_is_new { "新建连接" } else { "连接配置" };
    let action_label = if is_busy { "保存中..." } else { "保存连接" };

    let content = column![
        modal_header(title, Message::RedisTool(RedisToolMessage::CloseConnectionModal)),
        scrollable(build_connection_form_panel(app, true, is_busy))
            .direction(redis_scroll_direction())
            .height(Length::Fixed(520.0)),
        row![
            Space::new().width(Length::Fill),
            button(text("取消").size(13))
                .on_press_maybe(
                    (!is_busy)
                        .then_some(Message::RedisTool(RedisToolMessage::CloseConnectionModal,))
                )
                .padding([10, 14])
                .style(rounded_action_btn_style),
            button(text(action_label).size(13))
                .on_press_maybe(
                    (!is_busy).then_some(Message::RedisTool(RedisToolMessage::SaveDraft,))
                )
                .padding([10, 14])
                .style(primary_action_btn_style),
        ]
        .spacing(10)
        .align_y(Alignment::Center),
    ]
    .spacing(14)
    .width(Length::Fixed(720.0));

    modal_shell(content.into()).into()
}

/// 构建对应界面片段。
///
/// # 参数
/// - `app`: 当前视图构建所需的状态、配置或消息。
///
/// # 返回
/// 返回可直接嵌入父级视图的 Iced 控件或样式闭包。
///
/// # 错误
/// 此函数不返回 `Result`；不可用状态会通过空视图、禁用控件或回退文案表达。
pub(super) fn build_history_modal<'a>(app: &'a App) -> Element<'a, Message> {
    let query = app.redis_tool.history_filter.trim().to_ascii_lowercase();
    let records = app.redis_tool.history.iter().filter(|record| {
        (!app.redis_tool.history_only_write || record.is_write)
            && (query.is_empty()
                || record.connection_label.to_ascii_lowercase().contains(&query)
                || record.command.to_ascii_lowercase().contains(&query)
                || record.args.to_ascii_lowercase().contains(&query))
    });

    let mut rows = column![history_table_header()].spacing(8);
    let mut has_data = false;
    for record in records {
        has_data = true;
        rows = rows.push(history_table_row(record));
    }

    if !has_data {
        rows = rows.push(
            container(
                column![
                    text("暂无数据").size(16),
                    text("保存连接、导入导出或打开连接后，这里会记录操作历史。")
                        .size(12)
                        .style(settings_muted_text_style),
                ]
                .spacing(8)
                .align_x(Alignment::Center),
            )
            .width(Length::Fill)
            .padding([56, 0])
            .center_x(Length::Fill),
        );
    }

    let content = column![
        modal_header("日志", Message::RedisTool(RedisToolMessage::CloseHistoryModal)),
        row![
            text_input("输入关键字搜索", &app.redis_tool.history_filter)
                .on_input(|value| Message::RedisTool(RedisToolMessage::HistoryFilterChanged(value)))
                .padding([10, 12])
                .size(13)
                .width(Length::FillPortion(2))
                .style(settings_text_input_style),
            checkbox(app.redis_tool.history_only_write)
                .label("Only Write")
                .on_toggle(|value| {
                    Message::RedisTool(RedisToolMessage::HistoryOnlyWriteToggled(value))
                })
                .style(settings_checkbox_style),
        ]
        .spacing(12)
        .align_y(Alignment::Center),
        row![
            text(history_page_label(app)).size(12).style(settings_muted_text_style),
            Space::new().width(Length::Fill),
            button(text("上一页").size(12))
                .on_press_maybe(
                    (!app.redis_tool.is_gateway_loading()
                        && app.redis_tool.history_page_offset > 0)
                        .then_some(Message::RedisTool(RedisToolMessage::HistoryPreviousPage))
                )
                .padding([8, 12])
                .style(rounded_action_btn_style),
            button(text("下一页").size(12))
                .on_press_maybe(
                    (!app.redis_tool.is_gateway_loading() && app.redis_tool.history_has_more)
                        .then_some(Message::RedisTool(RedisToolMessage::HistoryNextPage))
                )
                .padding([8, 12])
                .style(rounded_action_btn_style),
        ]
        .spacing(10)
        .align_y(Alignment::Center),
        settings_panel(
            scrollable(rows).direction(redis_scroll_direction()).height(Length::Fixed(440.0)),
        ),
    ]
    .spacing(14)
    .width(Length::Fixed(500.0));

    modal_shell(content.into()).into()
}

/// 构建对应界面片段。
///
/// # 参数
/// - `app`: 当前视图构建所需的状态、配置或消息。
///
/// # 返回
/// 返回可直接嵌入父级视图的 Iced 控件或样式闭包。
///
/// # 错误
/// 此函数不返回 `Result`；不可用状态会通过空视图、禁用控件或回退文案表达。
pub(super) fn build_create_key_modal<'a>(app: &'a App) -> Element<'a, Message> {
    let is_busy = app.redis_tool.is_gateway_loading();
    let can_submit = !is_busy && !app.redis_tool.create_key_draft.name.trim().is_empty();
    let selected_type = Some(app.redis_tool.create_key_draft.key_type);

    let content = column![
        modal_header("新增 Key", Message::RedisTool(RedisToolMessage::CloseCreateKeyModal)),
        settings_panel(
            column![
                form_row(
                    "键名",
                    "支持冒号层级，例如 order:detail:1001。",
                    text_input("例如：cache:user:1", &app.redis_tool.create_key_draft.name)
                        .on_input(|value| Message::RedisTool(
                            RedisToolMessage::CreateKeyNameChanged(value)
                        ))
                        .padding([10, 12])
                        .size(13)
                        .width(Length::Fill)
                        .style(settings_text_input_style)
                        .into(),
                    false,
                ),
                form_row(
                    "类型",
                    "确认后会按默认值初始化：集合类型会写入一条占位数据，随后自动进入详情。",
                    pick_list(&RedisKeyValueKind::ALL[..], selected_type, |value| {
                        Message::RedisTool(RedisToolMessage::CreateKeyTypeChanged(value))
                    })
                    .padding([10, 12])
                    .text_size(13)
                    .style(settings_pick_list_style)
                    .menu_style(settings_pick_list_menu_style)
                    .width(Length::Fill)
                    .into(),
                    false,
                ),
            ]
            .spacing(0),
        ),
        row![
            Space::new().width(Length::Fill),
            button(text("取消").size(13))
                .on_press_maybe(
                    (!is_busy)
                        .then_some(Message::RedisTool(RedisToolMessage::CloseCreateKeyModal,))
                )
                .padding([10, 14])
                .style(rounded_action_btn_style),
            button(text(if is_busy { "创建中..." } else { "创建" }).size(13))
                .on_press_maybe(
                    can_submit.then_some(Message::RedisTool(RedisToolMessage::ConfirmCreateKey,))
                )
                .padding([10, 14])
                .style(primary_action_btn_style),
        ]
        .spacing(10)
        .align_y(Alignment::Center),
    ]
    .spacing(14)
    .width(Length::Fixed(640.0));

    modal_shell(content.into()).into()
}

#[cfg(test)]
#[path = "modal_tests.rs"]
mod modal_tests;
