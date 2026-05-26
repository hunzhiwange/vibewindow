//! 面板组件模块
//!
//! 本模块负责组织和管理思维导图编辑器中所有可用的面板组件。
//! 这些面板用于渲染各种编辑工具和配置选项的UI界面。
//!
//! # 面板类型
//!
//! - **背景面板 (background)**: 用于设置画布背景样式
//! - **图表类型面板 (diagram_type)**: 用于选择和切换图表类型
//! - **布局选择器面板 (layout_picker)**: 用于选择不同的布局方式
//! - **画笔面板 (pen)**: 用于配置画笔工具的样式和属性
//! - **主题面板 (theme)**: 用于选择和应用主题样式
//! - **工具工具栏面板 (tool_toolbar)**: 用于显示常用工具按钮

/// 背景配置面板模块
///
/// 提供画布背景样式设置功能的面板组件
mod background;

/// 图表类型选择面板模块
///
/// 提供图表类型选择和切换功能的面板组件
mod diagram_type;

/// 布局选择器面板模块
///
/// 提供不同布局方式选择功能的面板组件
mod layout_picker;

/// 画笔工具配置面板模块
///
/// 提供画笔样式和属性配置功能的面板组件
mod pen;

/// 主题选择面板模块
///
/// 提供主题样式选择和应用功能的面板组件
mod theme;

/// 工具工具栏面板模块
///
/// 提供常用工具按钮显示和操作功能的面板组件
mod tool_toolbar;

#[cfg(test)]
#[path = "background_tests.rs"]
mod background_tests;
#[cfg(test)]
#[path = "diagram_type_tests.rs"]
mod diagram_type_tests;
#[cfg(test)]
#[path = "pen_tests.rs"]
mod pen_tests;
#[cfg(test)]
mod tests;
#[cfg(test)]
#[path = "theme_tests.rs"]
mod theme_tests;
#[cfg(test)]
#[path = "tool_toolbar_tests.rs"]
mod tool_toolbar_tests;

/// 重导出背景面板渲染函数
///
/// 该函数用于渲染背景配置面板的UI界面
pub(super) use background::background_panel;

/// 重导出图表类型面板渲染函数
///
/// 该函数用于渲染图表类型选择面板的UI界面
pub(super) use diagram_type::diagram_type_panel;

/// 重导出画笔面板渲染函数
///
/// 该函数用于渲染画笔工具配置面板的UI界面
pub(super) use pen::pen_panel;

/// 重导出主题面板渲染函数
///
/// 该函数用于渲染主题选择面板的UI界面
pub(super) use theme::theme_panel;

/// 重导出工具工具栏渲染函数
///
/// 该函数用于渲染工具工具栏的UI界面
pub(super) use tool_toolbar::tool_toolbar;
