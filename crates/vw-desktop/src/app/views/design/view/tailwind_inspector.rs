//! # Tailwind 结构检查器模块
//!
//! 本模块负责底部 Tailwind DOM 检查器的渲染与树节点交互。
//! 它仅处理结构树展示、折叠与选中，不改变顶层视图的消息和状态模型。

use std::collections::HashSet;

use iced::widget::{Space, button, column, container, mouse_area, row, scrollable, svg, text};
use iced::{Background, Border, Color, Element, Length, Theme};

use crate::app::assets::{self, Icon};
use crate::app::message::DesignMessage;
use crate::app::views::design::canvas::tailwind::dom;
use crate::app::views::design::state::DesignState;
use crate::app::{App, Message};

/// 渲染 Tailwind 结构检查器面板
///
/// 当选中 Tailwind 类型的元素时，在界面底部显示该元素的 HTML 结构树，
/// 允许用户浏览和选择嵌套的子节点进行编辑。
pub(super) fn render_tailwind_inspector_panel<'a>(
    _app: &'a App,
    state: &'a DesignState,
) -> Element<'a, Message> {
    let element_id =
        state
            .selected_element_id
            .as_ref()
            .and_then(|id| state.doc.find_element(id).map(|el| (id, el)))
            .and_then(|(id, el)| {
                if el.kind.eq_ignore_ascii_case("tailwind") { Some(id.clone()) } else { None }
            })
            .or_else(|| state.doc.tailwind_selection.as_ref().map(|(id, _)| id.clone()));

    let Some(element_id) = element_id else {
        return Space::new().into();
    };

    let Some(el) = state.doc.find_element(&element_id) else {
        return Space::new().into();
    };

    if !el.kind.eq_ignore_ascii_case("tailwind") {
        return Space::new().into();
    }

    let Some(content) = el.content.as_deref() else {
        return Space::new().into();
    };

    let nodes = dom::parse_html(content);

    let selected_path = state
        .doc
        .tailwind_selection
        .as_ref()
        .and_then(|(id, path)| if id == &element_id { Some(path.clone()) } else { None });

    let header_btn_style = |theme: &Theme, status: button::Status| {
        let palette = theme.extended_palette();
        let bg = match status {
            button::Status::Hovered => Some(Background::Color(palette.background.weak.color)),
            button::Status::Pressed => Some(Background::Color(palette.background.strong.color)),
            _ => None,
        };
        button::Style {
            background: bg,
            border: Border { width: 0.0, color: Color::TRANSPARENT, radius: 8.0.into() },
            text_color: theme.palette().text,
            ..Default::default()
        }
    };

    let icon_svg = |icon: Icon, alpha: f32| {
        svg(assets::get_icon(icon)).width(Length::Fixed(14.0)).height(Length::Fixed(14.0)).style(
            move |theme: &Theme, _| iced::widget::svg::Style {
                color: Some(theme.palette().text.scale_alpha(alpha)),
            },
        )
    };

    let collapse_icon =
        if state.tailwind_inspector_collapsed { Icon::ChevronUp } else { Icon::ChevronDown };
    let collapse_btn: Element<'a, Message> = button(icon_svg(collapse_icon, 0.75))
        .on_press(Message::Design(DesignMessage::ToggleTailwindInspectorCollapsed))
        .style(header_btn_style)
        .padding(6)
        .into();

    let can_undo = state.history_index > 0;
    let can_redo = state.history_index + 1 < state.history.len();

    let undo_btn: Element<'a, Message> = if can_undo {
        button(icon_svg(Icon::ArrowCounterClockwise, 0.75))
            .style(header_btn_style)
            .padding(6)
            .on_press(Message::Design(DesignMessage::Undo))
            .into()
    } else {
        button(icon_svg(Icon::ArrowCounterClockwise, 0.25))
            .style(header_btn_style)
            .padding(6)
            .into()
    };

    let redo_btn: Element<'a, Message> = if can_redo {
        button(icon_svg(Icon::ArrowClockwise, 0.75))
            .style(header_btn_style)
            .padding(6)
            .on_press(Message::Design(DesignMessage::Redo))
            .into()
    } else {
        button(icon_svg(Icon::ArrowClockwise, 0.25)).style(header_btn_style).padding(6).into()
    };

    let header = row![
        collapse_btn,
        text("Tailwind 结构").size(12),
        Space::new().width(Length::Fill),
        undo_btn,
        redo_btn,
    ]
    .align_y(iced::Alignment::Center)
    .spacing(6);

    let body: Element<'a, Message> = if state.tailwind_inspector_collapsed {
        Space::new().height(Length::Fixed(0.0)).into()
    } else {
        let tree = column(
            nodes
                .iter()
                .cloned()
                .enumerate()
                .map(|(i, node)| -> Element<'a, Message> {
                    render_tailwind_tree_node(
                        element_id.clone(),
                        node,
                        vec![i],
                        selected_path.as_ref(),
                        &state.tailwind_tree_collapsed,
                    )
                })
                .collect::<Vec<_>>(),
        )
        .spacing(2);

        container(
            scrollable(tree).id(state.tailwind_tree_scroll_id.clone()).height(Length::Fixed(220.0)),
        )
        .width(Length::Fill)
        .into()
    };

    let panel = container(column![header, body].spacing(8))
        .width(Length::Fixed(940.0))
        .padding(10)
        .style(|theme: &Theme| {
            let ext = theme.extended_palette();
            iced::widget::container::Style {
                background: Some(Background::Color(ext.background.base.color)),
                border: Border {
                    width: 1.0,
                    color: ext.background.strong.color,
                    radius: 12.0.into(),
                },
                shadow: iced::Shadow {
                    color: Color::BLACK.scale_alpha(0.12),
                    offset: iced::Vector::new(0.0, 10.0),
                    blur_radius: 22.0,
                },
                ..Default::default()
            }
        });

    mouse_area(
        container(panel)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(iced::Padding { top: 0.0, right: 20.0, bottom: 18.0, left: 20.0 })
            .align_x(iced::alignment::Horizontal::Center)
            .align_y(iced::alignment::Vertical::Bottom),
    )
    .on_enter(Message::Design(DesignMessage::TailwindInspectorHover(true)))
    .on_exit(Message::Design(DesignMessage::TailwindInspectorHover(false)))
    .into()
}

/// 渲染 Tailwind 结构树的节点
///
/// 递归渲染单个 HTML 节点及其所有子节点，形成完整的 DOM 结构树。
fn render_tailwind_tree_node<'a>(
    element_id: String,
    node: dom::TailwindNode,
    path: Vec<usize>,
    selected_path: Option<&Vec<usize>>,
    collapsed: &HashSet<String>,
) -> Element<'a, Message> {
    let is_selected = selected_path.map(|p| p == &path).unwrap_or(false);
    let has_children = !node.children.is_empty();
    let key = tailwind_collapse_key(&element_id, &path);
    let is_collapsed = has_children && collapsed.contains(&key);

    let depth = path.len().saturating_sub(1) as f32;
    let indent = Space::new().width(Length::Fixed(depth * 14.0));

    let chevron: Element<'a, Message> = if has_children {
        let icon = if is_collapsed { Icon::ChevronRight } else { Icon::ChevronDown };
        button(
            svg(assets::get_icon(icon))
                .width(Length::Fixed(10.0))
                .height(Length::Fixed(10.0))
                .style(|theme: &Theme, _| iced::widget::svg::Style {
                    color: Some(theme.palette().text.scale_alpha(0.62)),
                }),
        )
        .on_press(Message::Design(DesignMessage::ToggleTailwindTreeCollapsed(
            element_id.clone(),
            path.clone(),
        )))
        .style(|_theme: &Theme, _status| button::Style {
            background: None,
            border: Border { width: 0.0, color: Color::TRANSPARENT, radius: 6.0.into() },
            ..Default::default()
        })
        .padding(2)
        .into()
    } else {
        Space::new().width(Length::Fixed(14.0)).height(Length::Fixed(14.0)).into()
    };

    let label = if let Some(t) = node.text.as_deref() {
        format!("\"{}\"", t)
    } else {
        format!("<{}>", node.tag)
    };

    let class_preview = node.attributes.get("class").cloned().unwrap_or_default();

    let label_content: Element<'a, Message> = if class_preview.is_empty() {
        text(label).size(12).into()
    } else {
        row![
            text(label).size(12),
            text(class_preview).size(11).style(|theme: &Theme| iced::widget::text::Style {
                color: Some(theme.palette().text.scale_alpha(0.45)),
            })
        ]
        .spacing(6)
        .into()
    };

    let label_btn: Element<'a, Message> = button(label_content)
        .on_press(Message::Design(DesignMessage::SelectTailwindNode(
            element_id.clone(),
            path.clone(),
        )))
        .style(move |theme: &Theme, status| {
            let palette = theme.extended_palette();
            let bg = if is_selected {
                Some(Background::Color(palette.primary.weak.color.scale_alpha(0.35)))
            } else {
                match status {
                    button::Status::Hovered => {
                        Some(Background::Color(palette.background.weak.color.scale_alpha(0.55)))
                    }
                    button::Status::Pressed => {
                        Some(Background::Color(palette.background.strong.color.scale_alpha(0.55)))
                    }
                    _ => None,
                }
            };
            button::Style {
                background: bg,
                border: Border { width: 0.0, color: Color::TRANSPARENT, radius: 8.0.into() },
                text_color: theme.palette().text,
                ..Default::default()
            }
        })
        .padding([3, 8])
        .width(Length::Fill)
        .into();

    let self_row = row![indent, chevron, label_btn].spacing(4).align_y(iced::Alignment::Center);

    let mut out = column![Element::from(self_row)].spacing(2);

    if has_children && !is_collapsed {
        for (i, child) in node.children.iter().cloned().enumerate() {
            let mut child_path = path.clone();
            child_path.push(i);
            out = out.push(render_tailwind_tree_node(
                element_id.clone(),
                child,
                child_path,
                selected_path,
                collapsed,
            ));
        }
    }

    out.into()
}

/// 生成 Tailwind 树节点的折叠状态键。
fn tailwind_collapse_key(id: &str, path: &[usize]) -> String {
    let mut s = String::with_capacity(id.len() + 1 + path.len() * 3);
    s.push_str(id);
    s.push('|');

    for (i, p) in path.iter().enumerate() {
        if i > 0 {
            s.push('.');
        }
        s.push_str(&p.to_string());
    }

    s
}
#[cfg(test)]
#[path = "tailwind_inspector_tests.rs"]
mod tailwind_inspector_tests;
