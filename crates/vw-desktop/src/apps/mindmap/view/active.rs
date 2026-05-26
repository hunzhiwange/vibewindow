//! 思维导图活跃视图模块
//!
//! 该模块负责渲染思维导图的活跃编辑界面，包括画布、工具栏、菜单和各种覆盖层。
//! 它是用户与思维导图进行交互的主要界面组件。
//!
//! # 子模块
//!
//! - [`previews`]: 提供各种样式和布局格式的预览组件
//! - [`render`]: 包含主要的渲染逻辑和UI组件构建

mod previews;
mod render;

#[cfg(test)]
#[path = "active_tests.rs"]
mod active_tests;
#[cfg(test)]
mod render_tests;

use crate::app::Message;
use crate::apps::mindmap::state::MindMapTab;
use iced::Element;

/// 渲染思维导图的活跃编辑视图
///
/// 该函数是活跃视图的入口点，负责将思维导图的当前状态转换为可视化的UI元素。
/// 它委托给内部的 `render` 模块来完成实际的渲染工作。
///
/// # 参数
///
/// * `tab` - 思维导图标签页的状态，包含文档、选择状态、UI状态等信息
///
/// # 返回值
///
/// 返回一个 Iced 元素，包含完整的活跃视图UI，包括画布、工具栏、菜单和覆盖层
///
/// # 示例
///
/// ```ignore
/// let tab = MindMapTab::new();
/// let element = render(&tab);
/// // element 可以被添加到 Iced 应用程序的界面中
/// ```
pub(super) fn render(tab: &MindMapTab) -> Element<'_, Message> {
    render::render(tab)
}
