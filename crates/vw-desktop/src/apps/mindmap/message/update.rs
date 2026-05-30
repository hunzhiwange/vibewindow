//! 思维导图消息更新处理模块
//!
//! 本模块实现了思维导图应用的消息分发和状态更新逻辑。作为 iced 框架中
//! `update` 函数的具体实现，它接收用户交互产生的各类消息，并根据消息类型
//! 调用相应的处理函数来更新应用状态或执行副作用操作。
//!
//! # 模块结构
//!
//! 消息处理被分散到多个子模块中，每个子模块负责一个功能领域：
//!
//! - [`canvas_ops`]: 画布操作（平移、缩放、工具切换、涂鸦等）
//! - [`color_ops`]: 颜色操作（颜色选择器、主题设置、样式配置等）
//! - [`file_ops`]: 文件操作（新建、打开、保存、导出等）
//! - [`markdown_ops`]: Markdown 导入操作
//! - [`node_meta_ops`]: 节点元数据操作（选择、优先级、URL、布局格式等）
//! - [`node_ops`]: 节点核心操作（文本编辑、增删改、复制粘贴、撤销重做等）
//!
//! # 设计模式
//!
//! 本模块采用"消息分发器"模式，`update` 函数作为中央路由器，将每种消息类型
//! 映射到对应的处理函数。这种设计使得代码职责清晰、易于测试和维护。

use crate::app::{App, Message};
use iced::Task;

use super::persist;
use super::types::MindMapMessage;

mod canvas_ops;
mod color_ops;
mod file_ops;
mod markdown_ops;
mod node_meta_ops;
mod node_ops;
mod node_ops_clipboard;
mod node_ops_helpers;
mod node_ops_history;
mod node_ops_structure;
mod node_ops_text;

#[cfg(test)]
mod canvas_ops_tests;
#[cfg(test)]
mod color_ops_tests;
#[cfg(test)]
mod markdown_ops_tests;
#[cfg(test)]
mod node_meta_ops_tests;
#[cfg(test)]
mod node_ops_clipboard_tests;
#[cfg(test)]
mod node_ops_helpers_tests;
#[cfg(test)]
mod node_ops_history_tests;
#[cfg(test)]
mod node_ops_structure_tests;
#[cfg(test)]
mod node_ops_tests;
#[cfg(test)]
mod node_ops_text_tests;
#[cfg(test)]
mod update_tests;

/// 处理思维导图应用的消息更新
///
/// 这是 iced 框架中 `update` 函数的核心实现。它接收一个 `MindMapMessage` 枚举，
/// 根据消息类型执行相应的状态更新或副作用操作，并返回可能产生的 `Task`。
///
/// # 参数
///
/// * `app` - 可变引用的应用状态，包含思维导图的所有数据和 UI 状态
/// * `message` - 要处理的消息，封装了用户交互或系统事件的具体类型
///
/// # 返回值
///
/// 返回一个 `Task<Message>`，可能包含需要执行的异步操作或命令。
/// 如果消息处理不需要产生副作用，则返回 `Task::none()`。
///
/// # 消息分类
///
/// 消息按功能领域分为以下几类：
///
/// ## 文件操作
/// - `New`, `Open`, `Save`, `SaveAs`, `SaveAsJson` - 基本文件操作
/// - `FileOpened`, `FileSaved`, `SaveFinished` - 文件操作完成回调
/// - `ToggleExportMenu`, `ExportPng`, `ExportJpeg`, `ExportSvg` - 导出功能
///
/// ## 画布操作
/// - `PanBy`, `Zoom`, `ZoomSet`, `ZoomFit` - 视图变换
/// - `ToggleZoomMenu` - 缩放菜单控制
/// - `SetCanvasTool` - 绘图工具切换
/// - `SetDoodleColor`, `SetDoodleWidth` - 涂鸦配置
/// - `DoodleCommit`, `DoodleErase` - 涂鸦操作
///
/// ## 节点选择与拖拽
/// - `SelectNode`, `ClearSelection` - 节点选择
/// - `NodeDragStart`, `NodeDragged` - 节点拖拽
///
/// ## 节点元数据
/// - `ToggleActionMenu`, `TogglePriorityPicker` - 菜单控制
/// - `SetNodePriority`, `ClearNodePriority` - 优先级管理
/// - `ToggleNodeUrlEditor`, `NodeUrlChanged`, `SaveNodeUrl`, `ClearNodeUrl`, `OpenNodeUrl` - URL 管理
/// - `ToggleDiagramTypePicker`, `SetDiagramType` - 图表类型
/// - `SetLayoutFormat`, `Set*LayoutFormat` - 布局格式配置
/// - `ToggleThemePanel` - 主题面板
///
/// ## 节点文本编辑
/// - `ToggleNodeTextEditor`, `NodeTextChanged`, `SaveNodeText` - 文本编辑
/// - `NodeTextEditorAction`, `NodeTextEditorEnter` - 编辑器交互
///
/// ## 颜色与样式
/// - `OpenColorPicker`, `ColorPickerChanged`, `ColorPickerFormatChanged` - 颜色选择
/// - `ResetColorTarget`, `SetBackground` - 背景设置
/// - `SetThemeGroup`, `SetThemeVariant`, `SaveThemeToCustom`, `DeleteCustomTheme` - 主题管理
/// - `CancelThemeBackground`, `SetEdgeStyle`, `SetNodeBorderStyle` - 样式设置
///
/// ## Markdown 导入
/// - `ToggleMarkdownImport`, `MarkdownImportEditorAction`, `ApplyMarkdownImport`
///
/// ## 节点操作
/// - `AddChild`, `AddSibling`, `AddChildAt`, `AddSiblingAt` - 添加节点
/// - `ToggleCollapseAt` - 折叠/展开
/// - `OpenNodeContextMenu`, `CloseContextMenu` - 右键菜单
/// - `CutNode`, `CopyNode`, `PasteNode`, `DuplicateNode`, `DeleteNode` - 剪贴板操作
/// - `Undo`, `Redo` - 撤销/重做
///
/// # 示例
///
/// ```ignore
/// // 在应用的 update 方法中调用
/// fn update(&mut self, message: Self::Message) -> Task<Message> {
///     match message {
///         Message::MindMap(msg) => mindmap::message::update(self, msg),
///         // ... 其他消息类型
///     }
/// }
/// ```
pub fn update(app: &mut App, message: MindMapMessage) -> Task<Message> {
    // 调试构建时打印消息类型，便于开发调试
    #[cfg(debug_assertions)]
    println!("MindMap Update: {:?}", message);

    match message {
        // ==================== 文件操作 ====================
        // 新建空白思维导图标签页
        MindMapMessage::New => file_ops::new_tab(app),
        // 打开文件对话框，选择要打开的文件
        MindMapMessage::Open => file_ops::open(),
        // 文件打开操作完成后的回调处理
        MindMapMessage::FileOpened(res) => file_ops::file_opened(app, res),
        MindMapMessage::LoadPersistedFinished(res) => persist::load_persisted_finished(app, res),
        // 保存当前思维导图到文件
        MindMapMessage::Save => file_ops::save(app),
        // 保存操作完成后的回调处理
        MindMapMessage::SaveFinished(res) => file_ops::save_finished(app, res),
        // 另存为对话框（默认格式）
        MindMapMessage::SaveAs => file_ops::save_as(app),
        // 另存为 JSON 格式
        MindMapMessage::SaveAsJson => file_ops::save_as_json(app),
        // 切换导出菜单的显示/隐藏状态
        MindMapMessage::ToggleExportMenu => node_meta_ops::toggle_export_menu(app),
        // 导出为 PNG 图片格式
        MindMapMessage::ExportPng => file_ops::export_png(app),
        // 导出为 JPEG 图片格式
        MindMapMessage::ExportJpeg => file_ops::export_jpeg(app),
        // 导出为 SVG 矢量图格式
        MindMapMessage::ExportSvg => file_ops::export_svg(app),
        // 导出操作完成后的回调处理
        MindMapMessage::ExportFinished(res) => file_ops::export_finished(app, res),
        // 文件保存完成后的路径回调
        MindMapMessage::FileSaved(path) => file_ops::file_saved(app, path),

        // ==================== 画布视图操作 ====================
        // 按指定偏移量平移画布视图
        MindMapMessage::PanBy(delta) => canvas_ops::pan_by(app, delta),
        // 按指定倍数缩放视图，可指定缩放中心点
        MindMapMessage::Zoom(factor, center_opt) => canvas_ops::zoom(app, factor, center_opt),
        // 直接设置视图的缩放级别
        MindMapMessage::ZoomSet(zoom) => canvas_ops::zoom_set(app, zoom),
        // 自动调整缩放以适应画布内容
        MindMapMessage::ZoomFit => canvas_ops::zoom_fit(app),
        // 切换缩放菜单的显示/隐藏状态
        MindMapMessage::ToggleZoomMenu => canvas_ops::toggle_zoom_menu(app),
        // 选择指定路径的节点
        MindMapMessage::SelectNode(path) => node_meta_ops::select_node(app, path),
        // 清除当前节点选择状态
        MindMapMessage::ClearSelection => node_meta_ops::clear_selection(app),
        // 开始节点拖拽操作，记录初始位置和点击坐标
        MindMapMessage::NodeDragStart(path, pos, click_screen) => {
            canvas_ops::node_drag_start(app, path, pos, click_screen)
        }
        // 节点拖拽过程中的位置更新
        MindMapMessage::NodeDragged(path, delta) => canvas_ops::node_dragged(app, path, delta),

        // ==================== 画布工具与涂鸦 ====================
        // 切换当前使用的画布工具（选择、涂鸦、橡皮擦等）
        MindMapMessage::SetCanvasTool(tool) => canvas_ops::set_canvas_tool(app, tool),
        // 设置涂鸦笔画的颜色
        MindMapMessage::SetDoodleColor(rgba) => canvas_ops::set_doodle_color(app, rgba),
        // 设置涂鸦笔画的宽度（像素）
        MindMapMessage::SetDoodleWidth(width_px) => canvas_ops::set_doodle_width(app, width_px),
        // 提交一条新的涂鸦笔画到画布
        MindMapMessage::DoodleCommit(stroke) => canvas_ops::doodle_commit(app, stroke),
        // 擦除指定区域内的涂鸦笔画
        MindMapMessage::DoodleErase(center_world, radius_world) => {
            canvas_ops::doodle_erase(app, center_world, radius_world)
        }

        // ==================== 节点元数据操作 ====================
        // 关闭所有选择器面板
        MindMapMessage::ClosePickers => node_meta_ops::close_pickers(app),
        // 切换操作菜单的显示/隐藏状态
        MindMapMessage::ToggleActionMenu => node_meta_ops::toggle_action_menu(app),
        // 切换优先级选择器的显示/隐藏状态
        MindMapMessage::TogglePriorityPicker => node_meta_ops::toggle_priority_picker(app),
        // 设置当前选中节点的优先级
        MindMapMessage::SetNodePriority(priority) => {
            node_meta_ops::set_node_priority(app, priority)
        }
        // 清除当前选中节点的优先级设置
        MindMapMessage::ClearNodePriority => node_meta_ops::clear_node_priority(app),
        // 切换节点 URL 编辑器的显示/隐藏状态
        MindMapMessage::ToggleNodeUrlEditor => node_meta_ops::toggle_node_url_editor(app),
        // 节点 URL 输入框内容变化
        MindMapMessage::NodeUrlChanged(v) => node_meta_ops::node_url_changed(app, v),
        // 保存节点 URL 编辑结果
        MindMapMessage::SaveNodeUrl => node_meta_ops::save_node_url(app),
        // 清除节点的 URL 设置
        MindMapMessage::ClearNodeUrl => node_meta_ops::clear_node_url(app),
        // 在浏览器中打开当前选中节点的 URL
        MindMapMessage::OpenNodeUrl => node_meta_ops::open_node_url(app),
        // 在浏览器中打开指定路径节点的 URL
        MindMapMessage::OpenNodeUrlAt(path) => node_meta_ops::open_node_url_at(app, path),

        // ==================== 节点文本编辑 ====================
        // 切换节点文本编辑器的显示/隐藏状态
        MindMapMessage::ToggleNodeTextEditor => node_ops::toggle_node_text_editor(app),
        // 节点文本编辑器内容变化
        MindMapMessage::NodeTextChanged(v) => node_ops::node_text_changed(app, v),
        // 处理文本编辑器的操作动作（如光标移动、选择等）
        MindMapMessage::NodeTextEditorAction(action) => {
            node_ops::node_text_editor_action(app, action)
        }
        // 处理文本编辑器中的 Enter 键按下事件
        MindMapMessage::NodeTextEditorEnter { shift } => {
            node_ops::node_text_editor_enter(app, shift)
        }
        // 保存节点文本编辑结果
        MindMapMessage::SaveNodeText => node_ops::save_node_text(app),

        // ==================== 颜色与主题操作 ====================
        // 打开颜色选择器，指定目标和初始颜色
        MindMapMessage::OpenColorPicker(target, color) => {
            color_ops::open_color_picker(app, target, color)
        }
        // 颜色选择器颜色变化回调
        MindMapMessage::ColorPickerChanged(color) => color_ops::color_picker_changed(app, color),
        // 颜色选择器格式变化回调
        MindMapMessage::ColorPickerFormatChanged(format) => {
            color_ops::color_picker_format_changed(app, format)
        }
        // 重置指定颜色目标到默认值
        MindMapMessage::ResetColorTarget(target) => color_ops::reset_color_target(app, target),
        // 设置画布背景样式
        MindMapMessage::SetBackground(bg) => color_ops::set_background(app, bg),

        // ==================== 图表类型与布局 ====================
        // 切换图表类型选择器的显示/隐藏状态
        MindMapMessage::ToggleDiagramTypePicker => node_meta_ops::toggle_diagram_type_picker(app),
        // 设置当前图表类型并关闭选择器
        MindMapMessage::SelectDiagramType(diagram_type) => {
            node_meta_ops::select_diagram_type(app, diagram_type)
        }
        // 设置当前图表类型
        MindMapMessage::SetDiagramType(diagram_type) => {
            node_meta_ops::set_diagram_type(app, diagram_type)
        }
        // 设置通用布局格式
        MindMapMessage::SetLayoutFormat(format) => node_meta_ops::set_layout_format(app, format),
        // 设置组织结构图布局格式
        MindMapMessage::SetOrgChartLayoutFormat(format) => {
            node_meta_ops::set_org_chart_layout_format(app, format)
        }
        // 设置鱼骨图布局格式
        MindMapMessage::SetFishboneLayoutFormat(format) => {
            node_meta_ops::set_fishbone_layout_format(app, format)
        }
        // 设置时间线布局格式
        MindMapMessage::SetTimelineLayoutFormat(format) => {
            node_meta_ops::set_timeline_layout_format(app, format)
        }
        // 设置括号图布局格式
        MindMapMessage::SetBracketLayoutFormat(format) => {
            node_meta_ops::set_bracket_layout_format(app, format)
        }
        // 设置树形图布局格式
        MindMapMessage::SetTreeLayoutFormat(format) => {
            node_meta_ops::set_tree_layout_format(app, format)
        }

        // ==================== 主题管理 ====================
        // 切换主题面板的显示/隐藏状态
        MindMapMessage::ToggleThemePanel => node_meta_ops::toggle_theme_panel(app),
        // 设置当前主题组
        MindMapMessage::SetThemeGroup(group_id) => color_ops::set_theme_group(app, group_id),
        // 设置主题组中的特定变体
        MindMapMessage::SetThemeVariant(group_id, variant) => {
            color_ops::set_theme_variant(app, group_id, variant)
        }
        // 将当前主题保存到自定义主题列表
        MindMapMessage::SaveThemeToCustom => color_ops::save_theme_to_custom(app),
        // 删除指定索引的自定义主题
        MindMapMessage::DeleteCustomTheme(index) => color_ops::delete_custom_theme(app, index),
        // 取消主题背景设置
        MindMapMessage::CancelThemeBackground => color_ops::cancel_theme_background(app),
        // 设置连接线样式
        MindMapMessage::SetEdgeStyle(style) => color_ops::set_edge_style(app, style),
        // 设置节点边框样式
        MindMapMessage::SetNodeBorderStyle(style) => color_ops::set_node_border_style(app, style),

        // ==================== Markdown 导入 ====================
        // 切换 Markdown 导入对话框的显示/隐藏状态
        MindMapMessage::ToggleMarkdownImport => markdown_ops::toggle_markdown_import(app),
        // 处理 Markdown 导入编辑器的操作动作
        MindMapMessage::MarkdownImportEditorAction(action) => {
            markdown_ops::markdown_import_editor_action(app, action)
        }
        // 应用 Markdown 导入，将内容转换为思维导图节点
        MindMapMessage::ApplyMarkdownImport => markdown_ops::apply_markdown_import(app),

        // ==================== 节点增删改操作 ====================
        // 为当前选中节点添加子节点
        MindMapMessage::AddChild => node_ops::add_child(app),
        // 为当前选中节点添加兄弟节点
        MindMapMessage::AddSibling => node_ops::add_sibling(app),
        // 为指定路径的节点添加子节点
        MindMapMessage::AddChildAt(path) => node_ops::add_child_at(app, path),
        // 为指定路径的节点添加兄弟节点
        MindMapMessage::AddSiblingAt(path) => node_ops::add_sibling_at(app, path),
        // 切换指定路径节点的折叠/展开状态
        MindMapMessage::ToggleCollapseAt(path) => node_ops::toggle_collapse_at(app, path),

        // ==================== 右键上下文菜单 ====================
        // 在指定位置打开节点的右键上下文菜单
        MindMapMessage::OpenNodeContextMenu(path, anchor) => {
            node_ops::open_context_menu(app, path, anchor)
        }
        // 关闭右键上下文菜单
        MindMapMessage::CloseContextMenu => node_ops::close_context_menu(app),

        // ==================== 剪贴板与节点操作 ====================
        // 剪切当前选中节点到剪贴板
        MindMapMessage::CutNode => node_ops::cut_node(app),
        // 复制当前选中节点到剪贴板
        MindMapMessage::CopyNode => node_ops::copy_node(app),
        // 从剪贴板粘贴节点
        MindMapMessage::PasteNode => node_ops::paste_node(app),
        // 复制并创建当前选中节点的副本
        MindMapMessage::DuplicateNode => node_ops::duplicate_node(app),
        // 删除当前选中的节点
        MindMapMessage::DeleteNode => node_ops::delete_node(app),

        // ==================== 撤销/重做 ====================
        // 撤销上一步操作
        MindMapMessage::Undo => node_ops::undo_node(app),
        // 重做上一步被撤销的操作
        MindMapMessage::Redo => node_ops::redo_node(app),
    }
}
