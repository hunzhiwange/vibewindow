//! 思维导图画布交互更新逻辑，负责拖拽、缩放、选择和涂鸦事件转发。

use super::super::layout::{compute_layout_for_diagram, layout_node_rect};
use super::super::transform::{screen_from_world, world_from_screen};
use crate::app::Message;
use crate::apps::mindmap::message::MindMapMessage;
use crate::apps::mindmap::state::{MindMapCanvasTool, MindMapDoodleStroke};
use iced::widget::canvas::{Action, Event};
use iced::{Point, Rectangle, Size, Vector, mouse};
use std::time::Duration;
use web_time::Instant;

use super::ui::cursor_in_blocked_ui;
use super::{DragMode, ERASER_RADIUS_PX, MindMapCanvas, MindMapCanvasState};

/// 构建或更新 update 相关行为。
///
/// 参数由调用方提供，返回值直接交给上层视图或状态处理；本函数不执行额外错误恢复。
pub(super) fn update(
    canvas: &MindMapCanvas<'_>,
    state: &mut MindMapCanvasState,
    event: &Event,
    bounds: Rectangle,
    cursor: mouse::Cursor,
) -> Option<Action<Message>> {
    let cursor_pos_in_bounds = cursor.position_in(bounds);
    let cursor_in_blocked_ui = cursor_in_blocked_ui(
        bounds,
        cursor_pos_in_bounds,
        &state.drag_mode,
        canvas.canvas_tool,
        &canvas.ui_blocked_rects,
    );

    match event {
        Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
            if cursor_in_blocked_ui {
                return None;
            }
            if canvas.theme_panel_open {
                return Some(Action::publish(Message::MindMapTool(MindMapMessage::ClosePickers)));
            }
            let cursor_pos = cursor_pos_in_bounds?;
            state.last_cursor = Some(cursor_pos);
            let now = Instant::now();

            match canvas.canvas_tool {
                MindMapCanvasTool::Pen => {
                    state.drag_mode = DragMode::DoodlePen;
                    state.doodle_points_world =
                        vec![world_from_screen(cursor_pos, canvas.pan, canvas.zoom)];
                    return Some(Action::capture());
                }
                MindMapCanvasTool::Eraser => {
                    state.drag_mode = DragMode::DoodleErase;
                    state.doodle_points_world.clear();
                    let p = world_from_screen(cursor_pos, canvas.pan, canvas.zoom);
                    let radius_world = ERASER_RADIUS_PX / canvas.zoom.max(0.0001);
                    return Some(Action::publish(Message::MindMapTool(
                        MindMapMessage::DoodleErase(p, radius_world),
                    )));
                }
                MindMapCanvasTool::Pan => {
                    state.drag_mode = DragMode::Pan;
                    return Some(Action::capture());
                }
                MindMapCanvasTool::Select => {}
            }

            let layout = compute_layout_for_diagram(
                canvas.doc,
                canvas.node_positions,
                canvas.node_priorities,
                canvas.node_urls,
                canvas.collapsed_paths,
                canvas.diagram_type,
                canvas.layout_format,
                canvas.org_chart_layout_format,
                canvas.fishbone_layout_format,
                canvas.timeline_layout_format,
                canvas.bracket_layout_format,
                canvas.tree_layout_format,
            );

            if matches!(state.drag_mode, DragMode::None)
                && let Some(msg) = canvas.hit_node_buttons(&layout, cursor_pos)
            {
                state.drag_mode = DragMode::None;
                state.last_cursor = None;
                state.hovered_node = None;
                return Some(Action::publish(Message::MindMapTool(msg)));
            }

            for n in &layout.nodes {
                let Some(url) = canvas.node_urls.get(&n.path) else {
                    continue;
                };
                if url.trim().is_empty() {
                    continue;
                }
                let world_rect = layout_node_rect(n);
                let top_left = screen_from_world(
                    Point::new(world_rect.x, world_rect.y),
                    canvas.pan,
                    canvas.zoom,
                );
                let size =
                    Size::new(world_rect.width * canvas.zoom, world_rect.height * canvas.zoom);
                let rect = Rectangle::new(top_left, size);

                let r = (8.0 * canvas.zoom).clamp(4.0, 12.0);
                let pad = (8.0 * canvas.zoom).clamp(4.0, 10.0);
                let center = Point::new(rect.x + rect.width - pad - r, rect.y + rect.height / 2.0);
                let dx = cursor_pos.x - center.x;
                let dy = cursor_pos.y - center.y;
                if dx * dx + dy * dy <= r * r {
                    state.drag_mode = DragMode::None;
                    return Some(Action::publish(Message::MindMapTool(
                        MindMapMessage::OpenNodeUrlAt(n.path.clone()),
                    )));
                }
            }

            let world = world_from_screen(cursor_pos, canvas.pan, canvas.zoom);
            for n in &layout.nodes {
                let r = layout_node_rect(n);
                if r.contains(world) {
                    let is_double_click = state
                        .last_click_at
                        .map(|t| now.duration_since(t) <= Duration::from_millis(350))
                        .unwrap_or(false)
                        && state.last_click_node.as_ref() == Some(&n.path)
                        && state
                            .last_click_pos
                            .map(|p| {
                                let dx = p.x - cursor_pos.x;
                                let dy = p.y - cursor_pos.y;
                                dx * dx + dy * dy <= 6.0 * 6.0
                            })
                            .unwrap_or(false);

                    state.last_click_at = Some(now);
                    state.last_click_node = Some(n.path.clone());
                    state.last_click_pos = Some(cursor_pos);

                    if is_double_click {
                        state.drag_mode = DragMode::None;
                        state.last_cursor = None;
                        return Some(Action::publish(Message::MindMapTool(
                            MindMapMessage::ToggleNodeTextEditor,
                        )));
                    }
                    state.drag_mode = DragMode::Node(n.path.clone());
                    return Some(Action::publish(Message::MindMapTool(
                        MindMapMessage::NodeDragStart(n.path.clone(), n.pos, cursor_pos),
                    )));
                }
            }

            state.drag_mode = DragMode::Pan;
            Some(Action::capture())
        }
        Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Right)) => {
            if cursor_in_blocked_ui {
                return None;
            }
            if canvas.theme_panel_open {
                return Some(Action::publish(Message::MindMapTool(MindMapMessage::ClosePickers)));
            }
            let cursor_pos = cursor_pos_in_bounds?;

            let layout = compute_layout_for_diagram(
                canvas.doc,
                canvas.node_positions,
                canvas.node_priorities,
                canvas.node_urls,
                canvas.collapsed_paths,
                canvas.diagram_type,
                canvas.layout_format,
                canvas.org_chart_layout_format,
                canvas.fishbone_layout_format,
                canvas.timeline_layout_format,
                canvas.bracket_layout_format,
                canvas.tree_layout_format,
            );

            let world = world_from_screen(cursor_pos, canvas.pan, canvas.zoom);
            for n in &layout.nodes {
                let r = layout_node_rect(n);
                if r.contains(world) {
                    state.drag_mode = DragMode::None;
                    state.last_cursor = None;
                    return Some(Action::publish(Message::MindMapTool(
                        MindMapMessage::OpenNodeContextMenu(n.path.clone(), cursor_pos),
                    )));
                }
            }

            state.drag_mode = DragMode::None;
            state.last_cursor = None;
            Some(Action::publish(Message::MindMapTool(MindMapMessage::CloseContextMenu)))
        }
        Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
            if cursor_in_blocked_ui {
                return None;
            }
            match std::mem::replace(&mut state.drag_mode, DragMode::None) {
                DragMode::DoodlePen => {
                    state.last_cursor = None;
                    let points = std::mem::take(&mut state.doodle_points_world);
                    if points.len() >= 2 {
                        return Some(Action::publish(Message::MindMapTool(
                            MindMapMessage::DoodleCommit(MindMapDoodleStroke {
                                points_world: points,
                                rgba: canvas.doodle_rgba,
                                width_px: canvas.doodle_width_px,
                            }),
                        )));
                    }
                    Some(Action::capture())
                }
                DragMode::DoodleErase => {
                    state.last_cursor = None;
                    state.doodle_points_world.clear();
                    Some(Action::capture())
                }
                DragMode::Pan => {
                    state.last_cursor = None;
                    Some(Action::publish(Message::MindMapTool(MindMapMessage::ClearSelection)))
                }
                _ => {
                    state.last_cursor = None;
                    Some(Action::capture())
                }
            }
        }
        Event::Mouse(mouse::Event::CursorMoved { .. }) => {
            if cursor_in_blocked_ui {
                return None;
            }
            let cursor_pos = cursor_pos_in_bounds?;
            let last = state.last_cursor.unwrap_or(cursor_pos);
            state.last_cursor = Some(cursor_pos);
            let delta = Vector::new(cursor_pos.x - last.x, cursor_pos.y - last.y);
            match &state.drag_mode {
                DragMode::Pan => {
                    Some(Action::publish(Message::MindMapTool(MindMapMessage::PanBy(delta))))
                }
                DragMode::Node(path) => {
                    Some(Action::publish(Message::MindMapTool(MindMapMessage::NodeDragged(
                        path.clone(),
                        Vector::new(delta.x / canvas.zoom, delta.y / canvas.zoom),
                    ))))
                }
                DragMode::DoodlePen => {
                    let p = world_from_screen(cursor_pos, canvas.pan, canvas.zoom);
                    let add = state
                        .doodle_points_world
                        .last()
                        .copied()
                        .map(|last| {
                            let dx = p.x - last.x;
                            let dy = p.y - last.y;
                            dx * dx + dy * dy > (1.0 / canvas.zoom.max(0.0001)).powi(2)
                        })
                        .unwrap_or(true);
                    if add {
                        state.doodle_points_world.push(p);
                        Some(Action::request_redraw())
                    } else {
                        None
                    }
                }
                DragMode::DoodleErase => {
                    let p = world_from_screen(cursor_pos, canvas.pan, canvas.zoom);
                    let radius_world = ERASER_RADIUS_PX / canvas.zoom.max(0.0001);
                    Some(Action::publish(Message::MindMapTool(MindMapMessage::DoodleErase(
                        p,
                        radius_world,
                    ))))
                }
                DragMode::None => {
                    if canvas.canvas_tool != MindMapCanvasTool::Select {
                        return None;
                    }
                    let layout = compute_layout_for_diagram(
                        canvas.doc,
                        canvas.node_positions,
                        canvas.node_priorities,
                        canvas.node_urls,
                        canvas.collapsed_paths,
                        canvas.diagram_type,
                        canvas.layout_format,
                        canvas.org_chart_layout_format,
                        canvas.fishbone_layout_format,
                        canvas.timeline_layout_format,
                        canvas.bracket_layout_format,
                        canvas.tree_layout_format,
                    );
                    let hovered = canvas.hovered_node_path(&layout, cursor_pos);
                    if hovered != state.hovered_node {
                        state.hovered_node = hovered;
                        Some(Action::request_redraw())
                    } else {
                        None
                    }
                }
            }
        }
        Event::Mouse(mouse::Event::WheelScrolled { delta }) => {
            if cursor_in_blocked_ui {
                return None;
            }
            let dy = match delta {
                mouse::ScrollDelta::Lines { y, .. } => -(*y) * 60.0,
                mouse::ScrollDelta::Pixels { y, .. } => -(*y),
            };
            if dy == 0.0 {
                return None;
            }
            Some(Action::publish(Message::MindMapTool(MindMapMessage::PanBy(Vector::new(0.0, dy)))))
        }
        _ => None,
    }
}
