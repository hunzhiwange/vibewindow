//! 图层面板消息处理模块
//!
//! 该模块负责处理设计编辑器中图层面板相关的所有用户交互消息，
//! 包括图层选择、拖拽排序、可见性切换、展开/折叠等操作。
//!
//! # 主要功能
//!
//! - **图层选择**：单选和多选元素节点
//! - **拖拽操作**：图层项的拖拽排序
//! - **展开/折叠**：树形结构的展开和折叠控制
//! - **可见性控制**：切换元素的显示/隐藏状态
//! - **Tailwind 节点操作**：Tailwind 组件内部节点的选择和展开
//! - **图层菜单**：右键菜单和上下文菜单的管理
//!
//! # 核心组件
//!
//! - [`update`]：主消息处理函数，路由所有图层面板相关消息
//! - [`select_single`]：单元素选择的辅助函数
//! - [`node_has_children`]：判断节点是否包含子元素

use crate::app::message::DesignMessage;
use crate::app::message::design::LayerAction;
use crate::app::views::design::canvas::tailwind::dom::TailwindNode;
use crate::app::views::design::models::DesignDoc;
use crate::app::views::design::models::compute_tree_metrics;
use crate::app::views::design::state::DesignState;
use crate::app::{App, Message};
use iced::Task;

/// 检查指定节点是否包含子元素
///
/// 该函数用于判断一个设计元素是否具有子节点，支持普通元素和引用类型元素。
/// 对于引用类型（kind == "ref"），会递归检查被引用元素的子节点。
///
/// # 参数
///
/// * `doc` - 设计文档引用，用于查找元素
/// * `id` - 待检查的元素 ID
///
/// # 返回值
///
/// 如果元素存在且包含子节点，返回 `true`；否则返回 `false`
///
/// # 示例
///
/// ```ignore
/// let has_kids = node_has_children(&doc, "element-123");
/// if has_kids {
///     // 该元素可以展开显示子节点
/// }
/// ```
fn node_has_children(doc: &DesignDoc, id: &str) -> bool {
    // 查找指定 ID 的元素，不存在则返回 false
    let Some(el) = doc.find_element(id) else {
        return false;
    };

    // 如果元素自身有子节点，直接返回 true
    if !el.children.is_empty() {
        return true;
    }

    // 对于引用类型元素，检查被引用的目标元素是否有子节点
    if el.kind == "ref"
        && let Some(ref_id) = el.reference.as_deref()
            && let Some(ref_el) = doc.find_element(ref_id)
        {
            return !ref_el.children.is_empty();
        }

    false
}

/// 执行单元素选择操作
///
/// 该函数处理选中单个设计元素的完整流程，包括：
/// 1. 保存当前正在编辑的内容（如果有）
/// 2. 更新选中状态
/// 3. 清空画布缓存和相关 UI 状态
/// 4. 展开父节点路径以确保选中元素可见
/// 5. 初始化编辑器内容（上下文编辑器、内容编辑器、Tailwind 编辑器等）
///
/// # 参数
///
/// * `state` - 设计状态的可变引用
/// * `id` - 要选中的元素 ID
///
/// # 副作用
///
/// 该函数会修改以下状态：
/// - `selected_element_id`：设置为主要选中元素
/// - `selected_element_ids`：清空并仅包含当前选中元素
/// - `canvas_cache`：清空画布缓存以触发重绘
/// - `expanded_nodes`：展开所有父节点
/// - 各种编辑器内容：根据元素类型初始化
fn select_single(state: &mut DesignState, id: String) {
    // 如果当前有正在编辑的元素，先保存其内容
    if state.editing_id.is_some() {
        if let Some(edit_id) = &state.editing_id {
            state.editing_content = state.editing_editor.text().to_string();
            state.doc.update_property(
                edit_id,
                "content",
                serde_json::Value::String(state.editing_content.clone()),
            );
        }
        state.editing_id = None;
        state.editing_content.clear();
    }

    // 更新选中状态：设置主要选中元素和选中集合
    state.selected_element_id = Some(id.clone());
    state.selected_element_ids.clear();
    state.selected_element_ids.insert(id.clone());

    // 清空画布缓存和相关 UI 状态
    state.canvas_cache.clear();
    state.selected_fill_index = None;
    state.tailwind_class_input.clear();
    state.tailwind_node_class_input.clear();
    state.tailwind_node_class_dropdown_open = false;

    // 展开从根节点到选中元素的路径，确保元素在树形视图中可见
    if let Some(path) = state.doc.find_path_to_element(&id) {
        for parent_id in path {
            state.expanded_nodes.insert(parent_id);
        }
    }

    // 根据选中元素初始化各种编辑器
    if let Some(element) = state.doc.find_element(&id) {
        // 初始化上下文编辑器
        let context = element.context.clone().unwrap_or_default();
        state.context_editor = iced::widget::text_editor::Content::with_text(&context);
        state.context_element_id = Some(id.clone());

        // 初始化内容编辑器
        let content = element.content.clone().unwrap_or_default();
        state.content_editor = iced::widget::text_editor::Content::with_text(&content);

        // 对于 Tailwind 类型元素，额外初始化 HTML 编辑器
        if element.kind.eq_ignore_ascii_case("tailwind") {
            state.tailwind_html_editor = iced::widget::text_editor::Content::with_text(&content);
        } else {
            state.tailwind_html_editor = iced::widget::text_editor::Content::new();
        }

        // 初始化 Tailwind 节点相关编辑器
        state.tailwind_node_class_editor = iced::widget::text_editor::Content::new();
        state.tailwind_node_text_editor = iced::widget::text_editor::Content::new();

        // 设置上下文面板展开状态：文本类型或有非空上下文时展开
        state.context_expanded = element.kind.eq_ignore_ascii_case("text")
            || element.context.as_deref().map(|s| !s.is_empty()).unwrap_or(false);
    } else {
        // 元素不存在时，清空所有编辑器
        state.context_editor = iced::widget::text_editor::Content::new();
        state.context_element_id = None;
        state.context_expanded = false;
        state.content_editor = iced::widget::text_editor::Content::new();
        state.tailwind_html_editor = iced::widget::text_editor::Content::new();
        state.tailwind_node_class_editor = iced::widget::text_editor::Content::new();
        state.tailwind_node_text_editor = iced::widget::text_editor::Content::new();
    }
}

/// 处理图层面板相关的所有消息
///
/// 该函数是图层面板消息处理的主入口，负责路由和处理所有与图层操作相关的用户交互。
/// 包括图层面板本身的显示/隐藏、图层项的选择、拖拽排序、可见性切换等操作。
///
/// # 参数
///
/// * `app` - 应用程序状态的可变引用
/// * `message` - 待处理的设计消息枚举
///
/// # 返回值
///
/// 返回一个 `Task<Message>`，可能包含需要执行的异步操作或 UI 命令。
/// 大多数操作返回 `Task::none()`，部分操作（如滚动到特定位置）会返回相应的任务。
///
/// # 处理的消息类型
///
/// ## 面板控制
/// - `ToggleLayerPanel`：切换图层面板的显示/隐藏
/// - `LayerPanelResizing`：处理面板宽度调整
///
/// ## 拖拽操作
/// - `LayerDragStart`：开始拖拽图层项
/// - `LayerDragOver`：拖拽经过目标图层项
/// - `LayerHover`：鼠标悬停在图层项上
/// - `LayerHoverLeave`：鼠标离开图层项
/// - `LayerDrop`：完成拖拽放置
///
/// ## 图层菜单
/// - `LayerMenuHover`：菜单项悬停
/// - `LayerMenuLeave`：菜单项离开
/// - `LayerMenuToggle`：切换菜单显示状态
/// - `LayerMenuClose`：关闭菜单
///
/// ## 选择操作
/// - `ElementSelected`：选中单个元素
/// - `SelectTailwindNode`：选中 Tailwind 组件内部节点
/// - `LayerRowPressed`：点击图层行
/// - `MultiSelect`：多选元素
///
/// ## 节点操作
/// - `ToggleNode`：切换节点展开/折叠
/// - `ToggleVisible`：切换元素可见性
/// - `MoveLayerItem`：移动图层项位置
/// - `LayerActionSelected`：执行图层菜单动作
///
/// # 示例
///
/// ```ignore
/// let task = update(&mut app, DesignMessage::ToggleLayerPanel);
/// // 执行返回的任务
/// ```
pub fn update(app: &mut App, message: DesignMessage) -> Task<Message> {
    match message {
        // ==================== 面板控制相关消息 ====================

        // 切换图层面板的显示/隐藏状态，并持久化到配置
        DesignMessage::ToggleLayerPanel => {
            app.show_layer_panel = !app.show_layer_panel;
            crate::app::set_config_field(
                "show_layer_panel",
                serde_json::Value::Bool(app.show_layer_panel),
            );
            Task::none()
        }

        // 处理面板宽度调整，限制在 150-500 像素范围内
        DesignMessage::LayerPanelResizing(width) => {
            let w = width.clamp(150.0, 500.0);
            app.layer_panel_width = w;
            Task::none()
        }

        // ==================== 拖拽操作相关消息 ====================

        // 开始拖拽：记录正在拖拽的图层 ID
        DesignMessage::LayerDragStart(id) => {
            app.dragging_layer = Some(id);
            Task::none()
        }

        // 拖拽经过：更新拖拽目标位置
        DesignMessage::LayerDragOver(id) => {
            if let Some(drag_id) = &app.dragging_layer {
                // 不能拖到自己身上
                if drag_id != &id {
                    if app.drag_target_layer.as_deref() != Some(&id) {
                        app.drag_target_layer = Some(id);
                    } else {
                        return Task::none();
                    }
                }
            }
            Task::none()
        }

        // 悬停处理：更新悬停状态，同时在拖拽时更新目标位置
        DesignMessage::LayerHover(id) => {
            if app.hovered_layer_id.as_deref() == Some(&id) {
                Task::none()
            } else {
                app.hovered_layer_id = Some(id.clone());
                // 如果正在拖拽，同时更新拖拽目标
                if let Some(drag_id) = &app.dragging_layer
                    && drag_id != &id && app.drag_target_layer.as_deref() != Some(&id) {
                        app.drag_target_layer = Some(id);
                    }
                Task::none()
            }
        }

        // 悬停离开：清除悬停状态
        DesignMessage::LayerHoverLeave => {
            if app.hovered_layer_id.is_some() {
                app.hovered_layer_id = None;
            }
            Task::none()
        }

        // ==================== 图层菜单相关消息 ====================

        // 菜单项悬停：更新活动菜单
        DesignMessage::LayerMenuHover(id) => {
            if app.active_layer_menu.as_deref() != Some(&id) {
                app.active_layer_menu = Some(id);
            }
            Task::none()
        }

        // 菜单项离开：关闭菜单
        DesignMessage::LayerMenuLeave => {
            app.active_layer_menu = None;
            app.layer_menu_anchor = None;
            Task::none()
        }

        // 切换菜单显示：如果已打开则关闭，否则在指定位置打开
        DesignMessage::LayerMenuToggle(id, x, y) => {
            if app.active_layer_menu.as_deref() == Some(&id) {
                app.active_layer_menu = None;
                app.layer_menu_anchor = None;
            } else {
                app.active_layer_menu = Some(id);
                app.layer_menu_anchor = Some(iced::Point::new(x, y));
            }
            Task::none()
        }

        // 关闭菜单
        DesignMessage::LayerMenuClose => {
            app.active_layer_menu = None;
            app.layer_menu_anchor = None;
            Task::none()
        }

        // ==================== 拖拽放置处理 ====================

        // 完成拖拽放置：将元素从原位置移动到目标位置之前
        DesignMessage::LayerDrop => {
            let drag_id = app.dragging_layer.take();
            let target_id = app.drag_target_layer.take();
            if let (Some(drag_id), Some(target_id)) = (drag_id, target_id) {
                // 确保不是拖到自己身上
                if drag_id != target_id
                    && let Some(state) = app.active_design_state_mut() {
                        // 从树中移除被拖拽的节点
                        fn remove_node(
                            children: &mut Vec<crate::app::views::design::models::DesignElement>,
                            id: &str,
                        ) -> Option<crate::app::views::design::models::DesignElement>
                        {
                            // 在当前层级查找
                            if let Some(idx) = children.iter().position(|c| c.id == id) {
                                return Some(children.remove(idx));
                            }
                            // 递归查找子节点
                            for child in children {
                                if let Some(el) = remove_node(&mut child.children, id) {
                                    return Some(el);
                                }
                            }
                            None
                        }

                        // 将节点插入到目标位置之前
                        fn insert_node(
                            children: &mut Vec<crate::app::views::design::models::DesignElement>,
                            target_id: &str,
                            element: crate::app::views::design::models::DesignElement,
                        ) -> Result<(), crate::app::views::design::models::DesignElement>
                        {
                            // 在当前层级查找目标位置
                            if let Some(idx) = children.iter().position(|c| c.id == target_id) {
                                children.insert(idx, element);
                                return Ok(());
                            }
                            // 递归查找子节点
                            let mut element = element;
                            for child in children {
                                match insert_node(&mut child.children, target_id, element) {
                                    Ok(_) => return Ok(()),
                                    Err(returned) => element = returned,
                                }
                            }
                            Err(element)
                        }

                        // 执行移动操作
                        if let Some(element) = remove_node(&mut state.doc.children, &drag_id) {
                            if let Err(element) =
                                insert_node(&mut state.doc.children, &target_id, element)
                            {
                                // 如果找不到目标位置，添加到末尾
                                state.doc.children.push(element);
                            }
                            // 更新树指标和清空缓存
                            state.layer_tree_metrics = compute_tree_metrics(&state.doc);
                            state.canvas_cache.clear();
                        }
                    }
            }
            Task::none()
        }
        // ==================== 其他消息处理 ====================
        // 这里处理需要活跃设计状态的消息
        _ => {
            // 对于特定消息，先关闭图层菜单
            if matches!(
                &message,
                DesignMessage::ElementSelected(_)
                    | DesignMessage::LayerRowPressed(_)
                    | DesignMessage::ToggleNode(_)
                    | DesignMessage::ToggleVisible(_)
                    | DesignMessage::MoveLayerItem(_, _)
                    | DesignMessage::LayerActionSelected(_, _)
            ) {
                app.active_layer_menu = None;
                app.layer_menu_anchor = None;
            }

            // 获取活跃的设计状态并处理消息
            if let Some(state) = app.active_design_state_mut() {
                match message {
                    // 选中单个元素
                    DesignMessage::ElementSelected(id) => {
                        select_single(state, id);
                        Task::none()
                    }

                    // 选中 Tailwind 组件内部的特定节点
                    DesignMessage::SelectTailwindNode(id, path) => {
                        // 记录 Tailwind 选择状态
                        state.doc.tailwind_selection = Some((id.clone(), path.clone()));
                        // 同时选中父元素
                        select_single(state, id.clone());

                        // 展开从根到目标节点的路径
                        for i in 1..path.len() {
                            let k = tailwind_collapse_key(&id, &path[..i]);
                            state.tailwind_tree_collapsed.remove(&k);
                        }

                        let mut scroll_to: Option<f32> = None;

                        // 处理 Tailwind 元素的节点选择
                        if let Some(element) = state.doc.find_element(&id)
                            && element.kind.eq_ignore_ascii_case("tailwind")
                            && let Some(content) = element.content.as_deref()
                        {
                            // 解析 HTML 内容为节点树
                            let nodes =
                                crate::app::views::design::canvas::tailwind::dom::parse_html(
                                    content,
                                );

                            // 计算所有可见节点的路径
                            let visible_paths = {
                                // 递归遍历节点树，收集可见路径
                                fn walk(
                                    element_id: &str,
                                    nodes: &[TailwindNode],
                                    collapsed: &std::collections::HashSet<String>,
                                    prefix: &mut Vec<usize>,
                                    out: &mut Vec<Vec<usize>>,
                                ) {
                                    for (i, node) in nodes.iter().enumerate() {
                                        prefix.push(i);
                                        out.push(prefix.clone());
                                        let k =
                                            tailwind_collapse_key(element_id, prefix.as_slice());
                                        let is_collapsed = collapsed.contains(&k);
                                        // 只有未折叠且有子节点时才递归
                                        if !is_collapsed && !node.children.is_empty() {
                                            walk(
                                                element_id,
                                                &node.children,
                                                collapsed,
                                                prefix,
                                                out,
                                            );
                                        }
                                        prefix.pop();
                                    }
                                }

                                let mut out = Vec::new();
                                let mut prefix = Vec::new();
                                walk(
                                    &id,
                                    &nodes,
                                    &state.tailwind_tree_collapsed,
                                    &mut prefix,
                                    &mut out,
                                );
                                out
                            };

                            // 计算滚动位置，使选中的节点可见
                            if let Some(idx) = visible_paths.iter().position(|p| p == &path) {
                                let denom = visible_paths.len().saturating_sub(1).max(1) as f32;
                                scroll_to = Some((idx as f32 / denom).clamp(0.0, 1.0));
                            }

                            // 获取选中节点并初始化编辑器
                            if let Some(node) =
                                crate::app::views::design::canvas::tailwind::dom::get_node_by_path(
                                    &nodes, &path,
                                )
                            {
                                // 对于文本节点，class 应该来自父节点
                                let class_target = if node.text.is_some() && path.len() > 1 {
                                    crate::app::views::design::canvas::tailwind::dom::get_node_by_path(
                                        &nodes,
                                        &path[..path.len() - 1],
                                    )
                                    .unwrap_or(node)
                                } else {
                                    node
                                };

                                // 初始化 class 和 text 编辑器
                                let class_val = class_target
                                    .attributes
                                    .get("class")
                                    .map(|s| s.as_str())
                                    .unwrap_or("")
                                    .to_string();
                                let text_val = node.text.as_deref().unwrap_or("").to_string();
                                state.tailwind_node_class_editor =
                                    iced::widget::text_editor::Content::with_text(&class_val);
                                state.tailwind_node_text_editor =
                                    iced::widget::text_editor::Content::with_text(&text_val);
                            }
                        }

                        // 如果需要滚动，返回滚动任务
                        if let Some(y) = scroll_to {
                            iced::widget::operation::snap_to(
                                state.tailwind_tree_scroll_id.clone(),
                                iced::widget::scrollable::RelativeOffset { x: None, y: Some(y) },
                            )
                            .map(|_: ()| Message::None)
                        } else {
                            Task::none()
                        }
                    }

                    // 切换 Tailwind 检查器的折叠状态
                    DesignMessage::ToggleTailwindInspectorCollapsed => {
                        state.tailwind_inspector_collapsed = !state.tailwind_inspector_collapsed;
                        Task::none()
                    }

                    // 切换 Tailwind 节点树的折叠状态
                    DesignMessage::ToggleTailwindTreeCollapsed(id, path) => {
                        let k = tailwind_collapse_key(&id, &path);
                        if state.tailwind_tree_collapsed.contains(&k) {
                            state.tailwind_tree_collapsed.remove(&k);
                        } else {
                            state.tailwind_tree_collapsed.insert(k);
                        }
                        Task::none()
                    }

                    // 点击图层行：选中元素，如果有子节点则切换展开状态
                    DesignMessage::LayerRowPressed(id) => {
                        let should_toggle = node_has_children(&state.doc, &id);
                        select_single(state, id.clone());
                        if should_toggle {
                            if state.expanded_nodes.contains(&id) {
                                state.expanded_nodes.remove(&id);
                            } else {
                                state.expanded_nodes.insert(id);
                            }
                        }
                        Task::none()
                    }

                    // 多选元素
                    DesignMessage::MultiSelect(ids) => {
                        // 保存当前正在编辑的内容
                        if state.editing_id.is_some() {
                            if let Some(edit_id) = &state.editing_id {
                                state.editing_content = state.editing_editor.text().to_string();
                                state.doc.update_property(
                                    edit_id,
                                    "content",
                                    serde_json::Value::String(state.editing_content.clone()),
                                );
                            }
                            state.editing_id = None;
                            state.editing_content.clear();
                        }

                        // 更新选中集合
                        state.selected_element_ids.clear();
                        for id in &ids {
                            state.selected_element_ids.insert(id.clone());
                        }
                        // 设置主要选中元素为第一个，如果集合为空则为 None
                        state.selected_element_id = ids.first().cloned();
                        state.canvas_cache.clear();
                        state.selected_fill_index = None;

                        // 为主要选中元素初始化编辑器
                        if let Some(primary_id) = state.selected_element_id.clone()
                            && let Some(element) = state.doc.find_element(&primary_id)
                        {
                            let context = element.context.clone().unwrap_or_default();
                            state.context_editor =
                                iced::widget::text_editor::Content::with_text(&context);
                            state.context_element_id = Some(primary_id.clone());

                            let content = element.content.clone().unwrap_or_default();
                            state.content_editor =
                                iced::widget::text_editor::Content::with_text(&content);
                            state.context_expanded = element.kind.eq_ignore_ascii_case("text")
                                || element
                                    .context
                                    .as_deref()
                                    .map(|s| !s.is_empty())
                                    .unwrap_or(false);
                        } else {
                            // 没有主要选中元素时，清空编辑器
                            state.context_editor = iced::widget::text_editor::Content::new();
                            state.context_element_id = None;
                            state.context_expanded = false;
                            state.content_editor = iced::widget::text_editor::Content::new();
                        }
                        Task::none()
                    }

                    // 切换节点的展开/折叠状态
                    DesignMessage::ToggleNode(id) => {
                        if state.expanded_nodes.contains(&id) {
                            state.expanded_nodes.remove(&id);
                        } else {
                            state.expanded_nodes.insert(id);
                        }
                        Task::none()
                    }

                    // 切换元素的可见性
                    DesignMessage::ToggleVisible(id) => {
                        let current_vis =
                            state.doc.find_element(&id).and_then(|e| e.visible).unwrap_or(true);
                        state.doc.update_property(
                            &id,
                            "visible",
                            serde_json::Value::Bool(!current_vis),
                        );
                        state.canvas_cache.clear();
                        Task::none()
                    }

                    // 移动图层项的位置（向上或向下）
                    DesignMessage::MoveLayerItem(id, delta) => {
                        // 递归查找并移动节点
                        fn move_node(
                            children: &mut Vec<crate::app::views::design::models::DesignElement>,
                            id: &str,
                            delta: i32,
                        ) -> bool {
                            // 在当前层级查找
                            if let Some(idx) = children.iter().position(|c| c.id == id) {
                                let new_idx = idx as i32 + delta;
                                // 检查新位置是否有效
                                if new_idx >= 0 && new_idx < children.len() as i32 {
                                    children.swap(idx, new_idx as usize);
                                    return true;
                                }
                                return false;
                            }
                            // 递归查找子节点
                            for child in children {
                                if move_node(&mut child.children, id, delta) {
                                    return true;
                                }
                            }
                            false
                        }
                        move_node(&mut state.doc.children, &id, delta);
                        state.canvas_cache.clear();
                        Task::none()
                    }

                    // 处理图层菜单动作
                    DesignMessage::LayerActionSelected(id, action) => match action {
                        // 切换可见性
                        LayerAction::ToggleVisible => update(app, DesignMessage::ToggleVisible(id)),
                        // 上移一位
                        LayerAction::MoveUp => update(app, DesignMessage::MoveLayerItem(id, -1)),
                        // 下移一位
                        LayerAction::MoveDown => update(app, DesignMessage::MoveLayerItem(id, 1)),
                        // 删除元素
                        LayerAction::Delete => {
                            // 递归查找并删除节点
                            fn remove_node(
                                children: &mut Vec<
                                    crate::app::views::design::models::DesignElement,
                                >,
                                id: &str,
                            ) -> Option<crate::app::views::design::models::DesignElement>
                            {
                                // 在当前层级查找
                                if let Some(idx) = children.iter().position(|c| c.id == id) {
                                    return Some(children.remove(idx));
                                }
                                // 递归查找子节点
                                for child in children {
                                    if let Some(el) = remove_node(&mut child.children, id) {
                                        return Some(el);
                                    }
                                }
                                None
                            }

                            // 执行删除
                            let _ = remove_node(&mut state.doc.children, &id);
                            // 清除选中状态
                            state.selected_element_id = None;
                            state.selected_element_ids.remove(&id);
                            // 更新树指标和清空缓存
                            state.layer_tree_metrics = compute_tree_metrics(&state.doc);
                            state.canvas_cache.clear();
                            Task::none()
                        }
                    },
                    _ => Task::none(),
                }
            } else {
                Task::none()
            }
        }
    }
}

/// 生成 Tailwind 节点树的折叠状态键
///
/// 该函数根据元素 ID 和节点路径生成一个唯一的字符串键，
/// 用于在 `tailwind_tree_collapsed` 集合中标识特定的节点。
///
/// # 参数
///
/// * `id` - Tailwind 元素的 ID
/// * `path` - 节点在 HTML 树中的路径（索引数组）
///
/// # 返回值
///
/// 返回格式为 `"{id}|{path}"` 的字符串，其中路径部分用点号分隔。
/// 例如：`"element-123|0.1.2"`
///
/// # 示例
///
/// ```ignore
/// let key = tailwind_collapse_key("my-element", &[0, 1, 2]);
/// assert_eq!(key, "my-element|0.1.2");
/// ```
fn tailwind_collapse_key(id: &str, path: &[usize]) -> String {
    // 预分配足够容量以避免多次重分配
    let mut s = String::with_capacity(id.len() + 1 + path.len() * 3);
    s.push_str(id);
    s.push('|');
    // 将路径索引用点号连接
    for (i, p) in path.iter().enumerate() {
        if i > 0 {
            s.push('.');
        }
        s.push_str(&p.to_string());
    }
    s
}

