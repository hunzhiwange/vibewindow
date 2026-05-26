//! 思维导图右键上下文菜单渲染模块
//!
//! 本模块负责渲染思维导图节点右键菜单的覆盖层组件。该菜单提供了常见的节点操作功能，
//! 包括剪切、拷贝、粘贴和删除操作，每个操作都配有相应的图标按钮和工具提示。
//!
//! # 主要功能
//!
//! - 根据操作可用性动态渲染菜单按钮
//! - 为禁用状态的按钮提供视觉反馈（灰色显示）
//! - 提供带有快捷键提示的工具提示功能
//! - 统一的菜单样式设计（白色背景、圆角边框、阴影效果）

use crate::app::Message;
use crate::app::assets::{self, Icon};
use crate::apps::mindmap::message::MindMapMessage;
use iced::widget::svg;
use iced::widget::{button, container, row, text, tooltip};
use iced::{Alignment, Background, Border, Color, Element, Length, Renderer, Theme};

/// 创建右键上下文菜单覆盖层
///
/// 该函数根据当前操作的可用性状态，构建一个包含四个操作按钮的上下文菜单。
/// 每个按钮都配有图标、工具提示，并根据操作是否可用显示不同的视觉样式。
///
/// # 参数
///
/// * `can_cut` - 是否允许剪切操作
/// * `can_copy` - 是否允许拷贝操作
/// * `can_paste` - 是否允许粘贴操作
/// * `can_delete` - 是否允许删除操作
///
/// # 返回值
///
/// 返回一个包含上下文菜单的 `Element` 组件，该组件可以嵌入到更大的 UI 层次结构中。
///
/// # 示例
///
/// ```ignore
/// let menu = context_menu_overlay(true, true, false, true);
/// // 创建一个包含剪切、拷贝、删除按钮的菜单，粘贴按钮被禁用
/// ```
pub(super) fn context_menu_overlay(
    can_cut: bool,
    can_copy: bool,
    can_paste: bool,
    can_delete: bool,
) -> Element<'static, Message> {
    // 创建带图标和工具提示的菜单按钮
    //
    // 该闭包用于创建一个统一的菜单按钮样式，包括：
    // - 图标显示（14x14 像素的 SVG 图标）
    // - 悬停和按下状态的视觉反馈
    // - 顶部工具提示显示操作名称和快捷键
    // - 根据消息是否存在决定按钮是否可交互
    //
    // # 参数
    //
    // * `icon` - 按钮显示的图标类型
    // * `msg` - 点击按钮时发送的消息，若为 `None` 则按钮处于禁用状态
    // * `tooltip_text` - 工具提示中显示的文本内容
    let menu_icon_btn = |icon: Icon,
                         msg: Option<MindMapMessage>,
                         tooltip_text: &'static str|
     -> Element<'static, Message, Theme, Renderer> {
        // 根据消息是否存在，创建可用或禁用状态的按钮
        let btn: Element<'static, Message, Theme, Renderer> = if let Some(m) = msg {
            // 创建可交互的按钮：使用主题色、响应悬停和按下状态
            button(
                container(
                    svg(assets::get_icon(icon))
                        .width(Length::Fixed(14.0))
                        .height(Length::Fixed(14.0))
                        .content_fit(iced::ContentFit::Contain)
                        // 图标颜色使用当前主题的文本颜色
                        .style(|theme: &Theme, _| iced::widget::svg::Style {
                            color: Some(theme.palette().text),
                        }),
                )
                .width(Length::Fill)
                .height(Length::Fill)
                .align_x(iced::alignment::Horizontal::Center)
                .align_y(iced::alignment::Vertical::Center),
            )
            .padding(0)
            .width(Length::Fixed(28.0))
            .height(Length::Fixed(28.0))
            // 按钮样式：根据交互状态调整背景色
            .style(|theme: &Theme, status| {
                let palette = theme.extended_palette();
                // 根据按钮状态选择背景颜色
                let bg = match status {
                    iced::widget::button::Status::Hovered => Some(palette.background.weak.color),
                    iced::widget::button::Status::Pressed => Some(palette.background.strong.color),
                    _ => None,
                };
                iced::widget::button::Style {
                    background: bg.map(Background::Color),
                    border: Border { width: 0.0, color: Color::TRANSPARENT, radius: 6.0.into() },
                    text_color: theme.palette().text,
                    ..Default::default()
                }
            })
            .on_press(Message::MindMapTool(m))
            .into()
        } else {
            // 创建禁用状态的按钮：使用灰色、无交互响应
            button(
                container(
                    svg(assets::get_icon(icon))
                        .width(Length::Fixed(14.0))
                        .height(Length::Fixed(14.0))
                        .content_fit(iced::ContentFit::Contain)
                        // 禁用状态的图标使用固定灰色
                        .style(move |_theme: &Theme, _| iced::widget::svg::Style {
                            color: Some(Color::from_rgba8(160, 160, 160, 1.0)),
                        }),
                )
                .width(Length::Fill)
                .height(Length::Fill)
                .align_x(iced::alignment::Horizontal::Center)
                .align_y(iced::alignment::Vertical::Center),
            )
            .padding(0)
            .width(Length::Fixed(28.0))
            .height(Length::Fixed(28.0))
            // 禁用状态按钮样式：无背景、灰色文本
            .style(move |_theme: &Theme, _status| iced::widget::button::Style {
                background: None,
                border: Border { width: 0.0, color: Color::TRANSPARENT, radius: 6.0.into() },
                text_color: Color::from_rgba8(160, 160, 160, 1.0),
                ..Default::default()
            })
            .into()
        };

        // 创建工具提示内容：深色背景、白色文本、圆角设计
        let tip_content =
            container(text(tooltip_text).size(12)).padding([6, 8]).style(|_theme: &Theme| {
                iced::widget::container::Style {
                    // 工具提示背景：深灰色半透明
                    background: Some(Color::from_rgba8(24, 24, 24, 0.96).into()),
                    text_color: Some(Color::WHITE),
                    border: Border { width: 0.0, color: Color::TRANSPARENT, radius: 8.0.into() },
                    // 工具提示阴影效果
                    shadow: iced::Shadow {
                        color: Color::BLACK.scale_alpha(0.30),
                        offset: iced::Vector::new(0.0, 6.0),
                        blur_radius: 16.0,
                    },
                    ..Default::default()
                }
            });
        // 将工具提示与按钮绑定，显示在按钮上方
        tooltip::Tooltip::new(btn, tip_content, tooltip::Position::Top).gap(4.0).into()
    };

    // 构建菜单容器：包含四个操作按钮的横向布局
    container(
        row![
            // 剪切按钮：仅当 can_cut 为 true 时可交互
            menu_icon_btn(
                Icon::Scissors,
                can_cut.then_some(MindMapMessage::CutNode),
                "剪切 (Ctrl+X)"
            ),
            // 拷贝按钮：仅当 can_copy 为 true 时可交互
            menu_icon_btn(
                Icon::Copy,
                can_copy.then_some(MindMapMessage::CopyNode),
                "拷贝 (Ctrl+C)"
            ),
            // 粘贴按钮：仅当 can_paste 为 true 时可交互
            menu_icon_btn(
                Icon::Clipboard,
                can_paste.then_some(MindMapMessage::PasteNode),
                "粘贴 (Ctrl+V)"
            ),
            // 删除按钮：仅当 can_delete 为 true 时可交互
            menu_icon_btn(
                Icon::Trash,
                can_delete.then_some(MindMapMessage::DeleteNode),
                "删除 (Delete)"
            ),
        ]
        .spacing(4)
        .align_y(Alignment::Center),
    )
    .padding([6, 8])
    // 菜单容器样式：白色背景、浅灰边框、阴影效果
    .style(|_theme: &Theme| iced::widget::container::Style {
        background: Some(Background::Color(Color::from_rgba8(255, 255, 255, 1.0))),
        border: Border {
            width: 1.0,
            color: Color::from_rgba8(210, 210, 210, 1.0),
            radius: 8.0.into(),
        },
        // 菜单阴影：轻微的投影效果增加层次感
        shadow: iced::Shadow {
            color: Color::BLACK.scale_alpha(0.10),
            offset: iced::Vector::new(0.0, 8.0),
            blur_radius: 16.0,
        },
        ..Default::default()
    })
    .into()
}
