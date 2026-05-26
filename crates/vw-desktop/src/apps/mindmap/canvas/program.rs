//! 思维导图画布程序入口，连接 Iced Canvas 的绘制、命中测试和交互更新。

mod draw;
mod hit_test;
mod interaction;
mod ui;
mod update;

#[cfg(test)]
mod interaction_tests;
#[cfg(test)]
mod ui_tests;
#[cfg(test)]
mod update_tests;

use crate::app::Message;
use crate::app::components::mind_map::MindNode;
use crate::apps::mindmap::canvas::theme::MindMapCustomTheme;
use crate::apps::mindmap::state::{
    BracketLayoutFormat, EdgeStyle, FishboneLayoutFormat, MindMapCanvasTool, MindMapDiagramType,
    MindMapDoodleStroke, MindMapLayoutFormat, OrgChartLayoutFormat, TimelineLayoutFormat,
    TreeLayoutFormat,
};
use iced::widget::canvas::{self, Action, Cache, Event, Geometry};
use iced::{Point, Rectangle, Renderer, Theme, Vector, mouse};
use std::collections::{HashMap, HashSet};
use web_time::Instant;

const ERASER_RADIUS_PX: f32 = 30.0;
const TOOLBAR_W: f32 = 168.0;
const TOOLBAR_H: f32 = 40.0;
const TOOLBAR_MARGIN: f32 = 14.0;
const PEN_PANEL_W: f32 = 460.0;
const PEN_PANEL_H: f32 = 40.0;
const PEN_PANEL_GAP: f32 = 8.0;

/// MindMapCanvasState 数据结构，承载当前模块对外传递的显式状态。
#[derive(Debug, Default)]
pub struct MindMapCanvasState {
    drag_mode: DragMode,
    last_cursor: Option<Point>,
    overlay_cache: Cache,
    hovered_node: Option<Vec<usize>>,
    doodle_points_world: Vec<Point>,
    last_click_at: Option<Instant>,
    last_click_node: Option<Vec<usize>>,
    last_click_pos: Option<Point>,
}

#[derive(Debug, Clone)]
#[derive(Default)]
enum DragMode {
    #[default]
    None,
    Pan,
    Node(Vec<usize>),
    DoodlePen,
    DoodleErase,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HoverButtonKind {
    ToggleCollapse,
    AddChild,
    AddSibling,
}


/// MindMapCanvas 数据结构，承载当前模块对外传递的显式状态。
pub struct MindMapCanvas<'a> {
    /// doc 字段，保存渲染或状态更新所需的输入数据。
    pub doc: &'a MindNode,
    /// cache 字段，保存渲染或状态更新所需的输入数据。
    pub cache: &'a Cache,
    /// pan 字段，保存渲染或状态更新所需的输入数据。
    pub pan: Vector,
    /// zoom 字段，保存渲染或状态更新所需的输入数据。
    pub zoom: f32,
    /// selected path 字段，保存渲染或状态更新所需的输入数据。
    pub selected_path: Option<&'a [usize]>,
    /// node positions 字段，保存渲染或状态更新所需的输入数据。
    pub node_positions: &'a HashMap<Vec<usize>, Point>,
    /// diagram type 字段，保存渲染或状态更新所需的输入数据。
    pub diagram_type: MindMapDiagramType,
    /// layout format 字段，保存渲染或状态更新所需的输入数据。
    pub layout_format: MindMapLayoutFormat,
    /// org chart layout format 字段，保存渲染或状态更新所需的输入数据。
    pub org_chart_layout_format: OrgChartLayoutFormat,
    /// fishbone layout format 字段，保存渲染或状态更新所需的输入数据。
    pub fishbone_layout_format: FishboneLayoutFormat,
    /// timeline layout format 字段，保存渲染或状态更新所需的输入数据。
    pub timeline_layout_format: TimelineLayoutFormat,
    /// bracket layout format 字段，保存渲染或状态更新所需的输入数据。
    pub bracket_layout_format: BracketLayoutFormat,
    /// tree layout format 字段，保存渲染或状态更新所需的输入数据。
    pub tree_layout_format: TreeLayoutFormat,
    /// node fills 字段，保存渲染或状态更新所需的输入数据。
    pub node_fills: &'a HashMap<Vec<usize>, u32>,
    /// node text colors 字段，保存渲染或状态更新所需的输入数据。
    pub node_text_colors: &'a HashMap<Vec<usize>, u32>,
    /// node border colors 字段，保存渲染或状态更新所需的输入数据。
    pub node_border_colors: &'a HashMap<Vec<usize>, u32>,
    /// node border style 字段，保存渲染或状态更新所需的输入数据。
    pub node_border_style: EdgeStyle,
    /// node border styles 字段，保存渲染或状态更新所需的输入数据。
    pub node_border_styles: &'a HashMap<Vec<usize>, EdgeStyle>,
    /// node priorities 字段，保存渲染或状态更新所需的输入数据。
    pub node_priorities: &'a HashMap<Vec<usize>, u8>,
    /// node urls 字段，保存渲染或状态更新所需的输入数据。
    pub node_urls: &'a HashMap<Vec<usize>, String>,
    /// collapsed paths 字段，保存渲染或状态更新所需的输入数据。
    pub collapsed_paths: &'a HashSet<Vec<usize>>,
    /// background 字段，保存渲染或状态更新所需的输入数据。
    pub background: Option<u32>,
    /// follow theme background 字段，保存渲染或状态更新所需的输入数据。
    pub follow_theme_background: bool,
    /// edge style 字段，保存渲染或状态更新所需的输入数据。
    pub edge_style: EdgeStyle,
    /// edge styles 字段，保存渲染或状态更新所需的输入数据。
    pub edge_styles: &'a HashMap<Vec<usize>, EdgeStyle>,
    /// edge colors 字段，保存渲染或状态更新所需的输入数据。
    pub edge_colors: &'a HashMap<Vec<usize>, u32>,
    /// canvas tool 字段，保存渲染或状态更新所需的输入数据。
    pub canvas_tool: MindMapCanvasTool,
    /// doodle rgba 字段，保存渲染或状态更新所需的输入数据。
    pub doodle_rgba: u32,
    /// doodle width px 字段，保存渲染或状态更新所需的输入数据。
    pub doodle_width_px: f32,
    /// doodles 字段，保存渲染或状态更新所需的输入数据。
    pub doodles: &'a [MindMapDoodleStroke],
    /// ui blocked rects 字段，保存渲染或状态更新所需的输入数据。
    pub ui_blocked_rects: Vec<Rectangle>,
    /// theme group 字段，保存渲染或状态更新所需的输入数据。
    pub theme_group: &'a str,
    /// theme variant 字段，保存渲染或状态更新所需的输入数据。
    pub theme_variant: usize,
    /// custom themes 字段，保存渲染或状态更新所需的输入数据。
    pub custom_themes: &'a [MindMapCustomTheme],
    /// theme panel open 字段，保存渲染或状态更新所需的输入数据。
    pub theme_panel_open: bool,
}

impl<'a> canvas::Program<Message> for MindMapCanvas<'a> {
    type State = MindMapCanvasState;

    fn update(
        &self,
        state: &mut Self::State,
        event: &Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<Action<Message>> {
        update::update(self, state, event, bounds, cursor)
    }

    fn mouse_interaction(
        &self,
        state: &Self::State,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        interaction::mouse_interaction(self, state, bounds, cursor)
    }

    fn draw(
        &self,
        state: &Self::State,
        renderer: &Renderer,
        theme: &Theme,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        draw::draw(self, state, renderer, theme, bounds, cursor)
    }
}

#[cfg(test)]
#[path = "program_tests.rs"]
mod program_tests;
