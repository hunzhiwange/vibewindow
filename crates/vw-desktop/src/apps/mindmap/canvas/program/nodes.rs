//! 思维导图画布节点绘制逻辑，负责节点主体、文本和状态徽标渲染。

use crate::app::components::mind_map;
use crate::app::views::design::canvas::rendering::svg::build_svg_path;
use crate::apps::mindmap::canvas::layout::{Layout, layout_node_rect};
use crate::apps::mindmap::canvas::style::{
    dash_segments_px, ideal_text_color, priority_color, rgba_u32_to_color,
};
use crate::apps::mindmap::canvas::transform::screen_from_world;
use crate::apps::mindmap::state::EdgeStyle;
use iced::widget::canvas::{Frame, LineCap, LineDash, Path, Stroke, Text};
use iced::{Color, Pixels, Point, Rectangle, Size, Theme};

use super::super::hit_test::descendant_count;
use super::super::{HoverButtonKind, MindMapCanvas};
use super::ThemeView;

/// 构建或更新 draw nodes 相关行为。
///
/// 参数由调用方提供，返回值直接交给上层视图或状态处理；本函数不执行额外错误恢复。
pub(super) fn draw_nodes(
    frame: &mut Frame,
    canvas: &MindMapCanvas<'_>,
    theme: &Theme,
    layout: &Layout,
    current_theme: &ThemeView<'_>,
    stroke_color: Color,
    stroke_width: f32,
) {
    for node_layout in &layout.nodes {
        let world_rect = layout_node_rect(node_layout);
        let top_left =
            screen_from_world(Point::new(world_rect.x, world_rect.y), canvas.pan, canvas.zoom);
        let size = Size::new(world_rect.width * canvas.zoom, world_rect.height * canvas.zoom);
        let rect = Rectangle::new(top_left, size);
        let is_root = node_layout.path.is_empty();

        let theme_fill = if is_root {
            current_theme.root_fill
        } else if node_layout.path.len() == 1 {
            current_theme.palette(node_layout.path[0])
        } else {
            current_theme.leaf_fill
        };
        let theme_text = if is_root {
            current_theme.root_text
        } else if node_layout.path.len() == 1 {
            current_theme.branch_text
        } else {
            current_theme.leaf_text
        };

        let fill = canvas
            .node_fills
            .get(&node_layout.path)
            .copied()
            .map(rgba_u32_to_color)
            .unwrap_or(rgba_u32_to_color(theme_fill));
        let border_color = canvas
            .node_border_colors
            .get(&node_layout.path)
            .copied()
            .map(rgba_u32_to_color)
            .unwrap_or(stroke_color);
        let text_color = canvas
            .node_text_colors
            .get(&node_layout.path)
            .copied()
            .map(rgba_u32_to_color)
            .unwrap_or(rgba_u32_to_color(theme_text));
        let priority = canvas.node_priorities.get(&node_layout.path).copied();
        let radius = if is_root {
            (8.0 * canvas.zoom).clamp(4.0, 12.0)
        } else {
            (rect.height / 2.0).max(0.0)
        };

        let node_path =
            Path::rounded_rectangle(Point::new(rect.x, rect.y), rect.size(), radius.into());
        frame.fill(&node_path, fill);

        if let Some(value) = priority {
            draw_priority_badge(frame, canvas, rect, value);
        }

        let node_style = canvas
            .node_border_styles
            .get(&node_layout.path)
            .copied()
            .unwrap_or(canvas.node_border_style);
        frame.stroke(
            &node_path,
            dashed_stroke(node_style, border_color, stroke_width, canvas.zoom),
        );

        let has_priority = priority.filter(|value| (1..=10).contains(value)).is_some();
        let has_url = canvas
            .node_urls
            .get(&node_layout.path)
            .is_some_and(|url| !url.trim().is_empty());
        let (text_pos, text_align) = if has_priority || has_url {
            let pad = (8.0 * canvas.zoom).clamp(4.0, 10.0);
            let r = (9.0 * canvas.zoom).clamp(4.0, 12.0);
            let after = (6.0 * canvas.zoom).clamp(3.0, 10.0);
            let mut x = rect.x + pad;
            if has_priority {
                x += r * 2.0 + after;
            }
            (
                Point::new(x, rect.y + rect.height / 2.0),
                iced::widget::text::Alignment::Left,
            )
        } else {
            (
                Point::new(rect.x + rect.width / 2.0, rect.y + rect.height / 2.0),
                iced::widget::text::Alignment::Center,
            )
        };

        let font_size = if is_root {
            (18.0 * canvas.zoom).clamp(14.0, 32.0)
        } else {
            (14.0 * canvas.zoom).clamp(10.0, 24.0)
        };
        let font = if is_root {
            iced::font::Font {
                weight: iced::font::Weight::Bold,
                ..Default::default()
            }
        } else {
            iced::font::Font::default()
        };
        let line_count = node_layout.text.split('\n').count().max(1);
        let line_step = (if is_root { 22.0 } else { 18.0 }) * canvas.zoom;
        let start_y =
            rect.y + rect.height / 2.0 - (line_count.saturating_sub(1) as f32) * line_step / 2.0;

        for (index, line) in node_layout.text.split('\n').enumerate() {
            frame.fill_text(Text {
                content: line.to_string(),
                position: Point::new(text_pos.x, start_y + index as f32 * line_step),
                color: text_color,
                size: Pixels(font_size),
                font,
                align_x: text_align,
                align_y: iced::alignment::Vertical::Center,
                ..Text::default()
            });
        }

        if has_url {
            draw_url_badge(frame, canvas, rect);
        }
        draw_toggle_button(frame, canvas, theme, node_layout.path.as_slice(), rect);
    }
}

fn draw_priority_badge(
    frame: &mut Frame,
    canvas: &MindMapCanvas<'_>,
    rect: Rectangle,
    priority: u8,
) {
    let pad = (8.0 * canvas.zoom).clamp(4.0, 10.0);
    let r = (9.0 * canvas.zoom).clamp(4.0, 12.0);
    let center = Point::new(rect.x + pad + r, rect.y + rect.height / 2.0);
    let circle = Path::circle(center, r);
    let background = priority_color(priority);
    frame.fill(&circle, background);
    frame.stroke(
        &circle,
        Stroke::default()
            .with_color(Color::from_rgba8(0, 0, 0, 0.12))
            .with_width((1.0 * canvas.zoom).clamp(0.8, 1.6)),
    );

    if (1..=9).contains(&priority) {
        frame.fill_text(Text {
            content: priority.to_string(),
            position: center,
            color: ideal_text_color(background),
            size: Pixels((12.0 * canvas.zoom).clamp(9.0, 18.0)),
            align_x: iced::widget::text::Alignment::Center,
            align_y: iced::alignment::Vertical::Center,
            ..Text::default()
        });
    } else if priority == 10 {
        let icon_size = (r * 1.15).clamp(6.0, 14.0);
        let origin = Point::new(center.x - icon_size / 2.0, center.y - icon_size / 2.0);
        let scale = icon_size / 16.0;
        let check_geometry =
            "M10.97 4.97a.75.75 0 0 1 1.07 1.05l-3.99 4.99a.75.75 0 0 1-1.08.02L4.324 8.384a.75.75 0 1 1 1.06-1.06l2.094 2.093 3.473-4.425z";
        if let Some(path) = build_svg_path(check_geometry, origin, scale) {
            frame.fill(&path, Color::WHITE);
        }
    }
}

fn draw_url_badge(frame: &mut Frame, canvas: &MindMapCanvas<'_>, rect: Rectangle) {
    let pad = (8.0 * canvas.zoom).clamp(4.0, 10.0);
    let r = (8.0 * canvas.zoom).clamp(4.0, 12.0);
    let center = Point::new(rect.x + rect.width - pad - r, rect.y + rect.height / 2.0);
    let circle = Path::circle(center, r);
    frame.fill(&circle, Color::from_rgba8(107, 114, 128, 1.0));
    frame.stroke(
        &circle,
        Stroke::default()
            .with_color(Color::from_rgba8(0, 0, 0, 0.12))
            .with_width((1.0 * canvas.zoom).clamp(0.8, 1.6)),
    );

    let icon_size = (r * 1.25).clamp(7.0, 16.0);
    let origin = Point::new(center.x - icon_size / 2.0, center.y - icon_size / 2.0);
    let scale = icon_size / 16.0;
    let link_geometry =
        "M6.354 5.5H4a3 3 0 0 0 0 6h3a3 3 0 0 0 2.83-4H9q-.13 0-.25.031A2 2 0 0 1 7 10.5H4a2 2 0 1 1 0-4h1.535c.218-.376.495-.714.82-1z M9 5.5a3 3 0 0 0-2.83 4h1.098A2 2 0 0 1 9 6.5h3a2 2 0 1 1 0-4h-1.535a4 4 0 0 1-.82 1H12a3 3 0 1 0 0-6z";
    if let Some(path) = build_svg_path(link_geometry, origin, scale) {
        frame.fill(&path, Color::WHITE);
    }
}

fn draw_toggle_button(
    frame: &mut Frame,
    canvas: &MindMapCanvas<'_>,
    theme: &Theme,
    node_path: &[usize],
    rect: Rectangle,
) {
    let Some(node) = mind_map::node(canvas.doc, node_path) else {
        return;
    };
    if node.children.is_empty() {
        return;
    }

    let collapsed = canvas.collapsed_paths.contains(node_path);
    let palette = theme.palette();
    let toggle = canvas
        .node_button_specs(node_path, rect)
        .into_iter()
        .find(|(kind, _, _)| *kind == HoverButtonKind::ToggleCollapse);
    let Some((_, center, r)) = toggle else {
        return;
    };

    let circle = Path::circle(center, r);
    frame.fill(&circle, Color::WHITE);
    frame.stroke(
        &circle,
        Stroke::default()
            .with_color(Color::from_rgba8(0, 0, 0, 0.14))
            .with_width((1.0 * canvas.zoom).clamp(0.9, 1.6)),
    );
    frame.fill_text(Text {
        content: if collapsed { "＋" } else { "−" }.to_string(),
        position: center,
        color: palette.text,
        size: Pixels((13.0 * canvas.zoom).clamp(10.0, 18.0)),
        align_x: iced::widget::text::Alignment::Center,
        align_y: iced::alignment::Vertical::Center,
        ..Text::default()
    });

    if !collapsed {
        return;
    }

    let child_count = descendant_count(node);
    if child_count == 0 {
        return;
    }

    let badge_rect = canvas.collapsed_count_badge_rect(node_path, center, r, child_count);
    let badge_path = Path::rounded_rectangle(
        Point::new(badge_rect.x, badge_rect.y),
        Size::new(badge_rect.width, badge_rect.height),
        (badge_rect.height / 2.0).into(),
    );
    frame.fill(&badge_path, Color::from_rgba8(17, 24, 39, 0.78));
    frame.stroke(
        &badge_path,
        Stroke::default()
            .with_color(Color::from_rgba8(0, 0, 0, 0.12))
            .with_width((1.0 * canvas.zoom).clamp(0.8, 1.6)),
    );
    frame.fill_text(Text {
        content: format!("+{child_count}"),
        position: Point::new(
            badge_rect.x + badge_rect.width / 2.0,
            badge_rect.y + badge_rect.height / 2.0,
        ),
        color: Color::WHITE,
        size: Pixels((11.0 * canvas.zoom).clamp(9.0, 16.0)),
        align_x: iced::widget::text::Alignment::Center,
        align_y: iced::alignment::Vertical::Center,
        ..Text::default()
    });
}

fn dashed_stroke(style: EdgeStyle, color: Color, width: f32, zoom: f32) -> Stroke<'static> {
    let mut stroke = Stroke {
        style: color.into(),
        width,
        ..Stroke::default()
    };
    let dash_segments = dash_segments_px(style, zoom);
    if let Some(segments) = dash_segments.as_ref() {
        stroke.line_dash = LineDash { segments, offset: 0 };
        if style == EdgeStyle::Dotted {
            stroke.line_cap = LineCap::Round;
        }
    }
    stroke
}
