//! 设计器设置视图模块，负责设置面板、快捷键说明与缩放控制的界面组合。
//!
//! 本模块只负责视图组合与样式适配，不持有业务状态，也不扩大外部能力边界。

mod panel;
mod shortcuts;
mod zoom_controls;

/// 重新导出 panel::render_settings_panel，作为上层模块访问该视图能力的稳定入口。
pub use panel::render_settings_panel;
/// 重新导出 shortcuts::render_shortcuts_panel，作为上层模块访问该视图能力的稳定入口。
pub use shortcuts::render_shortcuts_panel;
/// 重新导出 zoom_controls::render_zoom_controls，作为上层模块访问该视图能力的稳定入口。
pub use zoom_controls::render_zoom_controls;
