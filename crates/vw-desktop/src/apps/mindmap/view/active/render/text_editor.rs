//! 思维导图文本编辑器渲染逻辑，负责节点标题和正文编辑入口。

use crate::app::Message;
use crate::apps::mindmap::message::MindMapMessage;
use crate::apps::mindmap::state::MindMapTab;
use iced::widget::{Space, container, mouse_area, stack, text_editor};
use iced::{Background, Border, Color, Element, Length, Theme};

use super::super::super::common::{ideal_text_color, rgba_u32_to_color};
use super::geometry::selected_node_rect;

/// 构建或更新 with text editor overlay 相关行为。
///
/// 参数由调用方提供，返回值直接交给上层视图或状态处理；本函数不执行额外错误恢复。
pub(super) fn with_text_editor_overlay<'a>(
    tab: &'a MindMapTab,
    base: Element<'a, Message>,
) -> Element<'a, Message> {
    let backdrop = mouse_area(
        container(Space::new().width(Length::Fill).height(Length::Fill)).style(|_| {
            iced::widget::container::Style {
                background: Some(Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.25))),
                ..Default::default()
            }
        }),
    )
    .on_press(Message::MindMapTool(MindMapMessage::ClosePickers));

    let Some(node_rect) = selected_node_rect(tab) else {
        return stack(vec![base, backdrop.into()]).into();
    };

    let font_size = if tab.selected_path.as_deref().is_some_and(|path| path.is_empty()) {
        (18.0 * tab.zoom).clamp(14.0, 32.0)
    } else {
        (14.0 * tab.zoom).clamp(10.0, 24.0)
    };

    let node_fill = tab
        .selected_path
        .as_ref()
        .and_then(|path| tab.node_fills.get(path))
        .copied()
        .map(rgba_u32_to_color)
        .unwrap_or(Color::from_rgba8(255, 255, 255, 1.0));
    let editor_bg = Color { a: 1.0, ..node_fill };

    let node_text_color = tab
        .selected_path
        .as_ref()
        .and_then(|path| tab.node_text_colors.get(path))
        .copied()
        .map(rgba_u32_to_color)
        .unwrap_or_else(|| ideal_text_color(editor_bg));

    let editor_style = move |theme: &Theme, _status: iced::widget::text_editor::Status| {
        let palette = theme.palette();
        let luma = 0.299 * editor_bg.r + 0.587 * editor_bg.g + 0.114 * editor_bg.b;
        let border_color = if luma > 0.72 {
            Color::from_rgba(0.0, 0.0, 0.0, 0.18)
        } else {
            Color::from_rgba(1.0, 1.0, 1.0, 0.18)
        };
        iced::widget::text_editor::Style {
            background: Background::Color(editor_bg),
            border: Border { width: 1.0, color: border_color, radius: 8.0.into() },
            placeholder: node_text_color.scale_alpha(0.55),
            value: node_text_color,
            selection: palette.primary.scale_alpha(0.30),
        }
    };

    let input = text_editor(&tab.node_text_editor)
        .placeholder("输入文本…")
        .on_action(|action| Message::MindMapTool(MindMapMessage::NodeTextEditorAction(action)))
        .key_binding(|kp| {
            if matches!(kp.key.clone(), iced::keyboard::Key::Named(iced::keyboard::key::Named::Enter))
            {
                Some(iced::widget::text_editor::Binding::Custom(Message::MindMapTool(
                    MindMapMessage::NodeTextEditorEnter { shift: kp.modifiers.shift() },
                )))
            } else {
                iced::widget::text_editor::Binding::from_key_press(kp)
            }
        })
        .style(editor_style)
        .padding([2, 10])
        .size(font_size)
        .height(Length::Fill);

    let editor = mouse_area(
        container(input)
            .width(Length::Fixed(node_rect.width.max(40.0)))
            .height(Length::Fixed(node_rect.height.max(24.0)))
            .align_y(iced::alignment::Vertical::Center),
    )
    .on_press(Message::None);

    let editor_layer: Element<'_, Message> = container(editor)
        .padding(iced::Padding {
            top: node_rect.y,
            left: node_rect.x,
            ..Default::default()
        })
        .align_x(iced::alignment::Horizontal::Left)
        .align_y(iced::alignment::Vertical::Top)
        .width(Length::Fill)
        .height(Length::Fill)
        .into();

    stack(vec![base, backdrop.into(), editor_layer]).into()
}
