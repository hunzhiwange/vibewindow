//! 设计画布程序节点模块。
//!
//! 该模块处理程序节点展示与提示信息，让画布能够表达可执行或可交互的设计元素。

use super::super::models::{DesignDoc, DesignTool};
use super::geometry::{get_element_screen_bounds, rotate_point};
use super::hit::hit_test_handle;
use super::rendering::{
    draw_brush_preview_overlay, draw_eraser_overlay, draw_grid, draw_hover_edit_overlay,
    draw_selection_box, draw_selection_overlay, draw_shapes_tree, draw_texts_tree,
    draw_tool_preview_overlay,
};
use super::types::{DesignCanvasState, Handle};
use super::utils::find_element_by_id;
use crate::app::Message;
use crate::app::views::design::properties::fill::types::{FillItem, FillObject};
use iced::{
    Color, Pixels, Point, Rectangle, Renderer, Size, Theme, Vector, mouse,
    widget::canvas::{self, Action, Cache, Event, Geometry, Path, Stroke, Text},
};
use std::borrow::Cow;
use std::collections::HashSet;

mod mesh;
mod selection;
mod tooltip;
mod update;

const FRAME_HEADER_OFFSET_Y: f32 = 36.0;
const FRAME_HEADER_HEIGHT: f32 = 26.0;
const FRAME_HEADER_BUTTON_SIZE: f32 = 26.0;
const FRAME_HEADER_FONT_SIZE: f32 = 15.0;
const FRAME_HEADER_MIN_WIDTH: f32 = 104.0;
const FRAME_HEADER_HORIZONTAL_PADDING: f32 = 8.0;
const FRAME_HEADER_TEXT_RIGHT_PADDING: f32 = 14.0;
const FRAME_HEADER_GAP: f32 = 6.0;

#[derive(Clone, Copy)]
struct FrameHeaderLayout {
    btn_rect: Rectangle,
    title_rect: Rectangle,
}

/// 模块内部可见的 FrameHeaderHit 枚举，描述该模块支持的一组离散状态或事件。
pub(super) enum FrameHeaderHit<'a> {
    Title { id: &'a str, rect: Rectangle },
    Fit { id: &'a str, rect: Rectangle },
}

fn estimate_text_width(label: &str, font_size: f32) -> f32 {
    label.chars().map(|ch| if ch.is_ascii() { font_size * 0.56 } else { font_size * 0.94 }).sum()
}

fn color_luminance(color: Color) -> f32 {
    (color.r * 0.2126) + (color.g * 0.7152) + (color.b * 0.0722)
}

fn mix_color(base: Color, overlay: Color, amount: f32) -> Color {
    let t = amount.clamp(0.0, 1.0);
    Color {
        r: base.r + (overlay.r - base.r) * t,
        g: base.g + (overlay.g - base.g) * t,
        b: base.b + (overlay.b - base.b) * t,
        a: base.a + (overlay.a - base.a) * t,
    }
}

fn frame_header_label(label: Option<&str>) -> &str {
    label.filter(|value| !value.trim().is_empty()).unwrap_or("画板")
}

fn frame_header_tooltip_label(active_tool: DesignTool, base: &'static str) -> &'static str {
    if active_tool == DesignTool::Move {
        base
    } else {
        match base {
            "点击适配画板" => "点击适配画板（移动工具）",
            "点击选中页面" => "点击选中页面（移动工具）",
            _ => base,
        }
    }
}

fn frame_header_layout(rect: Rectangle, label: &str) -> FrameHeaderLayout {
    let row_y = rect.y - FRAME_HEADER_OFFSET_Y;
    let desired_title_width = estimate_text_width(label, FRAME_HEADER_FONT_SIZE)
        + FRAME_HEADER_HORIZONTAL_PADDING * 2.0
        + 4.0;
    let title_max = (rect.width - FRAME_HEADER_BUTTON_SIZE - FRAME_HEADER_GAP).max(56.0);
    let title_width = if title_max <= FRAME_HEADER_MIN_WIDTH {
        title_max
    } else {
        desired_title_width.clamp(FRAME_HEADER_MIN_WIDTH, title_max)
    };
    let btn_rect = Rectangle::new(
        Point::new(rect.x, row_y),
        Size::new(FRAME_HEADER_BUTTON_SIZE, FRAME_HEADER_BUTTON_SIZE),
    );
    let title_rect = Rectangle::new(
        Point::new(btn_rect.x + btn_rect.width + FRAME_HEADER_GAP, row_y),
        Size::new(title_width, FRAME_HEADER_HEIGHT),
    );

    FrameHeaderLayout { btn_rect, title_rect }
}

fn draw_frame_header(
    frame: &mut canvas::Frame,
    layout: FrameHeaderLayout,
    label: &str,
    background: Color,
    text_color: Color,
    accent: Color,
    selected: bool,
) {
    let is_dark = color_luminance(background) < 0.45;
    let neutral_surface = if is_dark {
        mix_color(background, Color::WHITE, 0.1)
    } else {
        mix_color(background, Color::BLACK, 0.045)
    };
    let surface = if selected {
        mix_color(neutral_surface, accent, if is_dark { 0.08 } else { 0.045 })
    } else {
        neutral_surface
    };
    let border = if selected {
        accent.scale_alpha(if is_dark { 0.42 } else { 0.28 })
    } else {
        text_color.scale_alpha(if is_dark { 0.14 } else { 0.1 })
    };
    let button_surface = if is_dark {
        mix_color(surface, Color::WHITE, 0.045)
    } else {
        mix_color(surface, Color::BLACK, 0.025)
    };
    let fit_color = if selected {
        accent.scale_alpha(if is_dark { 0.92 } else { 0.82 })
    } else {
        text_color.scale_alpha(if is_dark { 0.92 } else { 0.72 })
    };
    let button_path = Path::rounded_rectangle(
        Point::new(layout.btn_rect.x, layout.btn_rect.y),
        Size::new(layout.btn_rect.width, layout.btn_rect.height),
        8.0.into(),
    );
    frame.fill(&button_path, button_surface);
    frame.stroke(&button_path, Stroke::default().with_color(border).with_width(0.8));

    let fit_rect = Rectangle::new(
        Point::new(layout.btn_rect.x + 7.0, layout.btn_rect.y + 7.0),
        Size::new(layout.btn_rect.width - 14.0, layout.btn_rect.height - 14.0),
    );
    let fit_path = Path::rounded_rectangle(
        Point::new(fit_rect.x, fit_rect.y),
        Size::new(fit_rect.width, fit_rect.height),
        2.8.into(),
    );
    frame.stroke(&fit_path, Stroke::default().with_color(fit_color).with_width(0.9));
    frame.fill_rectangle(
        Point::new(layout.btn_rect.x + 5.4, layout.btn_rect.y + 5.4),
        Size::new(4.0, 1.0),
        fit_color,
    );
    frame.fill_rectangle(
        Point::new(layout.btn_rect.x + 5.4, layout.btn_rect.y + 5.4),
        Size::new(1.0, 4.0),
        fit_color,
    );
    frame.fill_rectangle(
        Point::new(layout.btn_rect.x + layout.btn_rect.width - 9.4, layout.btn_rect.y + 5.4),
        Size::new(4.0, 1.0),
        fit_color,
    );
    frame.fill_rectangle(
        Point::new(layout.btn_rect.x + layout.btn_rect.width - 6.4, layout.btn_rect.y + 5.4),
        Size::new(1.0, 4.0),
        fit_color,
    );
    frame.fill_rectangle(
        Point::new(layout.btn_rect.x + 5.4, layout.btn_rect.y + layout.btn_rect.height - 6.4),
        Size::new(4.0, 1.0),
        fit_color,
    );
    frame.fill_rectangle(
        Point::new(layout.btn_rect.x + 5.4, layout.btn_rect.y + layout.btn_rect.height - 9.4),
        Size::new(1.0, 4.0),
        fit_color,
    );
    frame.fill_rectangle(
        Point::new(
            layout.btn_rect.x + layout.btn_rect.width - 9.4,
            layout.btn_rect.y + layout.btn_rect.height - 6.4,
        ),
        Size::new(4.0, 1.0),
        fit_color,
    );
    frame.fill_rectangle(
        Point::new(
            layout.btn_rect.x + layout.btn_rect.width - 6.4,
            layout.btn_rect.y + layout.btn_rect.height - 9.4,
        ),
        Size::new(1.0, 4.0),
        fit_color,
    );

    let title_path = Path::rounded_rectangle(
        Point::new(layout.title_rect.x, layout.title_rect.y),
        Size::new(layout.title_rect.width, layout.title_rect.height),
        8.0.into(),
    );
    frame.fill(&title_path, surface);
    frame.stroke(&title_path, Stroke::default().with_color(border).with_width(0.8));

    let max_text_width = (layout.title_rect.width
        - FRAME_HEADER_HORIZONTAL_PADDING
        - FRAME_HEADER_TEXT_RIGHT_PADDING)
        .max(0.0);
    let display_label = fit_text_with_ellipsis(label, FRAME_HEADER_FONT_SIZE, max_text_width);
    frame.fill_text(Text {
        content: display_label,
        position: Point::new(
            layout.title_rect.x + FRAME_HEADER_HORIZONTAL_PADDING,
            layout.title_rect.y + layout.title_rect.height / 2.0,
        ),
        size: Pixels(FRAME_HEADER_FONT_SIZE),
        color: text_color.scale_alpha(if selected { 0.98 } else { 0.9 }),
        align_x: iced::alignment::Horizontal::Left.into(),
        align_y: iced::alignment::Vertical::Center,
        ..Default::default()
    });
}

fn fit_text_with_ellipsis(label: &str, font_size: f32, max_width: f32) -> String {
    if max_width <= 0.0 {
        return "...".to_string();
    }
    if estimate_text_width(label, font_size) <= max_width {
        return label.to_string();
    }
    let ellipsis = "...";
    let ellipsis_width = estimate_text_width(ellipsis, font_size);
    if ellipsis_width >= max_width {
        return ellipsis.to_string();
    }
    let mut out = String::new();
    for ch in label.chars() {
        let mut next = out.clone();
        next.push(ch);
        if estimate_text_width(&next, font_size) + ellipsis_width > max_width {
            break;
        }
        out = next;
    }
    if out.is_empty() { ellipsis.to_string() } else { format!("{out}{ellipsis}") }
}

fn interaction_for_handle(handle: Handle) -> mouse::Interaction {
    match handle {
        Handle::Top | Handle::Bottom => mouse::Interaction::ResizingVertically,
        Handle::Left | Handle::Right => mouse::Interaction::ResizingHorizontally,
        Handle::TopLeft | Handle::BottomRight => mouse::Interaction::ResizingDiagonallyDown,
        Handle::TopRight | Handle::BottomLeft => mouse::Interaction::ResizingDiagonallyUp,
        Handle::RotateTopLeft
        | Handle::RotateTopRight
        | Handle::RotateBottomLeft
        | Handle::RotateBottomRight => mouse::Interaction::Crosshair,
    }
}

/// 公开的 DesignCanvas 结构体，承载该模块边界内传递的结构化状态。
pub struct DesignCanvas<'a> {
    pub doc: Cow<'a, DesignDoc>,
    pub cache: &'a Cache,
    pub pan: Vector,
    pub zoom: f32,
    pub selected_id: Option<&'a str>,
    pub selected_ids: &'a HashSet<String>,
    pub selected_fill_index: Option<usize>,
    pub editing_id: Option<&'a str>,
    pub active_tool: DesignTool,
    pub brush_color_hex: &'a str,
    pub brush_width_px: f32,
    pub toolbar_icon_family: &'a str,
    pub toolbar_icon_name: &'a str,
    pub mouse_wheel_zoom_enabled: bool,
    pub show_slot_content: bool,
    pub show_slot_overflow: bool,
    pub color_picking: bool,
    pub hover_disabled: bool,
}

impl<'a> DesignCanvas<'a> {
    /// 模块内部可见的 frame_header_hit 函数。
    ///
    /// 参数由调用方提供，返回值表达该步骤的计算结果；遇到不可恢复的外部状态时通过现有返回类型向上层传播错误或空结果。
    pub(super) fn frame_header_hit(&self, cursor_pos: Point) -> Option<FrameHeaderHit<'_>> {
        for root in self.doc.children.iter().rev() {
            if root.kind != "frame" {
                continue;
            }
            if let Some(rect) =
                get_element_screen_bounds(self.doc.as_ref(), &root.id, self.pan, self.zoom)
            {
                let layout = frame_header_layout(rect, frame_header_label(root.name.as_deref()));
                if layout.btn_rect.contains(cursor_pos) {
                    return Some(FrameHeaderHit::Fit {
                        id: root.id.as_str(),
                        rect: layout.btn_rect,
                    });
                }
                if layout.title_rect.contains(cursor_pos) {
                    return Some(FrameHeaderHit::Title {
                        id: root.id.as_str(),
                        rect: layout.title_rect,
                    });
                }
            }
        }
        None
    }
}

impl<'a> canvas::Program<Message> for DesignCanvas<'a> {
    type State = DesignCanvasState;

    fn update(
        &self,
        state: &mut Self::State,
        event: &Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<Action<Message>> {
        match event {
            Event::Keyboard(iced::keyboard::Event::KeyPressed { key, .. }) => {
                self.update_key_pressed(state, key)
            }
            _ => self.update_pointer_event(state, event, bounds, cursor),
        }
    }

    fn mouse_interaction(
        &self,
        state: &Self::State,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        if let Some(cursor_pos) = cursor.position_in(bounds) {
            if state.mesh_drag.is_some() {
                return mouse::Interaction::Grabbing;
            }
            if state.is_panning {
                return mouse::Interaction::Grabbing;
            }
            if self.active_tool == DesignTool::Hand {
                return mouse::Interaction::Grab;
            }
            if matches!(self.active_tool, DesignTool::Pen | DesignTool::Eraser) {
                return mouse::Interaction::Crosshair;
            }
            if let Some((_, handle, _)) = state.resizing {
                return interaction_for_handle(handle);
            }
            if state.rotating.is_some() {
                return mouse::Interaction::Crosshair;
            }
            if state.selection_box_start.is_some() {
                return mouse::Interaction::Crosshair;
            }
            if state.tool_preview_start.is_some() {
                return mouse::Interaction::Crosshair;
            }

            if self.frame_header_hit(cursor_pos).is_some() {
                return mouse::Interaction::Pointer;
            }

            if self.active_tool == DesignTool::Move
                && let Some(sel_id) = self.selected_id
                && let Some(rect) =
                    get_element_screen_bounds(self.doc.as_ref(), sel_id, self.pan, self.zoom)
                && let Some(el) = find_element_by_id(&self.doc.children, sel_id)
            {
                let rotation = el.rotation.unwrap_or(0.0);
                let (check_x, check_y) = if rotation != 0.0 {
                    let cx = rect.x + rect.width / 2.0;
                    let cy = rect.y + rect.height / 2.0;
                    rotate_point(cursor_pos.x, cursor_pos.y, cx, cy, -rotation.to_radians())
                } else {
                    (cursor_pos.x, cursor_pos.y)
                };

                let fills = mesh::parse_fill_items(&el.fill);
                if let Some(fill_index) =
                    mesh::choose_mesh_fill_index(&fills, self.selected_fill_index)
                    && let Some(FillItem::Object(FillObject::Mesh(m))) = fills.get(fill_index)
                {
                    let mut mesh = m.clone();
                    mesh.normalize();
                    if mesh::hit_test_mesh(&mesh, rect, check_x, check_y).is_some() {
                        return mouse::Interaction::Pointer;
                    }
                }
            }

            if let Some(sel_id) = self.selected_id
                && let Some(rect) =
                    get_element_screen_bounds(self.doc.as_ref(), sel_id, self.pan, self.zoom)
            {
                let mut rotation = 0.0;
                if let Some(el) = find_element_by_id(&self.doc.children, sel_id) {
                    rotation = el.rotation.unwrap_or(0.0);
                }

                let (check_x, check_y) = if rotation != 0.0 {
                    let cx = rect.x + rect.width / 2.0;
                    let cy = rect.y + rect.height / 2.0;
                    rotate_point(cursor_pos.x, cursor_pos.y, cx, cy, -rotation.to_radians())
                } else {
                    (cursor_pos.x, cursor_pos.y)
                };

                if let Some(handle) = hit_test_handle(
                    rect.x,
                    rect.y,
                    rect.width,
                    rect.height,
                    check_x,
                    check_y,
                    self.zoom,
                ) {
                    return interaction_for_handle(handle);
                }
            }
        }
        mouse::Interaction::Idle
    }

    fn draw(
        &self,
        state: &Self::State,
        renderer: &Renderer,
        theme: &Theme,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let geom = self.cache.draw(renderer, bounds.size(), |frame| {
            let palette = theme.palette();
            frame.fill_rectangle(Point::ORIGIN, bounds.size(), palette.background);

            draw_grid(frame, bounds, self.pan, self.zoom, palette.text);

            let root_origin = Point::new(self.pan.x, self.pan.y);

            for child in &self.doc.children {
                draw_shapes_tree(
                    frame,
                    child,
                    root_origin,
                    self.zoom,
                    self.doc.as_ref(),
                    None,
                    None,
                    None,
                    true,
                    None,
                    self.show_slot_content,
                    self.show_slot_overflow,
                );
            }
            for child in &self.doc.children {
                draw_texts_tree(
                    frame,
                    child,
                    root_origin,
                    self.zoom,
                    self.doc.as_ref(),
                    None,
                    None,
                    None,
                    self.editing_id,
                    None,
                    self.show_slot_content,
                    self.show_slot_overflow,
                );
            }
        });

        state.overlay_cache.clear();
        let palette = theme.palette();
        let overlay_geom = state.overlay_cache.draw(renderer, bounds.size(), |frame| {
            if let Some(start) = state.selection_box_start
                && let Some(end) = cursor.position_in(bounds)
            {
                draw_selection_box(frame, start, end);
            }

            let cursor_pos = cursor.position_in(bounds);
            for root in &self.doc.children {
                if root.kind != "frame" {
                    continue;
                }
                if let Some(rect) =
                    get_element_screen_bounds(self.doc.as_ref(), &root.id, self.pan, self.zoom)
                {
                    let label = frame_header_label(root.name.as_deref());
                    let layout = frame_header_layout(rect, label);
                    draw_frame_header(
                        frame,
                        layout,
                        label,
                        palette.background,
                        palette.text,
                        palette.primary,
                        self.selected_ids.contains(&root.id),
                    );
                }
            }

            if let Some(cursor_pos) = cursor_pos {
                if let Some(hit) = self.frame_header_hit(cursor_pos) {
                    match hit {
                        FrameHeaderHit::Fit { rect, .. } => {
                            tooltip::draw_tooltip(
                                frame,
                                rect,
                                bounds,
                                frame_header_tooltip_label(self.active_tool, "点击适配画板"),
                            );
                        }
                        FrameHeaderHit::Title { rect, .. } => {
                            tooltip::draw_tooltip(
                                frame,
                                rect,
                                bounds,
                                frame_header_tooltip_label(self.active_tool, "点击选中页面"),
                            );
                        }
                    }
                }

                if let Some((_, _, has_moved)) = &state.moving_elements
                    && *has_moved
                    && let Some(target_id) = &state.drop_target_frame_id
                    && let Some(rect) =
                        get_element_screen_bounds(self.doc.as_ref(), target_id, self.pan, self.zoom)
                {
                    let path = Path::rounded_rectangle(
                        Point::new(rect.x, rect.y),
                        Size::new(rect.width, rect.height),
                        8.0.into(),
                    );
                    frame.stroke(
                        &path,
                        Stroke {
                            style: palette.primary.scale_alpha(0.8).into(),
                            width: 2.0,
                            ..Stroke::default()
                        },
                    );
                }

                let preview_parent_rect = state.tool_preview_parent_id.as_deref().and_then(|id| {
                    get_element_screen_bounds(self.doc.as_ref(), id, self.pan, self.zoom)
                });
                draw_tool_preview_overlay(
                    frame,
                    self.active_tool,
                    state.tool_preview_start,
                    state.tool_preview_current,
                    preview_parent_rect,
                );
                draw_brush_preview_overlay(
                    frame,
                    &state.brush_points_world,
                    self.pan,
                    self.zoom,
                    self.brush_color_hex,
                    self.brush_width_px,
                );
                if self.active_tool == DesignTool::Eraser {
                    draw_eraser_overlay(frame, cursor_pos, 30.0);
                }

                if state.moving_elements.is_none()
                    && state.resizing.is_none()
                    && state.rotating.is_none()
                    && state.mesh_drag.is_none()
                    && state.selection_box_start.is_none()
                    && !state.is_panning
                {
                    draw_hover_edit_overlay(
                        frame,
                        state.hovered_id.as_deref(),
                        self.doc.as_ref(),
                        self.pan,
                        self.zoom,
                        palette.primary.scale_alpha(0.85),
                        Color::from_rgb(0.0, 0.75, 1.0).scale_alpha(0.9),
                    );
                }
            }

            let has_selection = !self.selected_ids.is_empty();
            let show_selection = if self.active_tool == DesignTool::Move {
                has_selection
                    || state.resizing.is_some()
                    || state.rotating.is_some()
                    || state.mesh_drag.is_some()
                    || state.moving_elements.is_some()
                    || state.hovered_id.as_deref().is_some_and(|id| self.selected_ids.contains(id))
            } else {
                false
            };
            let selection_color = if show_selection {
                palette.primary.scale_alpha(0.85)
            } else {
                palette.primary.scale_alpha(0.45)
            };
            draw_selection_overlay(
                frame,
                self.selected_ids,
                self.doc.as_ref(),
                self.pan,
                self.zoom,
                self.selected_id,
                self.selected_fill_index,
                selection_color,
                show_selection,
                cursor_pos,
                state.hovered_tailwind_selection.as_ref(),
            );
        });

        vec![geom, overlay_geom]
    }
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
