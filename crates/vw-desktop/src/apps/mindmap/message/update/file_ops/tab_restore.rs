use crate::app::App;
use crate::app::components::mind_map;
use crate::apps::mindmap::canvas::theme::default_custom_themes;
use crate::apps::mindmap::model;
use crate::apps::mindmap::state::{MindMapDoodleStroke, MindMapTab};
use iced::{Point, Vector};
use std::collections::HashSet;

use super::super::super::tabs::ensure_top_tab;
use super::super::persist::persist;
use super::json_format::{MINDMAP_JSON_FORMAT, MindMapJsonFile, MindMapJsonV1};

fn next_tab_id(app: &App) -> String {
    let base = "mindmap";
    let mut n = app.mindmap_tabs.len() + 1;
    loop {
        let id = format!("{base}-{n}");
        if !app.mindmap_tabs.iter().any(|tab| tab.id == id) {
            return id;
        }
        n += 1;
    }
}

fn title_from_path_or_default(
    app: &App,
    file_path: Option<&str>,
    fallback: Option<String>,
) -> String {
    file_path
        .and_then(|path| std::path::Path::new(path).file_name().and_then(|name| name.to_str()))
        .map(str::to_string)
        .or(fallback)
        .unwrap_or_else(|| format!("思维导图 {}", app.mindmap_tabs.len() + 1))
}

fn register_tab(app: &mut App, tab: MindMapTab, id: String) {
    app.mindmap_tabs.push(tab);
    app.mindmap_active_tab_id = Some(id);
    let _ = persist(app);

    let top: Option<(String, String)> = app
        .mindmap_active_tab_id
        .as_ref()
        .cloned()
        .and_then(|active| app.mindmap_tabs.iter().find(|tab| tab.id == active))
        .map(|tab| (tab.id.clone(), tab.title.clone()));
    if let Some((id, title)) = top {
        ensure_top_tab(app, &id, &title);
    }
}

/// 从 Markdown 内容创建新标签页
///
/// 解析 Markdown 文本并创建新的思维导图标签页。
/// 如果内容为空，则使用默认文档结构。
///
/// # 参数
///
/// - `app`: 应用状态的可变引用
/// - `file_path`: 可选的文件路径（用于确定标题）
/// - `md`: Markdown 格式的思维导图内容
pub(super) fn new_tab_from_md(app: &mut App, file_path: Option<String>, md: String) {
    let doc = if md.trim().is_empty() { model::default_doc() } else { mind_map::parse(&md) };
    let id = next_tab_id(app);
    let title = title_from_path_or_default(app, file_path.as_deref(), None);
    let tab = MindMapTab::new(id.clone(), title, file_path, doc);
    register_tab(app, tab, id);
}

/// 从 JSON 内容创建新标签页
///
/// 解析 JSON 文本并创建包含完整状态的思维导图标签页。
///
/// # 参数
///
/// - `app`: 应用状态的可变引用
/// - `file_path`: 可选的文件路径（用于确定标题）
/// - `json_text`: JSON 格式的思维导图数据
///
/// # 返回
///
/// - `Ok(())`: 成功创建标签页
/// - `Err(String)`: 解析或验证失败时的错误信息
pub(super) fn new_tab_from_json(
    app: &mut App,
    file_path: Option<String>,
    json_text: String,
) -> Result<(), String> {
    let MindMapJsonFile { format, version, data } =
        serde_json::from_str(&json_text).map_err(|e| format!("解析 JSON 失败: {e}"))?;

    if format != MINDMAP_JSON_FORMAT || version != 1 {
        return Err("不支持的思维导图 JSON 格式".to_string());
    }

    let MindMapJsonV1 {
        title,
        markdown,
        diagram_type,
        layout_format,
        org_chart_layout_format,
        fishbone_layout_format,
        timeline_layout_format,
        bracket_layout_format,
        tree_layout_format,
        pan_x,
        pan_y,
        zoom,
        selected_path,
        node_positions,
        node_fills,
        node_text_colors,
        node_border_colors,
        node_border_style,
        node_border_styles,
        node_priorities,
        node_urls,
        collapsed_paths,
        background,
        follow_theme_background,
        edge_style,
        edge_styles,
        edge_colors,
        doodle_rgba,
        doodle_width_px,
        doodles,
        theme_group,
        theme_variant,
        custom_themes,
    } = data;

    let doc =
        if markdown.trim().is_empty() { model::default_doc() } else { mind_map::parse(&markdown) };
    let id = next_tab_id(app);
    let title = title_from_path_or_default(app, file_path.as_deref(), title);

    let mut tab = MindMapTab::new(id.clone(), title, file_path, doc);
    tab.diagram_type = diagram_type;
    tab.layout_format = layout_format;
    tab.org_chart_layout_format = org_chart_layout_format;
    tab.fishbone_layout_format = fishbone_layout_format;
    tab.timeline_layout_format = timeline_layout_format;
    tab.bracket_layout_format = bracket_layout_format;
    tab.tree_layout_format = tree_layout_format;
    tab.pan = Vector::new(pan_x, pan_y);
    tab.zoom = zoom.clamp(0.1, 10.0);
    tab.selected_path = selected_path;

    tab.node_positions =
        node_positions.into_iter().map(|p| (p.path, Point::new(p.x, p.y))).collect();

    tab.node_fills = node_fills.into_iter().map(|f| (f.path, f.rgba)).collect();
    tab.node_text_colors = node_text_colors.into_iter().map(|f| (f.path, f.rgba)).collect();
    tab.node_border_colors = node_border_colors.into_iter().map(|f| (f.path, f.rgba)).collect();
    tab.node_border_style = node_border_style;
    tab.node_border_styles = node_border_styles.into_iter().map(|e| (e.path, e.style)).collect();

    tab.node_priorities = node_priorities
        .into_iter()
        .filter_map(|p| (1..=9).contains(&p.priority).then_some((p.path, p.priority)))
        .collect();

    tab.node_urls = node_urls
        .into_iter()
        .filter_map(|u| {
            let url = u.url.trim().trim_matches('`').trim().to_string();
            (!url.is_empty()).then_some((u.path, url))
        })
        .collect();

    tab.collapsed_paths = collapsed_paths.into_iter().collect::<HashSet<_>>();
    tab.background = background;
    tab.follow_theme_background = follow_theme_background;
    tab.edge_style = edge_style;
    tab.edge_styles = edge_styles.into_iter().map(|e| (e.path, e.style)).collect();
    tab.edge_colors = edge_colors.into_iter().map(|e| (e.path, e.rgba)).collect();

    tab.doodle_rgba = if doodle_rgba == 0 { 0x111827FF } else { doodle_rgba };
    tab.doodle_width_px = if doodle_width_px <= 0.0 { 3.0 } else { doodle_width_px };
    tab.doodles = doodles
        .into_iter()
        .filter_map(|stroke| {
            let points = stroke
                .points
                .into_iter()
                .map(|point| Point::new(point.x, point.y))
                .collect::<Vec<_>>();
            (points.len() >= 2).then_some(MindMapDoodleStroke {
                points_world: points,
                rgba: stroke.rgba,
                width_px: stroke.width_px,
            })
        })
        .collect();

    tab.theme_group = theme_group;
    tab.theme_variant = theme_variant;
    tab.custom_themes =
        if custom_themes.is_empty() { default_custom_themes() } else { custom_themes };

    register_tab(app, tab, id);
    Ok(())
}
