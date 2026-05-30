//! 思维导图画布边线绘制逻辑，负责不同图形类型的连接线形态。

use std::collections::HashMap;

use crate::apps::mindmap::canvas::layout::{Layout, layout_node_rect};
use crate::apps::mindmap::canvas::style::{dash_segments_px, rgba_u32_to_color};
use crate::apps::mindmap::canvas::transform::screen_from_world;
use crate::apps::mindmap::state::{
    BracketLayoutFormat, EdgeStyle, FishboneLayoutFormat, MindMapDiagramType, OrgChartLayoutFormat,
    TreeLayoutFormat,
};
use iced::widget::canvas::{Frame, LineCap, LineDash, Path, Stroke};
use iced::{Color, Point, Rectangle, Size};

use super::{FishboneMeta, MindMapCanvas, ThemeView};

/// 构建或更新 draw special diagram backdrop 相关行为。
///
/// 参数由调用方提供，返回值直接交给上层视图或状态处理；本函数不执行额外错误恢复。
pub(super) fn draw_special_diagram_backdrop(
    frame: &mut Frame,
    canvas: &MindMapCanvas<'_>,
    layout: &Layout,
    current_theme: &ThemeView<'_>,
    stroke_color: Color,
    stroke_width: f32,
) {
    if canvas.diagram_type != MindMapDiagramType::Fishbone {
        return;
    }

    let Some(root) = layout.nodes.iter().find(|node| node.path.is_empty()) else {
        return;
    };
    let root_rect = layout_node_rect(root);
    let spine_dir = match canvas.fishbone_layout_format {
        FishboneLayoutFormat::HeadRight => -1.0,
        FishboneLayoutFormat::HeadLeft => 1.0,
    };

    let mut extreme_x = root.pos.x + spine_dir * 480.0;
    for node in layout.nodes.iter().filter(|node| node.path.len() == 1) {
        if spine_dir < 0.0 {
            extreme_x = extreme_x.min(node.pos.x);
        } else {
            extreme_x = extreme_x.max(node.pos.x);
        }
    }

    let tail_x = extreme_x + spine_dir * 320.0;
    let spine_y = root.pos.y;
    let apex_x = if spine_dir < 0.0 { root_rect.x } else { root_rect.x + root_rect.width };
    let apex = Point::new(apex_x, spine_y);
    let base = Point::new(apex.x + spine_dir * 18.0, spine_y);
    let tail = Point::new(tail_x, spine_y);
    let spine_color =
        current_theme.line_color.map(rgba_u32_to_color).unwrap_or(stroke_color).scale_alpha(0.55);

    let tail_s = screen_from_world(tail, canvas.pan, canvas.zoom);
    let base_s = screen_from_world(base, canvas.pan, canvas.zoom);
    let apex_s = screen_from_world(apex, canvas.pan, canvas.zoom);
    frame.stroke(
        &Path::line(tail_s, base_s),
        Stroke::default().with_width(stroke_width).with_color(spine_color),
    );

    let half_w = (7.0 * canvas.zoom).clamp(4.0, 10.0);
    let base_up_s = screen_from_world(
        Point::new(base.x, base.y - half_w / canvas.zoom),
        canvas.pan,
        canvas.zoom,
    );
    let base_dn_s = screen_from_world(
        Point::new(base.x, base.y + half_w / canvas.zoom),
        canvas.pan,
        canvas.zoom,
    );
    let arrow = Path::new(|builder| {
        builder.move_to(apex_s);
        builder.line_to(base_up_s);
        builder.line_to(base_dn_s);
        builder.close();
    });
    frame.fill(&arrow, spine_color.scale_alpha(0.85));
}

/// 构建或更新 fishbone meta 相关行为。
///
/// 参数由调用方提供，返回值直接交给上层视图或状态处理；本函数不执行额外错误恢复。
pub(super) fn fishbone_meta(canvas: &MindMapCanvas<'_>, layout: &Layout) -> Option<FishboneMeta> {
    if canvas.diagram_type != MindMapDiagramType::Fishbone {
        return None;
    }

    let root = layout.nodes.iter().find(|node| node.path.is_empty())?;
    let root_rect = layout_node_rect(root);
    let spine_dir = match canvas.fishbone_layout_format {
        FishboneLayoutFormat::HeadRight => -1.0,
        FishboneLayoutFormat::HeadLeft => 1.0,
    };
    Some((root.pos, root.size, root_rect, spine_dir))
}

/// 构建或更新 draw edges 相关行为。
///
/// 参数由调用方提供，返回值直接交给上层视图或状态处理；本函数不执行额外错误恢复。
pub(super) fn draw_edges(
    frame: &mut Frame,
    canvas: &MindMapCanvas<'_>,
    layout: &Layout,
    current_theme: &ThemeView<'_>,
    stroke_color: Color,
    stroke_width: f32,
    fishbone_meta: Option<&FishboneMeta>,
) {
    if canvas.diagram_type == MindMapDiagramType::Bracket {
        draw_bracket_edges(frame, canvas, layout, current_theme, stroke_width);
        return;
    }

    for edge in &layout.edges {
        let from = layout.nodes.iter().find(|node| node.path == edge.from);
        let to = layout.nodes.iter().find(|node| node.path == edge.to);
        let (Some(a), Some(b)) = (from, to) else {
            continue;
        };

        let a_rect = layout_node_rect(a);
        let b_rect = layout_node_rect(b);
        let (start_world, end_world) =
            edge_endpoints(canvas, edge, a, b, a_rect, b_rect, fishbone_meta);

        let start = screen_from_world(start_world, canvas.pan, canvas.zoom);
        let end = screen_from_world(end_world, canvas.pan, canvas.zoom);
        let path = edge_path(canvas, edge, start, end, fishbone_meta.is_some());
        let style = canvas.edge_styles.get(&edge.to).copied().unwrap_or(canvas.edge_style);
        let color = edge_color(canvas, current_theme, &edge.to, stroke_color);
        frame.stroke(&path, edge_stroke(style, color, stroke_width, canvas.zoom));
    }
}

#[derive(Clone)]
struct BracketChild {
    path: Vec<usize>,
    rect: Rectangle,
    center_y: f32,
}

fn draw_bracket_edges(
    frame: &mut Frame,
    canvas: &MindMapCanvas<'_>,
    layout: &Layout,
    current_theme: &ThemeView<'_>,
    stroke_width: f32,
) {
    let mut children_by_parent: HashMap<Vec<usize>, Vec<Vec<usize>>> = HashMap::new();
    for edge in &layout.edges {
        children_by_parent.entry(edge.from.clone()).or_default().push(edge.to.clone());
    }

    let gap = (14.0 * canvas.zoom).clamp(10.0, 26.0);
    let prefer_on_right = canvas.bracket_layout_format == BracketLayoutFormat::BraceRight;

    for (parent_path, child_paths) in children_by_parent {
        let Some(parent_node) = layout.nodes.iter().find(|node| node.path == parent_path) else {
            continue;
        };

        let parent_world = layout_node_rect(parent_node);
        let parent_top_left =
            screen_from_world(Point::new(parent_world.x, parent_world.y), canvas.pan, canvas.zoom);
        let parent_rect = Rectangle::new(
            parent_top_left,
            Size::new(parent_world.width * canvas.zoom, parent_world.height * canvas.zoom),
        );
        let parent_center_y = parent_rect.y + parent_rect.height / 2.0;
        let mut children = Vec::new();

        for child_path in child_paths {
            let Some(child_node) = layout.nodes.iter().find(|node| node.path == child_path) else {
                continue;
            };

            let child_world = layout_node_rect(child_node);
            let child_top_left = screen_from_world(
                Point::new(child_world.x, child_world.y),
                canvas.pan,
                canvas.zoom,
            );
            let child_rect = Rectangle::new(
                child_top_left,
                Size::new(child_world.width * canvas.zoom, child_world.height * canvas.zoom),
            );
            children.push(BracketChild {
                path: child_node.path.clone(),
                rect: child_rect,
                center_y: child_rect.y + child_rect.height / 2.0,
            });
        }

        children.sort_by(|a, b| a.center_y.total_cmp(&b.center_y));
        draw_bracket_group(
            frame,
            canvas,
            current_theme,
            stroke_width,
            gap,
            parent_rect,
            parent_center_y,
            &children,
            prefer_on_right,
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_bracket_group(
    frame: &mut Frame,
    canvas: &MindMapCanvas<'_>,
    current_theme: &ThemeView<'_>,
    stroke_width: f32,
    gap: f32,
    parent_rect: Rectangle,
    parent_center_y: f32,
    children: &[BracketChild],
    on_right: bool,
) {
    if children.is_empty() {
        return;
    }

    let first_path = &children[0].path;
    let style = canvas.edge_styles.get(first_path).copied().unwrap_or(canvas.edge_style);
    let color = edge_color(canvas, current_theme, first_path, Color::BLACK);
    let stroke = edge_stroke(style, color, stroke_width, canvas.zoom);
    let parent_edge_x = if on_right { parent_rect.x + parent_rect.width } else { parent_rect.x };

    if children.len() == 1 {
        let child = &children[0];
        let child_edge_x = if on_right { child.rect.x } else { child.rect.x + child.rect.width };
        let start = Point::new(parent_edge_x, parent_center_y);
        let end = Point::new(child_edge_x, child.center_y);
        frame.stroke(&Path::line(start, end), stroke);
        return;
    }

    let mut y_top = f32::INFINITY;
    let mut y_bottom = f32::NEG_INFINITY;
    for child in children {
        y_top = y_top.min(child.rect.y);
        y_bottom = y_bottom.max(child.rect.y + child.rect.height);
    }
    if !y_top.is_finite() {
        return;
    }

    let y_mid = (y_top + y_bottom) / 2.0;
    let x0 = if on_right { parent_rect.x + parent_rect.width + gap } else { parent_rect.x - gap };
    let concave_dir = if on_right { 1.0 } else { -1.0 };
    let h = (y_bottom - y_top).max(1.0);
    let w = (10.0 * canvas.zoom).clamp(8.0, 22.0);
    let notch_dy = (h * 0.07).clamp(8.0, 18.0);
    let notch_x = x0 + concave_dir * w;
    let bulge_x = x0 - concave_dir * w;

    let brace = Path::new(|builder| {
        builder.move_to(Point::new(x0, y_top));
        builder.bezier_curve_to(
            Point::new(bulge_x, y_top),
            Point::new(bulge_x, y_mid - notch_dy),
            Point::new(x0, y_mid - notch_dy),
        );
        builder.bezier_curve_to(
            Point::new(x0, y_mid - notch_dy * 0.5),
            Point::new(notch_x, y_mid - notch_dy * 0.5),
            Point::new(notch_x, y_mid),
        );
        builder.bezier_curve_to(
            Point::new(notch_x, y_mid + notch_dy * 0.5),
            Point::new(x0, y_mid + notch_dy * 0.5),
            Point::new(x0, y_mid + notch_dy),
        );
        builder.bezier_curve_to(
            Point::new(bulge_x, y_mid + notch_dy),
            Point::new(bulge_x, y_bottom),
            Point::new(x0, y_bottom),
        );
    });
    frame.stroke(&brace, stroke);

    let connector = Path::new(|builder| {
        builder.move_to(Point::new(parent_edge_x, parent_center_y));
        builder.line_to(Point::new(x0, parent_center_y));
        builder.line_to(Point::new(x0, y_mid));
    });
    frame.stroke(&connector, stroke);

    for child in children {
        let child_edge_x = if on_right { child.rect.x } else { child.rect.x + child.rect.width };
        let start = Point::new(notch_x, child.center_y);
        let end = Point::new(child_edge_x, child.center_y);
        frame.stroke(&Path::line(start, end), stroke);
    }
}

fn edge_endpoints(
    canvas: &MindMapCanvas<'_>,
    edge: &crate::apps::mindmap::canvas::layout::EdgeLayout,
    a: &crate::apps::mindmap::canvas::layout::NodeLayout,
    b: &crate::apps::mindmap::canvas::layout::NodeLayout,
    a_rect: Rectangle,
    b_rect: Rectangle,
    fishbone_meta: Option<&FishboneMeta>,
) -> (Point, Point) {
    if let Some((root_pos, root_size, _root_rect, spine_dir)) = fishbone_meta {
        let spine_y = root_pos.y;
        let base_branch_dx = 160.0f32;
        let to_len = edge.to.len();
        let from_len = edge.from.len();

        if to_len == 1 {
            let branch_dx = base_branch_dx.max(root_size.width / 2.0 + b.size.width / 2.0 + 140.0);
            let spine_x = b.pos.x - spine_dir * branch_dx;
            let start_world = Point::new(spine_x, spine_y);
            let end_world = if *spine_dir > 0.0 {
                Point::new(b_rect.x, b.pos.y)
            } else {
                Point::new(b_rect.x + b_rect.width, b.pos.y)
            };
            return (start_world, end_world);
        }

        if from_len == 1 && to_len == 2 {
            let branch_dx = base_branch_dx.max(root_size.width / 2.0 + a.size.width / 2.0 + 140.0);
            let spine_x = a.pos.x - spine_dir * branch_dx;
            let parent_attach_x = if *spine_dir > 0.0 { a_rect.x } else { a_rect.x + a_rect.width };
            let y = b.pos.y;
            let denom = a.pos.y - spine_y;
            let t = if denom.abs() < 1.0 { 1.0 } else { ((y - spine_y) / denom).clamp(0.0, 1.0) };
            let rib_x = spine_x + t * (parent_attach_x - spine_x);
            let start_world = Point::new(rib_x, y);
            let end_world = if *spine_dir > 0.0 {
                Point::new(b_rect.x, y)
            } else {
                Point::new(b_rect.x + b_rect.width, y)
            };
            return (start_world, end_world);
        }

        let start_world = if *spine_dir > 0.0 {
            Point::new(a_rect.x + a_rect.width, a.pos.y)
        } else {
            Point::new(a_rect.x, a.pos.y)
        };
        let end_world = if *spine_dir > 0.0 {
            Point::new(b_rect.x, b.pos.y)
        } else {
            Point::new(b_rect.x + b_rect.width, b.pos.y)
        };
        return (start_world, end_world);
    }

    match canvas.diagram_type {
        MindMapDiagramType::OrgChart
            if matches!(
                canvas.org_chart_layout_format,
                OrgChartLayoutFormat::TopDown | OrgChartLayoutFormat::LeftRight,
            ) =>
        {
            (Point::new(a.pos.x, a_rect.y + a_rect.height), Point::new(b.pos.x, b_rect.y))
        }
        MindMapDiagramType::Timeline if edge.from.len() == 1 && edge.to.len() >= 2 => {
            let up = b.pos.y < a.pos.y;
            let attach_y = if up { a_rect.y } else { a_rect.y + a_rect.height };
            let end_x = if b.pos.x >= a.pos.x { b_rect.x } else { b_rect.x + b_rect.width };
            (Point::new(a.pos.x, attach_y), Point::new(end_x, b.pos.y))
        }
        MindMapDiagramType::Tree => {
            (Point::new(a.pos.x, a_rect.y + a_rect.height), Point::new(b.pos.x, b_rect.y))
        }
        _ => {
            let to_right = b.pos.x >= a.pos.x;
            let start_world = if to_right {
                Point::new(a_rect.x + a_rect.width, a.pos.y)
            } else {
                Point::new(a_rect.x, a.pos.y)
            };
            let end_world = if to_right {
                Point::new(b_rect.x, b.pos.y)
            } else {
                Point::new(b_rect.x + b_rect.width, b.pos.y)
            };
            (start_world, end_world)
        }
    }
}

fn edge_path(
    canvas: &MindMapCanvas<'_>,
    edge: &crate::apps::mindmap::canvas::layout::EdgeLayout,
    start: Point,
    end: Point,
    is_fishbone: bool,
) -> Path {
    if is_fishbone {
        return Path::line(start, end);
    }

    if canvas.diagram_type == MindMapDiagramType::Timeline
        && edge.from.len() == 1
        && edge.to.len() >= 2
    {
        let pivot = Point::new(start.x, end.y);
        return Path::new(|builder| {
            builder.move_to(start);
            builder.line_to(pivot);
            builder.line_to(end);
        });
    }

    if canvas.diagram_type == MindMapDiagramType::Tree {
        return match canvas.tree_layout_format {
            TreeLayoutFormat::SymmetricSplit
            | TreeLayoutFormat::LeftAligned
            | TreeLayoutFormat::RightAligned => {
                let mid_y = end.y;
                let p1 = Point::new(start.x, mid_y);
                let p2 = Point::new(end.x, mid_y);
                Path::new(|builder| {
                    builder.move_to(start);
                    builder.line_to(p1);
                    builder.line_to(p2);
                    builder.line_to(end);
                })
            }
            TreeLayoutFormat::FanDown => {
                let dy = (end.y - start.y).abs();
                let control = (dy * 0.5).clamp(40.0, 220.0);
                let dir = if end.y >= start.y { 1.0 } else { -1.0 };
                let c1 = Point::new(start.x, start.y + control * dir);
                let c2 = Point::new(end.x, end.y - control * dir);
                Path::new(|builder| {
                    builder.move_to(start);
                    builder.bezier_curve_to(c1, c2, end);
                })
            }
        };
    }

    if matches!(canvas.diagram_type, MindMapDiagramType::OrgChart)
        && canvas.org_chart_layout_format == OrgChartLayoutFormat::TopDown
    {
        let dy = (end.y - start.y).abs();
        let control = (dy * 0.5).clamp(40.0, 220.0);
        let dir = if end.y >= start.y { 1.0 } else { -1.0 };
        let c1 = Point::new(start.x, start.y + control * dir);
        let c2 = Point::new(end.x, end.y - control * dir);
        return Path::new(|builder| {
            builder.move_to(start);
            builder.bezier_curve_to(c1, c2, end);
        });
    }

    if matches!(canvas.diagram_type, MindMapDiagramType::OrgChart)
        && canvas.org_chart_layout_format == OrgChartLayoutFormat::LeftRight
    {
        let mid_y = (start.y + end.y) / 2.0;
        let p1 = Point::new(start.x, mid_y);
        let p2 = Point::new(end.x, mid_y);
        return Path::new(|builder| {
            builder.move_to(start);
            builder.line_to(p1);
            builder.line_to(p2);
            builder.line_to(end);
        });
    }

    let dx = (end.x - start.x).abs();
    let control = (dx * 0.5).clamp(40.0, 220.0);
    let dir = if end.x >= start.x { 1.0 } else { -1.0 };
    let c1 = Point::new(start.x + control * dir, start.y);
    let c2 = Point::new(end.x - control * dir, end.y);
    Path::new(|builder| {
        builder.move_to(start);
        builder.bezier_curve_to(c1, c2, end);
    })
}

fn edge_color(
    canvas: &MindMapCanvas<'_>,
    current_theme: &ThemeView<'_>,
    path: &[usize],
    stroke_color: Color,
) -> Color {
    canvas.edge_colors.get(path).copied().map(rgba_u32_to_color).unwrap_or_else(|| {
        if let Some(color) = current_theme.line_color {
            rgba_u32_to_color(color)
        } else if path.is_empty() {
            stroke_color
        } else {
            rgba_u32_to_color(current_theme.palette(path[0]))
        }
    })
}

fn edge_stroke(style: EdgeStyle, color: Color, width: f32, zoom: f32) -> Stroke<'static> {
    let mut stroke = Stroke { style: color.into(), width, ..Stroke::default() };
    let dash_segments = dash_segments_px(style, zoom);
    if let Some(segments) = dash_segments.as_ref() {
        stroke.line_dash = LineDash { segments, offset: 0 };
        if style == EdgeStyle::Dotted {
            stroke.line_cap = LineCap::Round;
        }
    }
    stroke
}
