//! Git 面板头部组件
//!
//! 该模块提供 Git 面板的头部界面渲染功能，包含分支显示和操作按钮。
//!
//! # 主要功能
//!
//! - 显示当前分支名称
//! - 展开/折叠全部文件列表
//! - 显示/隐藏摘要输入框
//! - 过滤选项按钮
//!
//! # 交互特性
//!
//! - 顶部操作按钮始终可见
//! - 保持紧凑间距，避免右侧按钮贴边

use iced::Element;
use iced::alignment::Vertical;
use iced::widget::{Space, button, container, row, text};
use iced::{Background, Border, Length};

use crate::app::assets::Icon;
use crate::app::{App, Message, message};

use super::ui::square_icon_button_micro;

/// 渲染 Git 面板的头部界面
///
/// 该函数构建 Git 面板头部的完整 UI，包括分支名称和一系列操作按钮。
/// 操作按钮始终显示，方便直接切换常用视图和过滤能力。
///
/// # 参数
///
/// - `app`: 应用状态引用，用于读取当前分支、视图模式等状态
/// - `files_list`: 文件列表，用于判断是否全部展开/折叠
///
/// # 返回值
///
/// 返回构建好的 Element，包含完整的头部 UI 组件
///
/// # 界面元素
///
/// 1. **分支按钮**: 显示当前分支名称，点击打开终端
/// 2. **全选按钮**: 选中当前列表中的所有变更行
/// 3. **反选按钮**: 反选当前列表中的所有变更行
/// 4. **摘要输入框开关**: 显示或隐藏 Git diff 摘要输入框
/// 5. **过滤选项**: 打开或关闭 Git 过滤选项面板
///
pub fn view(app: &App, files_list: Vec<String>) -> Element<'_, Message> {
    // 获取当前分支名称，如果未选择则显示 "-"
    let branch_name = app.selected_branch.clone().unwrap_or_else(|| "-".to_string());

    // 构建分支按钮，显示当前分支名称
    // 点击时发送打开终端的消息
    let branch = button(text(branch_name).size(13).height(Length::Fill).align_y(Vertical::Center))
        .on_press(Message::View(message::ViewMessage::OpenTerminalPressed))
        .height(Length::Fixed(21.0))
        .padding([0, 10])
        .style(|theme: &iced::Theme, status| {
            let palette = theme.extended_palette();
            // 根据按钮状态（悬停、按下、默认）设置不同的背景色
            let bg = match status {
                iced::widget::button::Status::Hovered => Some(palette.background.weak.color.into()),
                iced::widget::button::Status::Pressed => {
                    Some(palette.background.strong.color.into())
                }
                _ => Some(palette.background.base.color.into()),
            };
            iced::widget::button::Style {
                background: bg,
                border: Border {
                    width: 1.0,
                    color: palette.background.strong.color,
                    radius: 8.0.into(),
                },
                text_color: theme.palette().text,
                ..Default::default()
            }
        });

    let select_all_btn = square_icon_button_micro(
        Icon::CheckSquare,
        "全选当前列表的变更行".to_string(),
        Message::Git(message::GitMessage::SelectAllVisibleFileLines(files_list.clone())),
    );

    let invert_select_btn = square_icon_button_micro(
        Icon::Square,
        "反选当前列表的变更行".to_string(),
        Message::Git(message::GitMessage::InvertVisibleFileLines(files_list.clone())),
    );

    // 构建摘要输入框显示/隐藏切换按钮
    // 用于控制 Git diff 摘要输入框的可见性
    let summary_toggle = {
        // 根据当前显示状态选择图标和提示文本
        let (icon, tip) = if app.show_git_diff_summary {
            (Icon::EyeSlash, "隐藏摘要输入框".to_string())
        } else {
            (Icon::Eye, "显示摘要输入框".to_string())
        };
        square_icon_button_micro(
            icon,
            tip,
            Message::View(message::ViewMessage::ToggleGitDiffSummary),
        )
    };

    // 构建过滤选项按钮
    // 用于打开或关闭 Git 文件过滤选项面板
    let filter_btn = square_icon_button_micro(
        Icon::Sliders,
        "过滤选项".to_string(),
        Message::Git(message::GitMessage::ToggleFilterOptions(!app.show_git_filter_options)),
    );

    let actions: Element<'_, Message> = row![
        row![select_all_btn, invert_select_btn, summary_toggle, filter_btn]
            .spacing(8)
            .align_y(iced::Alignment::Center),
        Space::new().width(Length::Fixed(10.0))
    ]
    .align_y(iced::Alignment::Center)
    .into();

    // 构建头部内容行：分支名称 + 弹性空间 + 操作按钮
    let content = row![branch, Space::new().width(Length::Fill), actions]
        .spacing(10)
        .align_y(iced::Alignment::Center);

    // 为内容添加容器样式，设置背景色
    let content = container(content).style(|theme: &iced::Theme| {
        let palette = theme.extended_palette();
        iced::widget::container::Style {
            background: Some(Background::Color(palette.background.base.color)),
            ..Default::default()
        }
    });

    // 用 MouseArea 包裹内容，实现悬停显示/隐藏操作按钮的交互
    iced::widget::MouseArea::new(content)
        .on_enter(Message::Git(message::GitMessage::HoverGitPanelHeaderEnter))
        .on_exit(Message::Git(message::GitMessage::HoverGitPanelHeaderExit))
        .into()
}
