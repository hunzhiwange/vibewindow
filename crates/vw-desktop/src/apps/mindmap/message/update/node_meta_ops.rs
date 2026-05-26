//! 节点元数据操作模块
//!
//! 本模块提供思维导图节点的元数据相关操作功能，包括：
//! - 节点选择与清除选择
//! - 节点优先级管理
//! - 节点 URL 链接管理
//! - 图表类型与布局格式设置
//! - 各类选择器和面板的切换控制
//!
//! 所有操作都会触发相应的 UI 状态更新和持久化。

use crate::app::{App, Message};
use crate::apps::mindmap::state::{
    FishboneLayoutFormat, MindMapDiagramType, MindMapLayoutFormat, MindMapTab,
    OrgChartLayoutFormat, TimelineLayoutFormat, TreeLayoutFormat,
};
use iced::Task;
use iced::widget::text_editor;

use super::super::persist::persist;
use super::node_ops;

/// 如果 URL 编辑器处于打开状态，提交其内容到节点元数据
///
/// 该函数会先调用通用的文本编辑器提交逻辑，然后检查 URL 编辑器是否打开。
/// 如果打开且存在选中的节点路径，则将编辑器中的 URL 值（去除首尾空白和反引号后）
/// 保存到 `node_urls` 映射中。如果 URL 为空，则从映射中移除该条目。
///
/// # 参数
///
/// - `tab` - 可变的思维导图标签页引用
pub(super) fn commit_url_editor_if_needed(tab: &mut MindMapTab) {
    node_ops::commit_text_editor_if_needed(tab);

    // 如果 URL 编辑器未打开，直接返回
    if !tab.show_url_editor {
        return;
    }

    // 获取当前选中的节点路径
    let Some(path) = tab.selected_path.clone() else {
        return;
    };

    // 处理 URL 值：去除首尾空白和反引号
    let url = tab.url_editor_value.trim().trim_matches('`').trim().to_string();

    // 根据处理后的 URL 是否为空，决定插入或移除
    if url.is_empty() {
        tab.node_urls.remove(&path);
    } else {
        tab.node_urls.insert(path, url);
    }
}

/// 选择指定的节点
///
/// 选中给定路径的节点，并重置所有相关的 UI 状态：
/// - 提交未保存的 URL 编辑器内容
/// - 设置选中路径
/// - 关闭所有弹出菜单和选择器
/// - 清空编辑器内容和画布缓存
/// - 触发持久化
///
/// # 参数
///
/// - `app` - 可变的应用状态引用
/// - `path` - 节点路径，由节点在树中的位置索引组成
///
/// # 返回
///
/// 返回空的任务（`Task::none()`），因为此操作不需要异步处理
pub(super) fn select_node(app: &mut App, path: Vec<usize>) -> Task<Message> {
    if let Some(tab) = app.active_mindmap_tab_mut() {
        // 先提交任何未保存的 URL 编辑器内容
        commit_url_editor_if_needed(tab);

        // 设置新的选中路径
        tab.selected_path = Some(path);

        // 关闭所有弹出式 UI 组件
        tab.show_context_menu = false;
        tab.context_menu_anchor = None;
        tab.show_zoom_menu = false;
        tab.show_diagram_type_picker = false;
        tab.show_url_editor = false;
        tab.show_text_editor = false;

        // 清空编辑器内容
        tab.url_editor_value.clear();
        tab.node_text_editor = text_editor::Content::new();

        // 清空画布缓存以触发重绘
        tab.canvas_cache.clear();

        // 持久化状态
        let _ = persist(app);
    }
    Task::none()
}

/// 清除当前节点的选择状态
///
/// 取消当前选中的节点，并重置所有相关的 UI 状态：
/// - 提交未保存的 URL 编辑器内容
/// - 清除选中路径
/// - 关闭所有弹出菜单、选择器和面板
/// - 清空编辑器内容和画布缓存
/// - 触发持久化
///
/// # 参数
///
/// - `app` - 可变的应用状态引用
///
/// # 返回
///
/// 返回空的任务（`Task::none()`），因为此操作不需要异步处理
pub(super) fn clear_selection(app: &mut App) -> Task<Message> {
    if let Some(tab) = app.active_mindmap_tab_mut() {
        // 先提交任何未保存的 URL 编辑器内容
        commit_url_editor_if_needed(tab);

        // 清除选中路径
        tab.selected_path = None;

        // 关闭所有弹出式 UI 组件
        tab.show_context_menu = false;
        tab.context_menu_anchor = None;
        tab.active_color_picker = None;
        tab.show_zoom_menu = false;
        tab.show_priority_picker = false;
        tab.show_url_editor = false;
        tab.show_text_editor = false;
        tab.show_export_menu = false;
        tab.show_diagram_type_picker = false;
        tab.show_theme_panel = false;

        // 清空编辑器内容
        tab.url_editor_value.clear();
        tab.node_text_editor = text_editor::Content::new();

        // 清空画布缓存以触发重绘
        tab.canvas_cache.clear();

        // 持久化状态
        let _ = persist(app);
    }
    Task::none()
}

/// 关闭所有选择器和面板
///
/// 关闭当前打开的所有选择器、面板和菜单，同时：
/// - 提交未保存的 URL 编辑器内容
/// - 清空编辑器内容和画布缓存
/// - 触发持久化
///
/// 这是一个通用的"关闭所有弹出层"操作，通常在用户点击画布空白区域时调用。
///
/// # 参数
///
/// - `app` - 可变的应用状态引用
///
/// # 返回
///
/// 返回空的任务（`Task::none()`），因为此操作不需要异步处理
pub(super) fn close_pickers(app: &mut App) -> Task<Message> {
    #[cfg(debug_assertions)]
    println!("ClosePickers");

    if let Some(tab) = app.active_mindmap_tab_mut() {
        // 先提交任何未保存的 URL 编辑器内容
        commit_url_editor_if_needed(tab);

        // 关闭所有选择器和面板
        tab.active_color_picker = None;
        tab.show_diagram_type_picker = false;
        tab.show_markdown_import = false;
        tab.show_export_menu = false;
        tab.show_zoom_menu = false;
        tab.show_priority_picker = false;
        tab.show_url_editor = false;
        tab.show_text_editor = false;
        tab.show_action_menu = false;
        tab.show_theme_panel = false;

        // 清空编辑器内容
        tab.url_editor_value.clear();
        tab.node_text_editor = text_editor::Content::new();

        // 清空画布缓存以触发重绘
        tab.canvas_cache.clear();

        // 持久化状态
        let _ = persist(app);
    }
    Task::none()
}

/// 切换图表类型选择器的显示状态
///
/// 切换图表类型选择器的显示/隐藏状态。如果选择器被打开：
/// - 提交未保存的 URL 编辑器内容
/// - 关闭所有其他选择器和面板
/// - 清空编辑器内容
///
/// 无论打开还是关闭，都会清空画布缓存以触发重绘，并持久化状态。
///
/// # 参数
///
/// - `app` - 可变的应用状态引用
///
/// # 返回
///
/// 返回空的任务（`Task::none()`），因为此操作不需要异步处理
pub(super) fn toggle_diagram_type_picker(app: &mut App) -> Task<Message> {
    if let Some(tab) = app.active_mindmap_tab_mut() {
        // 切换显示状态
        tab.show_diagram_type_picker = !tab.show_diagram_type_picker;

        // 如果打开选择器，关闭其他所有选择器并清理状态
        if tab.show_diagram_type_picker {
            commit_url_editor_if_needed(tab);
            tab.active_color_picker = None;
            tab.show_markdown_import = false;
            tab.show_export_menu = false;
            tab.show_zoom_menu = false;
            tab.show_priority_picker = false;
            tab.show_url_editor = false;
            tab.show_text_editor = false;
            tab.show_action_menu = false;
            tab.show_theme_panel = false;
            tab.url_editor_value.clear();
            tab.node_text_editor = text_editor::Content::new();
        }

        // 清空画布缓存以触发重绘
        tab.canvas_cache.clear();

        // 持久化状态
        let _ = persist(app);
    }
    Task::none()
}

/// 设置思维导图的图表类型
///
/// 将当前思维导图的图表类型更改为指定类型。如果类型发生变化：
/// - 清空节点位置缓存（因为不同图表类型有不同的布局算法）
///
/// 无论是否变化，都会清空画布缓存并持久化状态。
///
/// # 参数
///
/// - `app` - 可变的应用状态引用
/// - `diagram_type` - 新的图表类型
///
/// # 返回
///
/// 返回空的任务（`Task::none()`），因为此操作不需要异步处理
pub(super) fn set_diagram_type(app: &mut App, diagram_type: MindMapDiagramType) -> Task<Message> {
    if let Some(tab) = app.active_mindmap_tab_mut() {
        // 只有类型确实改变时才清空节点位置
        if tab.diagram_type != diagram_type {
            tab.diagram_type = diagram_type;
            tab.node_positions.clear();
        }

        // 清空画布缓存以触发重绘
        tab.canvas_cache.clear();

        // 持久化状态
        let _ = persist(app);
    }
    Task::none()
}

pub(super) fn select_diagram_type(
    app: &mut App,
    diagram_type: MindMapDiagramType,
) -> Task<Message> {
    if let Some(tab) = app.active_mindmap_tab_mut() {
        tab.show_diagram_type_picker = false;

        if tab.diagram_type != diagram_type {
            tab.diagram_type = diagram_type;
            tab.node_positions.clear();
        }

        tab.canvas_cache.clear();
        let _ = persist(app);
    }
    Task::none()
}

/// 设置思维导图的布局格式
///
/// 更新标准思维导图的布局格式，并清空节点位置和画布缓存以触发重新布局。
///
/// # 参数
///
/// - `app` - 可变的应用状态引用
/// - `layout_format` - 新的布局格式
///
/// # 返回
///
/// 返回空的任务（`Task::none()`），因为此操作不需要异步处理
pub(super) fn set_layout_format(
    app: &mut App,
    layout_format: MindMapLayoutFormat,
) -> Task<Message> {
    if let Some(tab) = app.active_mindmap_tab_mut() {
        tab.layout_format = layout_format;
        tab.show_diagram_type_picker = false;

        // 清空节点位置缓存，触发重新布局计算
        tab.node_positions.clear();

        // 清空画布缓存以触发重绘
        tab.canvas_cache.clear();

        // 持久化状态
        let _ = persist(app);
    }
    Task::none()
}

/// 设置组织结构图的布局格式
///
/// 更新组织结构图的布局格式，并清空节点位置和画布缓存以触发重新布局。
///
/// # 参数
///
/// - `app` - 可变的应用状态引用
/// - `layout_format` - 新的组织结构图布局格式
///
/// # 返回
///
/// 返回空的任务（`Task::none()`），因为此操作不需要异步处理
pub(super) fn set_org_chart_layout_format(
    app: &mut App,
    layout_format: OrgChartLayoutFormat,
) -> Task<Message> {
    if let Some(tab) = app.active_mindmap_tab_mut() {
        tab.org_chart_layout_format = layout_format;
        tab.show_diagram_type_picker = false;

        // 清空节点位置缓存，触发重新布局计算
        tab.node_positions.clear();

        // 清空画布缓存以触发重绘
        tab.canvas_cache.clear();

        // 持久化状态
        let _ = persist(app);
    }
    Task::none()
}

/// 设置鱼骨图的布局格式
///
/// 更新鱼骨图的布局格式，并清空节点位置和画布缓存以触发重新布局。
///
/// # 参数
///
/// - `app` - 可变的应用状态引用
/// - `layout_format` - 新的鱼骨图布局格式
///
/// # 返回
///
/// 返回空的任务（`Task::none()`），因为此操作不需要异步处理
pub(super) fn set_fishbone_layout_format(
    app: &mut App,
    layout_format: FishboneLayoutFormat,
) -> Task<Message> {
    if let Some(tab) = app.active_mindmap_tab_mut() {
        tab.fishbone_layout_format = layout_format;
        tab.show_diagram_type_picker = false;

        // 清空节点位置缓存，触发重新布局计算
        tab.node_positions.clear();

        // 清空画布缓存以触发重绘
        tab.canvas_cache.clear();

        // 持久化状态
        let _ = persist(app);
    }
    Task::none()
}

/// 设置时间线图的布局格式
///
/// 更新时间线图的布局格式，并清空节点位置和画布缓存以触发重新布局。
///
/// # 参数
///
/// - `app` - 可变的应用状态引用
/// - `layout_format` - 新的时间线布局格式
///
/// # 返回
///
/// 返回空的任务（`Task::none()`），因为此操作不需要异步处理
pub(super) fn set_timeline_layout_format(
    app: &mut App,
    layout_format: TimelineLayoutFormat,
) -> Task<Message> {
    if let Some(tab) = app.active_mindmap_tab_mut() {
        tab.timeline_layout_format = layout_format;
        tab.show_diagram_type_picker = false;

        // 清空节点位置缓存，触发重新布局计算
        tab.node_positions.clear();

        // 清空画布缓存以触发重绘
        tab.canvas_cache.clear();

        // 持久化状态
        let _ = persist(app);
    }
    Task::none()
}

/// 设置括号图的布局格式
///
/// 更新括号图的布局格式，并清空节点位置和画布缓存以触发重新布局。
///
/// # 参数
///
/// - `app` - 可变的应用状态引用
/// - `layout_format` - 新的括号图布局格式
///
/// # 返回
///
/// 返回空的任务（`Task::none()`），因为此操作不需要异步处理
pub(super) fn set_bracket_layout_format(
    app: &mut App,
    layout_format: crate::apps::mindmap::state::BracketLayoutFormat,
) -> Task<Message> {
    if let Some(tab) = app.active_mindmap_tab_mut() {
        tab.bracket_layout_format = layout_format;
        tab.show_diagram_type_picker = false;

        // 清空节点位置缓存，触发重新布局计算
        tab.node_positions.clear();

        // 清空画布缓存以触发重绘
        tab.canvas_cache.clear();

        // 持久化状态
        let _ = persist(app);
    }
    Task::none()
}

/// 设置树形图的布局格式
///
/// 更新树形图的布局格式，并清空节点位置和画布缓存以触发重新布局。
///
/// # 参数
///
/// - `app` - 可变的应用状态引用
/// - `layout_format` - 新的树形图布局格式
///
/// # 返回
///
/// 返回空的任务（`Task::none()`），因为此操作不需要异步处理
pub(super) fn set_tree_layout_format(
    app: &mut App,
    layout_format: TreeLayoutFormat,
) -> Task<Message> {
    if let Some(tab) = app.active_mindmap_tab_mut() {
        tab.tree_layout_format = layout_format;
        tab.show_diagram_type_picker = false;

        // 清空节点位置缓存，触发重新布局计算
        tab.node_positions.clear();

        // 清空画布缓存以触发重绘
        tab.canvas_cache.clear();

        // 持久化状态
        let _ = persist(app);
    }
    Task::none()
}

/// 切换导出菜单的显示状态
///
/// 切换导出菜单的显示/隐藏状态。如果菜单被打开：
/// - 提交未保存的 URL 编辑器内容
/// - 关闭所有其他选择器和面板
/// - 清空编辑器内容
///
/// # 参数
///
/// - `app` - 可变的应用状态引用
///
/// # 返回
///
/// 返回空的任务（`Task::none()`），因为此操作不需要异步处理
pub(super) fn toggle_export_menu(app: &mut App) -> Task<Message> {
    if let Some(tab) = app.active_mindmap_tab_mut() {
        // 切换显示状态
        tab.show_export_menu = !tab.show_export_menu;

        // 如果打开菜单，关闭其他所有选择器并清理状态
        if tab.show_export_menu {
            commit_url_editor_if_needed(tab);
            tab.active_color_picker = None;
            tab.show_diagram_type_picker = false;
            tab.show_markdown_import = false;
            tab.show_zoom_menu = false;
            tab.show_priority_picker = false;
            tab.show_url_editor = false;
            tab.show_text_editor = false;
            tab.show_action_menu = false;
            tab.show_theme_panel = false;
            tab.url_editor_value.clear();
            tab.node_text_editor = text_editor::Content::new();
        }
    }
    Task::none()
}

/// 切换主题面板的显示状态
///
/// 切换主题面板的显示/隐藏状态。如果面板被打开：
/// - 提交未保存的 URL 编辑器内容
/// - 关闭所有其他选择器和面板
/// - 清空编辑器内容
///
/// 无论打开还是关闭，都会清空画布缓存并持久化状态。
///
/// # 参数
///
/// - `app` - 可变的应用状态引用
///
/// # 返回
///
/// 返回空的任务（`Task::none()`），因为此操作不需要异步处理
pub(super) fn toggle_theme_panel(app: &mut App) -> Task<Message> {
    if let Some(tab) = app.active_mindmap_tab_mut() {
        // 切换显示状态
        tab.show_theme_panel = !tab.show_theme_panel;

        // 如果打开面板，关闭其他所有选择器并清理状态
        if tab.show_theme_panel {
            commit_url_editor_if_needed(tab);
            tab.active_color_picker = None;
            tab.show_diagram_type_picker = false;
            tab.show_markdown_import = false;
            tab.show_zoom_menu = false;
            tab.show_priority_picker = false;
            tab.show_url_editor = false;
            tab.show_text_editor = false;
            tab.show_action_menu = false;
            tab.url_editor_value.clear();
            tab.node_text_editor = text_editor::Content::new();
        }

        // 清空画布缓存以触发重绘
        tab.canvas_cache.clear();

        // 持久化状态
        let _ = persist(app);
    }
    Task::none()
}

/// 切换操作菜单的显示状态
///
/// 切换操作菜单的显示/隐藏状态。如果菜单被打开：
/// - 提交未保存的 URL 编辑器内容
/// - 关闭所有其他选择器和面板
/// - 清空编辑器内容
///
/// # 参数
///
/// - `app` - 可变的应用状态引用
///
/// # 返回
///
/// 返回空的任务（`Task::none()`），因为此操作不需要异步处理
pub(super) fn toggle_action_menu(app: &mut App) -> Task<Message> {
    if let Some(tab) = app.active_mindmap_tab_mut() {
        // 切换显示状态
        tab.show_action_menu = !tab.show_action_menu;

        // 如果打开菜单，关闭其他所有选择器并清理状态
        if tab.show_action_menu {
            commit_url_editor_if_needed(tab);
            tab.active_color_picker = None;
            tab.show_diagram_type_picker = false;
            tab.show_markdown_import = false;
            tab.show_zoom_menu = false;
            tab.show_priority_picker = false;
            tab.show_url_editor = false;
            tab.show_text_editor = false;
            tab.show_theme_panel = false;
            tab.url_editor_value.clear();
            tab.node_text_editor = text_editor::Content::new();
        }
    }
    Task::none()
}

/// 切换优先级选择器的显示状态
///
/// 切换优先级选择器的显示/隐藏状态。如果选择器被打开：
/// - 提交未保存的 URL 编辑器内容
/// - 关闭所有其他选择器和面板
/// - 清空编辑器内容
///
/// # 参数
///
/// - `app` - 可变的应用状态引用
///
/// # 返回
///
/// 返回空的任务（`Task::none()`），因为此操作不需要异步处理
pub(super) fn toggle_priority_picker(app: &mut App) -> Task<Message> {
    if let Some(tab) = app.active_mindmap_tab_mut() {
        // 切换显示状态
        tab.show_priority_picker = !tab.show_priority_picker;

        // 如果打开选择器，关闭其他所有选择器并清理状态
        if tab.show_priority_picker {
            commit_url_editor_if_needed(tab);
            tab.active_color_picker = None;
            tab.show_diagram_type_picker = false;
            tab.show_markdown_import = false;
            tab.show_zoom_menu = false;
            tab.show_action_menu = false;
            tab.show_theme_panel = false;
            tab.show_url_editor = false;
            tab.show_text_editor = false;
            tab.url_editor_value.clear();
            tab.node_text_editor = text_editor::Content::new();
        }
    }
    Task::none()
}

/// 设置当前选中节点的优先级
///
/// 为当前选中的节点设置优先级值（1-10）。优先级为 10 表示最高优先级，
/// 其他值会被限制在 1-9 范围内。设置后会关闭优先级选择器，
/// 清空画布缓存并持久化状态。
///
/// # 参数
///
/// - `app` - 可变的应用状态引用
/// - `priority` - 优先级值（1-10，其中 10 为特殊最高优先级）
///
/// # 返回
///
/// 返回空的任务（`Task::none()`），因为此操作不需要异步处理
pub(super) fn set_node_priority(app: &mut App, priority: u8) -> Task<Message> {
    if let Some(tab) = app.active_mindmap_tab_mut() {
        if let Some(path) = tab.selected_path.clone() {
            // 优先级 10 为特殊值（最高优先级），其他值限制在 1-9 范围
            let p = if priority == 10 { 10 } else { priority.clamp(1, 9) };
            tab.node_priorities.insert(path, p);
        }

        // 关闭优先级选择器
        tab.show_priority_picker = false;

        // 清空画布缓存以触发重绘
        tab.canvas_cache.clear();

        // 持久化状态
        let _ = persist(app);
    }
    Task::none()
}

/// 清除当前选中节点的优先级
///
/// 移除当前选中节点的优先级设置，清空画布缓存并持久化状态。
///
/// # 参数
///
/// - `app` - 可变的应用状态引用
///
/// # 返回
///
/// 返回空的任务（`Task::none()`），因为此操作不需要异步处理
pub(super) fn clear_node_priority(app: &mut App) -> Task<Message> {
    if let Some(tab) = app.active_mindmap_tab_mut() {
        if let Some(path) = tab.selected_path.clone() {
            tab.node_priorities.remove(&path);
        }

        tab.show_priority_picker = false;

        // 清空画布缓存以触发重绘
        tab.canvas_cache.clear();

        // 持久化状态
        let _ = persist(app);
    }
    Task::none()
}

/// 切换节点 URL 编辑器的显示状态
///
/// 切换 URL 编辑器的显示/隐藏状态。
///
/// **关闭时**：
/// - 提交编辑器内容
/// - 关闭编辑器
/// - 清空编辑器值和画布缓存
/// - 持久化状态
///
/// **打开时**：
/// - 关闭所有其他选择器和面板
/// - 从 `node_urls` 中加载当前节点的 URL 到编辑器（如果存在）
///
/// # 参数
///
/// - `app` - 可变的应用状态引用
///
/// # 返回
///
/// 返回空的任务（`Task::none()`），因为此操作不需要异步处理
pub(super) fn toggle_node_url_editor(app: &mut App) -> Task<Message> {
    if let Some(tab) = app.active_mindmap_tab_mut() {
        // 如果编辑器已经打开，则关闭它
        if tab.show_url_editor {
            commit_url_editor_if_needed(tab);
            tab.show_url_editor = false;
            tab.url_editor_value.clear();
            tab.canvas_cache.clear();
            let _ = persist(app);
            return Task::none();
        }

        // 打开编辑器
        tab.show_url_editor = true;

        // 关闭所有其他选择器和面板
        tab.active_color_picker = None;
        tab.show_diagram_type_picker = false;
        tab.show_markdown_import = false;
        tab.show_zoom_menu = false;
        tab.show_priority_picker = false;
        tab.show_text_editor = false;
        tab.show_action_menu = false;
        tab.show_theme_panel = false;
        tab.node_text_editor = text_editor::Content::new();

        // 从现有节点 URL 加载到编辑器，或使用空字符串
        tab.url_editor_value = tab
            .selected_path
            .as_ref()
            .and_then(|p| tab.node_urls.get(p))
            .cloned()
            .unwrap_or_default();
    }
    Task::none()
}

/// 处理节点 URL 编辑器内容变化
///
/// 当用户在 URL 编辑器中输入内容时调用此函数，更新编辑器的当前值。
///
/// # 参数
///
/// - `app` - 可变的应用状态引用
/// - `v` - 新的 URL 字符串值
///
/// # 返回
///
/// 返回空的任务（`Task::none()`），因为此操作不需要异步处理
pub(super) fn node_url_changed(app: &mut App, v: String) -> Task<Message> {
    #[cfg(debug_assertions)]
    println!("NodeUrlChanged: {}", v);

    if let Some(tab) = app.active_mindmap_tab_mut() {
        // 更新编辑器中的 URL 值
        tab.url_editor_value = v;
    }
    Task::none()
}

/// 保存当前编辑的节点 URL
///
/// 将 URL 编辑器中的当前值保存到选中节点的元数据中。
/// URL 会经过处理（去除首尾空白和反引号），如果结果为空则移除该节点的 URL。
/// 保存后会清空画布缓存并持久化状态。
///
/// # 参数
///
/// - `app` - 可变的应用状态引用
///
/// # 返回
///
/// 返回空的任务（`Task::none()`），因为此操作不需要异步处理
pub(super) fn save_node_url(app: &mut App) -> Task<Message> {
    #[cfg(debug_assertions)]
    println!("SaveNodeUrl");

    if let Some(tab) = app.active_mindmap_tab_mut()
        && let Some(path) = tab.selected_path.clone() {
            // 处理 URL：去除首尾空白和反引号
            let url = tab.url_editor_value.trim().trim_matches('`').trim().to_string();

            #[cfg(debug_assertions)]
            println!("Saving URL for path {:?}: '{}'", path, url);

            // 根据处理后的 URL 是否为空，决定插入或移除
            if url.is_empty() {
                tab.node_urls.remove(&path);
            } else {
                tab.node_urls.insert(path, url);
            }

            // 清空画布缓存以触发重绘
            tab.show_url_editor = false;
            tab.url_editor_value.clear();
            tab.canvas_cache.clear();

            // 持久化状态
            let _ = persist(app);
        }
    Task::none()
}

/// 清除当前选中节点的 URL
///
/// 从选中节点的元数据中移除 URL，同时清空编辑器值、画布缓存并持久化状态。
///
/// # 参数
///
/// - `app` - 可变的应用状态引用
///
/// # 返回
///
/// 返回空的任务（`Task::none()`），因为此操作不需要异步处理
pub(super) fn clear_node_url(app: &mut App) -> Task<Message> {
    #[cfg(debug_assertions)]
    println!("ClearNodeUrl");

    if let Some(tab) = app.active_mindmap_tab_mut()
        && let Some(path) = tab.selected_path.clone() {
            #[cfg(debug_assertions)]
            println!("Clearing URL for path {:?}", path);

            // 从节点 URL 映射中移除
            tab.node_urls.remove(&path);

            // 清空编辑器值
            tab.show_url_editor = false;
            tab.url_editor_value.clear();

            // 清空画布缓存以触发重绘
            tab.canvas_cache.clear();

            // 持久化状态
            let _ = persist(app);
        }
    Task::none()
}

/// 在系统浏览器中打开当前选中节点的 URL
///
/// 根据编辑器状态获取 URL（如果编辑器打开则使用编辑器中的值，
/// 否则使用已保存的节点 URL），去除首尾空白和反引号后在系统默认浏览器中打开。
///
/// 注意：此功能仅在非 WebAssembly 目标平台上可用。
///
/// # 参数
///
/// - `app` - 可变的应用状态引用
///
/// # 返回
///
/// 返回空的任务（`Task::none()`），因为此操作不需要异步处理
pub(super) fn open_node_url(app: &mut App) -> Task<Message> {
    if let Some(tab) = app.active_mindmap_tab_mut()
        && let Some(path) = tab.selected_path.as_ref() {
            // 根据 URL 编辑器状态获取 URL
            let url = if tab.show_url_editor {
                tab.url_editor_value.clone()
            } else {
                tab.node_urls.get(path).cloned().unwrap_or_default()
            };

            // 处理 URL：去除首尾空白和反引号
            let url = url.trim().trim_matches('`').trim().to_string();

            // 如果 URL 非空，在系统浏览器中打开
            if !url.is_empty() {
                #[cfg(not(target_arch = "wasm32"))]
                {
                    let _ = open::that(url);
                }
            }
        }
    Task::none()
}

/// 在指定节点路径打开其 URL
///
/// 选中指定路径的节点并在系统浏览器中打开其关联的 URL。
/// 此函数会先提交任何未保存的 URL 编辑器内容，
/// 然后设置新的选中路径，最后打开该节点的 URL。
///
/// 注意：此功能仅在非 WebAssembly 目标平台上可用。
///
/// # 参数
///
/// - `app` - 可变的应用状态引用
/// - `path` - 目标节点的路径
///
/// # 返回
///
/// 返回空的任务（`Task::none()`），因为此操作不需要异步处理
pub(super) fn open_node_url_at(app: &mut App, path: Vec<usize>) -> Task<Message> {
    // 获取要打开的 URL，同时更新选中状态
    let url_to_open = if let Some(tab) = app.active_mindmap_tab_mut() {
        // 先提交任何未保存的 URL 编辑器内容
        commit_url_editor_if_needed(tab);

        // 设置新的选中路径
        tab.selected_path = Some(path.clone());

        // 关闭 URL 编辑器并清空编辑器值
        tab.show_url_editor = false;
        tab.url_editor_value.clear();

        // 清空画布缓存以触发重绘
        tab.canvas_cache.clear();

        // 获取该节点的 URL（如果存在）
        Some(tab.node_urls.get(&path).cloned().unwrap_or_default())
    } else {
        None
    };

    // 持久化状态
    let _ = persist(app);

    // 如果获取到 URL，在系统浏览器中打开
    if let Some(url) = url_to_open {
        // 处理 URL：去除首尾空白和反引号
        let url = url.trim().trim_matches('`').trim().to_string();

        if !url.is_empty() {
            #[cfg(not(target_arch = "wasm32"))]
            {
                let _ = open::that(url);
            }
        }
    }
    Task::none()
}
