//! Git 筛选选项视图模块
//!
//! 本模块提供 Git 文件筛选功能的 UI 组件，用于在 Git 面板中显示和配置文件筛选条件。
//!
//! # 主要功能
//!
//! - 文本查询筛选：通过输入关键字筛选文件路径
//! - 状态筛选：按文件状态（新增/修改/删除/包含/排除）筛选
//! - 快速清除：一键清除所有筛选条件
//!
//! # 组件结构
//!
//! ```text
//! ┌─────────────────────────────────────┐
//! │ 筛选选项                      [✕] │
//! ├─────────────────────────────────────┤
//! │ [文本输入框]                        │
//! │ [○] 已包含到提交（N）               │
//! │ [○] 已排除出提交（N）               │
//! │ [○] 新增文件（N）                   │
//! │ [○] 修改文件（N）                   │
//! │ [○] 删除文件（N）                   │
//! │ [清除筛选]                          │
//! └─────────────────────────────────────┘
//! ```

use iced::widget::{button, column, container, row, text, text_input, toggler};
use iced::{Alignment, Background, Border, Color, Element, Length, Theme};

use crate::app::components::system_settings_common::{
    round_icon_btn_style, rounded_action_btn_style, settings_muted_text_style, settings_panel_style,
    settings_text_input_style, settings_value_badge,
};
use crate::app::{App, Message, message};

fn filter_toggle_row<'a>(
    title: &'static str,
    count: usize,
    control: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    let title_block = row![
        text(title).size(13),
        settings_value_badge(count.to_string()),
    ]
    .spacing(8)
    .align_y(Alignment::Center);

    let control = container(control.into())
        .width(Length::Fixed(40.0))
        .center_x(Length::Shrink)
        .center_y(Length::Shrink);

    container(
        row![
            container(title_block).width(Length::Fill).center_y(Length::Shrink),
            control,
        ]
        .spacing(12)
        .align_y(Alignment::Center),
    )
    .padding([10, 12])
    .width(Length::Fill)
    .style(|theme: &Theme| {
        let palette = theme.extended_palette();
        let is_dark = theme.palette().background.r
            + theme.palette().background.g
            + theme.palette().background.b
            < 1.5;

        iced::widget::container::Style {
            background: Some(Background::Color(if is_dark {
                palette.background.weak.color.scale_alpha(0.18)
            } else {
                Color::from_rgba8(246, 248, 252, 0.96)
            })),
            border: Border {
                width: 1.0,
                color: if is_dark {
                    palette.background.strong.color.scale_alpha(0.78)
                } else {
                    Color::from_rgba8(15, 23, 42, 0.06)
                },
                radius: 14.0.into(),
            },
            ..Default::default()
        }
    })
    .into()
}

/// 构建 Git 筛选选项面板视图
///
/// 该函数创建一个包含多种筛选选项的 UI 面板，允许用户通过以下方式筛选 Git 文件列表：
/// - 文本查询：匹配文件路径
/// - 状态开关：按文件的 Git 状态筛选
///
/// # 参数
///
/// - `app`: 应用状态引用，包含当前筛选条件和主题配置
/// - `included_count`: 已包含到提交的文件数量
/// - `excluded_count`: 已排除出提交的文件数量
/// - `new_count`: 新增文件的数量
/// - `modified_count`: 修改文件的数量
/// - `deleted_count`: 删除文件的数量
///
/// # 返回值
///
/// 返回一个 `Element<Message>`，即渲染后的 UI 组件
///
/// # 示例
///
/// ```ignore
/// let filter_view = view(
///     &app,
///     included_files.len(),
///     excluded_files.len(),
///     new_files.len(),
///     modified_files.len(),
///     deleted_files.len(),
/// );
/// ```
pub fn view(
    app: &App,
    included_count: usize,
    excluded_count: usize,
    new_count: usize,
    modified_count: usize,
    deleted_count: usize,
) -> Element<'_, Message> {
    let active_filter_count = usize::from(!app.git_filter_query.trim().is_empty())
        + usize::from(app.git_filter_included)
        + usize::from(app.git_filter_excluded)
        + usize::from(app.git_filter_new)
        + usize::from(app.git_filter_modified)
        + usize::from(app.git_filter_deleted);

    // 标题栏：包含"筛选选项"文本和关闭按钮
    let title_text = container(
        row![
            text("筛选选项").size(14),
            settings_value_badge(format!("{} 项启用", active_filter_count)),
            text("按路径与变更状态快速聚焦文件列表。")
                .size(11)
                .style(settings_muted_text_style),
        ]
        .spacing(8)
        .align_y(Alignment::Center),
    )
    .width(Length::Fill)
    .center_y(Length::Shrink);

    let title = row![
        title_text,
        // 关闭按钮，点击后隐藏筛选选项面板
        container(
            button(text("✕").size(12))
            .padding(0)
            .width(Length::Fixed(28.0))
            .height(Length::Fixed(28.0))
            .on_press(Message::Git(message::GitMessage::ToggleFilterOptions(false)))
            .style(round_icon_btn_style)
        )
        .center_x(Length::Shrink)
        .center_y(Length::Shrink)
    ]
    .spacing(8)
    .align_y(iced::Alignment::Center);

    // 筛选文本输入框：用于输入文件路径查询关键字
    let filter_input = text_input("筛选", &app.git_filter_query)
        .on_input(|v| Message::Git(message::GitMessage::FilterQueryChanged(v)))
        .padding([10, 12])
        .size(13)
        .style(settings_text_input_style)
        .width(Length::Fill);

    // 各状态筛选开关
    // 已包含到提交的文件筛选开关
    let included = toggler(app.git_filter_included)
        .on_toggle(|b| Message::Git(message::GitMessage::FilterToggleIncluded(b)));

    // 已排除出提交的文件筛选开关
    let excluded = toggler(app.git_filter_excluded)
        .on_toggle(|b| Message::Git(message::GitMessage::FilterToggleExcluded(b)));

    // 新增文件筛选开关
    let newf = toggler(app.git_filter_new)
        .on_toggle(|b| Message::Git(message::GitMessage::FilterToggleNew(b)));

    // 修改文件筛选开关
    let modifiedf = toggler(app.git_filter_modified)
        .on_toggle(|b| Message::Git(message::GitMessage::FilterToggleModified(b)));

    // 删除文件筛选开关
    let deletedf = toggler(app.git_filter_deleted)
        .on_toggle(|b| Message::Git(message::GitMessage::FilterToggleDeleted(b)));

    // 清除所有筛选条件的按钮
    let clear = button(text("清除筛选").size(12))
        .padding([8, 12])
        .on_press(Message::Git(message::GitMessage::ClearFilters))
        .style(rounded_action_btn_style);

    // 组装完整的筛选选项面板
    container(
        column![
            title,
            filter_input,
            // 各筛选选项行：开关 + 标签（含数量）
            column![
                filter_toggle_row(
                    "已包含到提交",
                    included_count,
                    included,
                ),
                filter_toggle_row(
                    "已排除出提交",
                    excluded_count,
                    excluded,
                ),
                filter_toggle_row("新增文件", new_count, newf),
                filter_toggle_row(
                    "修改文件",
                    modified_count,
                    modifiedf,
                ),
                filter_toggle_row("删除文件", deleted_count, deletedf),
            ]
            .spacing(6),
            row![
                container(text("")).width(Length::Fill).center_y(Length::Shrink),
                container(clear).center_x(Length::Shrink).center_y(Length::Shrink)
            ]
                .align_y(Alignment::Center)
                .spacing(8)
        ]
        .spacing(12),
    )
    .padding([16, 18])
    .style(settings_panel_style)
    .into()
}
