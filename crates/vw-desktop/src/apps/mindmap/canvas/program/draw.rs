//! 思维导图画布绘制模块。

#[path = "doodles.rs"]
mod doodles;
#[path = "edges.rs"]
mod edges;
#[path = "nodes.rs"]
mod nodes;
#[path = "overlay.rs"]
mod overlay;

#[cfg(test)]
#[path = "doodles_tests.rs"]
mod doodles_tests;
#[cfg(test)]
#[path = "draw_tests.rs"]
mod draw_tests;
#[cfg(test)]
#[path = "edges_tests.rs"]
mod edges_tests;
#[cfg(test)]
#[path = "nodes_tests.rs"]
mod nodes_tests;
#[cfg(test)]
#[path = "overlay_tests.rs"]
mod overlay_tests;

use crate::apps::mindmap::canvas::layout::{Layout, compute_layout_for_diagram};
use crate::apps::mindmap::canvas::style::{
    DEFAULT_STROKE_RGBA, node_border_width_px, rgba_u32_to_color,
};
use crate::apps::mindmap::canvas::theme::{MindMapThemeView, resolve_theme};
use iced::widget::canvas::{Frame, Geometry};
use iced::{Point, Rectangle, Renderer, Size, Theme, mouse};

use super::{MindMapCanvas, MindMapCanvasState};

type FishboneMeta = (Point, Size, Rectangle, f32);

pub(super) fn draw(
    canvas: &MindMapCanvas<'_>,
    state: &MindMapCanvasState,
    renderer: &Renderer,
    theme: &Theme,
    bounds: Rectangle,
    cursor: mouse::Cursor,
) -> Vec<Geometry> {
    let geom = canvas.cache.draw(renderer, bounds.size(), |frame: &mut Frame| {
        let layout = layout_for_canvas(canvas);
        let current_theme =
            resolve_theme(canvas.theme_group, canvas.theme_variant, canvas.custom_themes);
        let palette = theme.extended_palette();
        let background = if let Some(rgba) = canvas.background {
            rgba_u32_to_color(rgba)
        } else if canvas.follow_theme_background {
            rgba_u32_to_color(current_theme.background_color)
        } else {
            palette.background.base.color
        };
        let stroke_color = rgba_u32_to_color(DEFAULT_STROKE_RGBA);
        let stroke_width = node_border_width_px(canvas.zoom);

        frame.fill_rectangle(Point::ORIGIN, bounds.size(), background);
        edges::draw_special_diagram_backdrop(
            frame,
            canvas,
            &layout,
            &current_theme,
            stroke_color,
            stroke_width,
        );

        let fishbone_meta = edges::fishbone_meta(canvas, &layout);
        edges::draw_edges(
            frame,
            canvas,
            &layout,
            &current_theme,
            stroke_color,
            stroke_width,
            fishbone_meta.as_ref(),
        );
        nodes::draw_nodes(
            frame,
            canvas,
            theme,
            &layout,
            &current_theme,
            stroke_color,
            stroke_width,
        );
        doodles::draw_committed_doodles(frame, canvas);
    });

    state.overlay_cache.clear();
    let overlay_geom = state.overlay_cache.draw(renderer, bounds.size(), |frame: &mut Frame| {
        overlay::draw_overlay(frame, canvas, state, bounds, cursor);
    });

    vec![geom, overlay_geom]
}

pub(super) fn layout_for_canvas(canvas: &MindMapCanvas<'_>) -> Layout {
    compute_layout_for_diagram(
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
    )
}

pub(super) type ThemeView<'a> = MindMapThemeView<'a>;
