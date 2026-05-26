//! 思维导图持久化消息处理，负责文件读写、导入导出和状态落盘入口。

use crate::app::components::mind_map;
use crate::app::{App, Message};
use crate::apps::mindmap::canvas::theme::{MindMapCustomTheme, default_custom_themes};
use crate::apps::mindmap::model;
use crate::apps::mindmap::state::{
    BracketLayoutFormat, EdgeStyle, FishboneLayoutFormat, MindMapDiagramType, MindMapLayoutFormat,
    MindMapTab, OrgChartLayoutFormat, TreeLayoutFormat,
};
use iced::{Point, Task, Vector};
use std::collections::HashSet;

#[cfg(target_arch = "wasm32")]
use super::MindMapMessage;
use super::tabs::sync_top_tabs;

pub(super) fn default_edge_style() -> EdgeStyle {
    EdgeStyle::Solid
}

pub(super) fn default_org_chart_layout_format() -> OrgChartLayoutFormat {
    OrgChartLayoutFormat::default()
}

pub(super) fn default_fishbone_layout_format() -> FishboneLayoutFormat {
    FishboneLayoutFormat::default()
}

pub(super) fn default_bracket_layout_format() -> BracketLayoutFormat {
    BracketLayoutFormat::default()
}

pub(super) fn default_tree_layout_format() -> TreeLayoutFormat {
    TreeLayoutFormat::default()
}

/// 构建或更新 persist 相关行为。
///
/// 参数由调用方提供，返回值直接交给上层视图或状态处理；本函数不执行额外错误恢复。
pub(super) fn persist(app: &App) -> Task<Message> {
    let active_id = app.mindmap_active_tab_id.clone();
    let tabs = app
        .mindmap_tabs
        .iter()
        .map(|t| PersistedTab {
            id: t.id.clone(),
            title: t.title.clone(),
            file_path: t.file_path.clone(),
            markdown: mind_map::to_markdown(&t.doc),
            diagram_type: t.diagram_type,
            layout_format: t.layout_format,
            org_chart_layout_format: t.org_chart_layout_format,
            fishbone_layout_format: t.fishbone_layout_format,
            bracket_layout_format: t.bracket_layout_format,
            tree_layout_format: t.tree_layout_format,
            pan_x: t.pan.x,
            pan_y: t.pan.y,
            zoom: t.zoom,
            selected_path: t.selected_path.clone(),
            node_positions: t
                .node_positions
                .iter()
                .map(|(path, pt)| PersistedNodePos {
                    path: path.clone(),
                    x: pt.x,
                    y: pt.y,
                })
                .collect(),
            node_fills: t
                .node_fills
                .iter()
                .map(|(path, rgba)| PersistedNodeFill {
                    path: path.clone(),
                    rgba: *rgba,
                })
                .collect(),
            node_text_colors: t
                .node_text_colors
                .iter()
                .map(|(path, rgba)| PersistedNodeTextColor {
                    path: path.clone(),
                    rgba: *rgba,
                })
                .collect(),
            node_border_colors: t
                .node_border_colors
                .iter()
                .map(|(path, rgba)| PersistedNodeBorderColor {
                    path: path.clone(),
                    rgba: *rgba,
                })
                .collect(),
            node_border_style: t.node_border_style,
            node_border_styles: t
                .node_border_styles
                .iter()
                .map(|(path, style)| {
                    PersistedNodeBorderStyle {
                        path: path.clone(),
                        style: *style,
                    }
                })
                .collect(),
            node_priorities: t
                .node_priorities
                .iter()
                .map(|(path, priority)| PersistedNodePriority {
                    path: path.clone(),
                    priority: *priority,
                })
                .collect(),
            node_urls: t
                .node_urls
                .iter()
                .map(|(path, url)| PersistedNodeUrl {
                    path: path.clone(),
                    url: url.clone(),
                })
                .collect(),
            collapsed_paths: t.collapsed_paths.iter().cloned().collect(),
            background: t.background,
            follow_theme_background: t.follow_theme_background,
            edge_style: t.edge_style,
            edge_styles: t
                .edge_styles
                .iter()
                .map(|(path, style)| {
                    PersistedEdgeStyle {
                        path: path.clone(),
                        style: *style,
                    }
                })
                .collect(),
            edge_colors: t
                .edge_colors
                .iter()
                .map(|(path, rgba)| PersistedEdgeColor {
                    path: path.clone(),
                    rgba: *rgba,
                })
                .collect(),
            doodle_rgba: t.doodle_rgba,
            doodle_width_px: t.doodle_width_px,
            doodles: t
                .doodles
                .iter()
                .map(|s| PersistedDoodleStroke {
                    rgba: s.rgba,
                    width_px: s.width_px,
                    points: s
                        .points_world
                        .iter()
                        .map(|p| PersistedPoint { x: p.x, y: p.y })
                        .collect(),
                })
                .collect(),
            theme_group: t.theme_group.clone(),
            theme_variant: t.theme_variant,
            custom_themes: t.custom_themes.clone(),
        })
        .collect::<Vec<_>>();

    let state = PersistedState { active_id, tabs };
    if let Ok(v) = serde_json::to_value(state) {
        #[cfg(target_arch = "wasm32")]
        {
            return Task::perform(crate::app::config::save_mindmap_tabs_owned(v), |_| {
                Message::None
            });
        }

        #[cfg(not(target_arch = "wasm32"))]
        crate::app::save_mindmap_tabs(&v);
    }

    Task::none()
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct PersistedState {
    active_id: Option<String>,
    tabs: Vec<PersistedTab>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct PersistedTab {
    id: String,
    title: String,
    file_path: Option<String>,
    markdown: String,
    #[serde(default)]
    diagram_type: MindMapDiagramType,
    #[serde(default)]
    layout_format: MindMapLayoutFormat,
    #[serde(default = "default_org_chart_layout_format")]
    org_chart_layout_format: OrgChartLayoutFormat,
    #[serde(default = "default_fishbone_layout_format")]
    fishbone_layout_format: FishboneLayoutFormat,
    #[serde(default = "default_bracket_layout_format")]
    bracket_layout_format: BracketLayoutFormat,
    #[serde(default = "default_tree_layout_format")]
    tree_layout_format: TreeLayoutFormat,
    pan_x: f32,
    pan_y: f32,
    zoom: f32,
    selected_path: Option<Vec<usize>>,
    node_positions: Vec<PersistedNodePos>,
    node_fills: Vec<PersistedNodeFill>,
    #[serde(default)]
    node_text_colors: Vec<PersistedNodeTextColor>,
    #[serde(default)]
    node_border_colors: Vec<PersistedNodeBorderColor>,
    #[serde(default = "default_edge_style")]
    node_border_style: EdgeStyle,
    #[serde(default)]
    node_border_styles: Vec<PersistedNodeBorderStyle>,
    #[serde(default)]
    node_priorities: Vec<PersistedNodePriority>,
    #[serde(default)]
    node_urls: Vec<PersistedNodeUrl>,
    #[serde(default)]
    collapsed_paths: Vec<Vec<usize>>,
    background: Option<u32>,
    #[serde(default = "default_follow_theme_background")]
    follow_theme_background: bool,
    #[serde(default = "default_edge_style")]
    edge_style: EdgeStyle,
    #[serde(default)]
    edge_styles: Vec<PersistedEdgeStyle>,
    #[serde(default)]
    edge_colors: Vec<PersistedEdgeColor>,
    #[serde(default)]
    doodle_rgba: u32,
    #[serde(default)]
    doodle_width_px: f32,
    #[serde(default)]
    doodles: Vec<PersistedDoodleStroke>,
    #[serde(default = "default_theme_group")]
    theme_group: String,
    #[serde(default)]
    theme_variant: usize,
    #[serde(default)]
    custom_themes: Vec<MindMapCustomTheme>,
}

pub(super) fn default_theme_group() -> String {
    "classic".to_string()
}

pub(super) fn default_follow_theme_background() -> bool {
    true
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct PersistedNodePos {
    path: Vec<usize>,
    x: f32,
    y: f32,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct PersistedNodeFill {
    path: Vec<usize>,
    rgba: u32,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct PersistedNodeTextColor {
    path: Vec<usize>,
    rgba: u32,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct PersistedNodeBorderColor {
    path: Vec<usize>,
    rgba: u32,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct PersistedNodeBorderStyle {
    path: Vec<usize>,
    style: EdgeStyle,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct PersistedNodePriority {
    path: Vec<usize>,
    priority: u8,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct PersistedNodeUrl {
    path: Vec<usize>,
    url: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct PersistedEdgeStyle {
    path: Vec<usize>,
    style: EdgeStyle,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct PersistedEdgeColor {
    path: Vec<usize>,
    rgba: u32,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct PersistedDoodleStroke {
    rgba: u32,
    width_px: f32,
    points: Vec<PersistedPoint>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct PersistedPoint {
    x: f32,
    y: f32,
}

fn apply_persisted_state(app: &mut App, state: PersistedState) {
    let mut tabs = Vec::new();
    for t in state.tabs {
        let doc = if t.markdown.trim().is_empty() {
            model::default_doc()
        } else {
            mind_map::parse(&t.markdown)
        };
        let mut tab = MindMapTab::new(t.id, t.title, t.file_path, doc);
        tab.diagram_type = t.diagram_type;
        tab.layout_format = t.layout_format;
        tab.org_chart_layout_format = t.org_chart_layout_format;
        tab.fishbone_layout_format = t.fishbone_layout_format;
        tab.bracket_layout_format = t.bracket_layout_format;
        tab.tree_layout_format = t.tree_layout_format;
        tab.pan = Vector::new(t.pan_x, t.pan_y);
        tab.zoom = t.zoom.clamp(0.1, 10.0);
        tab.selected_path = t.selected_path;
        tab.node_positions =
            t.node_positions.into_iter().map(|p| (p.path, Point::new(p.x, p.y))).collect();
        tab.node_fills = t.node_fills.into_iter().map(|f| (f.path, f.rgba)).collect();
        tab.node_text_colors = t.node_text_colors.into_iter().map(|f| (f.path, f.rgba)).collect();
        tab.node_border_colors =
            t.node_border_colors.into_iter().map(|f| (f.path, f.rgba)).collect();
        tab.node_border_style = t.node_border_style;
        tab.node_border_styles =
            t.node_border_styles.into_iter().map(|e| (e.path, e.style)).collect();
        tab.node_priorities = t
            .node_priorities
            .into_iter()
            .filter_map(|p| (1..=9).contains(&p.priority).then_some((p.path, p.priority)))
            .collect();
        tab.node_urls = t
            .node_urls
            .into_iter()
            .filter_map(|u| {
                let url = u.url.trim().trim_matches('`').trim().to_string();
                (!url.is_empty()).then_some((u.path, url))
            })
            .collect();
        tab.collapsed_paths = t.collapsed_paths.into_iter().collect::<HashSet<_>>();
        tab.background = t.background;
        tab.follow_theme_background = t.follow_theme_background;
        tab.edge_style = t.edge_style;
        tab.edge_styles = t.edge_styles.into_iter().map(|e| (e.path, e.style)).collect();
        tab.edge_colors = t.edge_colors.into_iter().map(|e| (e.path, e.rgba)).collect();
        tab.doodle_rgba = if t.doodle_rgba == 0 { 0x111827FF } else { t.doodle_rgba };
        tab.doodle_width_px = if t.doodle_width_px <= 0.0 { 3.0 } else { t.doodle_width_px };
        tab.doodles = t
            .doodles
            .into_iter()
            .filter_map(|s| {
                let pts = s.points.into_iter().map(|p| Point::new(p.x, p.y)).collect::<Vec<_>>();
                (pts.len() >= 2).then_some(crate::apps::mindmap::state::MindMapDoodleStroke {
                    points_world: pts,
                    rgba: s.rgba,
                    width_px: s.width_px,
                })
            })
            .collect();
        tab.theme_group = t.theme_group;
        tab.theme_variant = t.theme_variant;
        tab.custom_themes =
            if t.custom_themes.is_empty() { default_custom_themes() } else { t.custom_themes };
        tabs.push(tab);
    }
    app.mindmap_tabs = tabs;
    app.mindmap_active_tab_id = state.active_id;
    sync_top_tabs(app);
}

/// 构建或更新 load persisted 相关行为。
///
/// 参数由调用方提供，返回值直接交给上层视图或状态处理；本函数不执行额外错误恢复。
pub fn load_persisted(app: &mut App) -> Task<Message> {
    #[cfg(target_arch = "wasm32")]
    {
        let _ = app;
        return Task::perform(crate::app::load_mindmap_tabs_async(), |res| {
            Message::MindMapTool(MindMapMessage::LoadPersistedFinished(res))
        });
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let Some(v) = crate::app::load_mindmap_tabs() else {
            return Task::none();
        };
        let Ok(state) = serde_json::from_value::<PersistedState>(v) else {
            return Task::none();
        };
        apply_persisted_state(app, state);
        Task::none()
    }
}

/// 构建或更新 load persisted finished 相关行为。
///
/// 参数由调用方提供，返回值直接交给上层视图或状态处理；本函数不执行额外错误恢复。
pub(super) fn load_persisted_finished(
    app: &mut App,
    result: Result<Option<serde_json::Value>, String>,
) -> Task<Message> {
    let Ok(Some(v)) = result else {
        return Task::none();
    };
    let Ok(state) = serde_json::from_value::<PersistedState>(v) else {
        return Task::none();
    };
    apply_persisted_state(app, state);
    Task::none()
}
