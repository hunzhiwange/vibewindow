//! 思维导图画布操作更新逻辑，处理节点选择、布局切换和画布参数变化。

use crate::app::{App, Message};
use crate::apps::mindmap::canvas::layout::{compute_layout, layout_bounds_world};
use crate::apps::mindmap::state::{MindMapCanvasTool, MindMapDoodleStroke};
use iced::widget::text_editor;
use iced::{Point, Task, Vector};

use super::super::persist::persist;

/// 构建或更新 pan by 相关行为。
///
/// 参数由调用方提供，返回值直接交给上层视图或状态处理；本函数不执行额外错误恢复。
pub(super) fn pan_by(app: &mut App, delta: Vector) -> Task<Message> {
    if let Some(tab) = app.active_mindmap_tab_mut() {
        tab.pan = Vector::new(tab.pan.x + delta.x, tab.pan.y + delta.y);
        tab.canvas_cache.clear();
        let _ = persist(app);
    }
    Task::none()
}

/// 构建或更新 zoom 相关行为。
///
/// 参数由调用方提供，返回值直接交给上层视图或状态处理；本函数不执行额外错误恢复。
pub(super) fn zoom(app: &mut App, factor: f32, center_opt: Option<Point>) -> Task<Message> {
    let (win_w, win_h) = app.window_size;
    if let Some(tab) = app.active_mindmap_tab_mut() {
        let old_zoom = tab.zoom.max(0.0001);
        let new_zoom = (old_zoom * factor).clamp(0.1, 10.0);

        let pt = center_opt.unwrap_or(Point::new(win_w / 2.0, win_h / 2.0));
        let p_screen = Vector::new(pt.x, pt.y);
        tab.pan = p_screen - (p_screen - tab.pan) * (new_zoom / old_zoom);

        tab.zoom = new_zoom;
        tab.canvas_cache.clear();
        let _ = persist(app);
    }
    Task::none()
}

/// 构建或更新 zoom set 相关行为。
///
/// 参数由调用方提供，返回值直接交给上层视图或状态处理；本函数不执行额外错误恢复。
pub(super) fn zoom_set(app: &mut App, zoom: f32) -> Task<Message> {
    let (win_w, win_h) = app.window_size;
    if let Some(tab) = app.active_mindmap_tab_mut() {
        let old_zoom = tab.zoom.max(0.0001);
        let new_zoom = zoom.clamp(0.1, 10.0);

        let pt = Point::new(win_w / 2.0, win_h / 2.0);
        let p_screen = Vector::new(pt.x, pt.y);
        tab.pan = p_screen - (p_screen - tab.pan) * (new_zoom / old_zoom);

        tab.zoom = new_zoom;
        tab.show_zoom_menu = false;
        tab.canvas_cache.clear();
        let _ = persist(app);
    }
    Task::none()
}

/// 构建或更新 zoom fit 相关行为。
///
/// 参数由调用方提供，返回值直接交给上层视图或状态处理；本函数不执行额外错误恢复。
pub(super) fn zoom_fit(app: &mut App) -> Task<Message> {
    let (win_w, win_h) = app.window_size;
    if let Some(tab) = app.active_mindmap_tab_mut() {
        let layout = compute_layout(
            &tab.doc,
            &tab.node_positions,
            &tab.node_priorities,
            &tab.node_urls,
            &tab.collapsed_paths,
            tab.layout_format,
        );
        if layout.nodes.is_empty() {
            tab.show_zoom_menu = false;
            return Task::none();
        }

        let bounds = layout_bounds_world(&layout);
        let pad: f32 = 90.0;
        let avail_w = (win_w - pad * 2.0).max(1.0);
        let avail_h = (win_h - pad * 2.0).max(1.0);
        let scale_x = if bounds.width > 0.0 { avail_w / bounds.width } else { 1.0 };
        let scale_y = if bounds.height > 0.0 { avail_h / bounds.height } else { 1.0 };

        let new_zoom = (scale_x.min(scale_y) * 0.95).clamp(0.1, 10.0);

        let world_center =
            Point::new(bounds.x + bounds.width / 2.0, bounds.y + bounds.height / 2.0);
        let screen_center = Vector::new(win_w / 2.0, win_h / 2.0);
        tab.pan = screen_center - Vector::new(world_center.x * new_zoom, world_center.y * new_zoom);

        tab.zoom = new_zoom;
        tab.show_zoom_menu = false;
        tab.canvas_cache.clear();
        let _ = persist(app);
    }
    Task::none()
}

/// 构建或更新 toggle zoom menu 相关行为。
///
/// 参数由调用方提供，返回值直接交给上层视图或状态处理；本函数不执行额外错误恢复。
pub(super) fn toggle_zoom_menu(app: &mut App) -> Task<Message> {
    if let Some(tab) = app.active_mindmap_tab_mut() {
        super::node_meta_ops::commit_url_editor_if_needed(tab);
        tab.show_zoom_menu = !tab.show_zoom_menu;
        if tab.show_zoom_menu {
            tab.active_color_picker = None;
            tab.show_diagram_type_picker = false;
            tab.show_markdown_import = false;
            tab.show_priority_picker = false;
            tab.show_url_editor = false;
            tab.show_text_editor = false;
            tab.url_editor_value.clear();
            tab.node_text_editor = text_editor::Content::new();
            tab.show_action_menu = false;
            tab.show_context_menu = false;
            tab.context_menu_anchor = None;
            tab.show_theme_panel = false;
        }
        tab.canvas_cache.clear();
        let _ = persist(app);
    }
    Task::none()
}

/// 构建或更新 node drag start 相关行为。
///
/// 参数由调用方提供，返回值直接交给上层视图或状态处理；本函数不执行额外错误恢复。
pub(super) fn node_drag_start(
    app: &mut App,
    path: Vec<usize>,
    pos: Point,
    click_screen: Point,
) -> Task<Message> {
    if let Some(tab) = app.active_mindmap_tab_mut() {
        super::node_meta_ops::commit_url_editor_if_needed(tab);
        tab.selected_path = Some(path.clone());
        tab.show_context_menu = false;
        tab.context_menu_anchor = None;
        tab.show_theme_panel = false;
        tab.show_text_editor = false;
        tab.node_text_editor = text_editor::Content::new();
        tab.node_positions.entry(path).or_insert(pos);
        tab.last_click_screen = Some(click_screen);
        tab.canvas_cache.clear();
        let _ = persist(app);
    }
    Task::none()
}

/// 构建或更新 node dragged 相关行为。
///
/// 参数由调用方提供，返回值直接交给上层视图或状态处理；本函数不执行额外错误恢复。
pub(super) fn node_dragged(app: &mut App, path: Vec<usize>, delta: Vector) -> Task<Message> {
    if let Some(tab) = app.active_mindmap_tab_mut() {
        if let Some(pt) = tab.node_positions.get_mut(&path) {
            pt.x += delta.x;
            pt.y += delta.y;
        }
        tab.canvas_cache.clear();
        let _ = persist(app);
    }
    Task::none()
}

/// 构建或更新 set canvas tool 相关行为。
///
/// 参数由调用方提供，返回值直接交给上层视图或状态处理；本函数不执行额外错误恢复。
pub(super) fn set_canvas_tool(app: &mut App, tool: MindMapCanvasTool) -> Task<Message> {
    if let Some(tab) = app.active_mindmap_tab_mut() {
        super::node_meta_ops::commit_url_editor_if_needed(tab);
        tab.canvas_tool = tool;
        tab.show_context_menu = false;
        tab.context_menu_anchor = None;
        tab.show_theme_panel = false;
        tab.show_text_editor = false;
        tab.node_text_editor = text_editor::Content::new();
        tab.canvas_cache.clear();
        let _ = persist(app);
    }
    Task::none()
}

/// 构建或更新 set doodle color 相关行为。
///
/// 参数由调用方提供，返回值直接交给上层视图或状态处理；本函数不执行额外错误恢复。
pub(super) fn set_doodle_color(app: &mut App, rgba: u32) -> Task<Message> {
    if let Some(tab) = app.active_mindmap_tab_mut() {
        tab.doodle_rgba = rgba;
        tab.canvas_cache.clear();
        let _ = persist(app);
    }
    Task::none()
}

/// 构建或更新 set doodle width 相关行为。
///
/// 参数由调用方提供，返回值直接交给上层视图或状态处理；本函数不执行额外错误恢复。
pub(super) fn set_doodle_width(app: &mut App, width_px: f32) -> Task<Message> {
    let w = width_px.clamp(1.0, 18.0);
    if let Some(tab) = app.active_mindmap_tab_mut() {
        tab.doodle_width_px = w;
        tab.canvas_cache.clear();
        let _ = persist(app);
    }
    Task::none()
}

/// 构建或更新 doodle commit 相关行为。
///
/// 参数由调用方提供，返回值直接交给上层视图或状态处理；本函数不执行额外错误恢复。
pub(super) fn doodle_commit(app: &mut App, stroke: MindMapDoodleStroke) -> Task<Message> {
    if stroke.points_world.len() < 2 {
        return Task::none();
    }
    if let Some(tab) = app.active_mindmap_tab_mut() {
        tab.doodles.push(stroke);
        tab.canvas_cache.clear();
        let _ = persist(app);
    }
    Task::none()
}

/// 构建或更新 doodle erase 相关行为。
///
/// 参数由调用方提供，返回值直接交给上层视图或状态处理；本函数不执行额外错误恢复。
pub(super) fn doodle_erase(app: &mut App, center_world: Point, radius_world: f32) -> Task<Message> {
    let r = radius_world.max(0.0);
    if r <= 0.0 {
        return Task::none();
    }

    if let Some(tab) = app.active_mindmap_tab_mut() {
        let r2 = r * r;
        let mut out = Vec::new();
        let mut changed = false;

        for s in &tab.doodles {
            let mut current = Vec::new();
            for p in &s.points_world {
                let dx = p.x - center_world.x;
                let dy = p.y - center_world.y;
                if dx * dx + dy * dy > r2 {
                    current.push(*p);
                } else {
                    changed = true;
                    if current.len() >= 2 {
                        out.push(MindMapDoodleStroke {
                            points_world: std::mem::take(&mut current),
                            rgba: s.rgba,
                            width_px: s.width_px,
                        });
                    } else {
                        current.clear();
                    }
                }
            }
            if current.len() >= 2 {
                out.push(MindMapDoodleStroke {
                    points_world: current,
                    rgba: s.rgba,
                    width_px: s.width_px,
                });
            }
        }

        if changed {
            tab.doodles = out;
            tab.canvas_cache.clear();
            let _ = persist(app);
        }
    }
    Task::none()
}
