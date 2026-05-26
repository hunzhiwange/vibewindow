//! 项目视图组件模块，负责会话列表和项目工具菜单等可复用界面。
//!
//! 本模块只负责视图组合与样式适配，不持有业务状态，也不扩大外部能力边界。

mod new_session;
mod projects_list;
mod sessions_panel;

/// 重新导出 new_session::new_session_picker_layer，作为上层模块访问该视图能力的稳定入口。
pub use new_session::new_session_picker_layer;
/// 重新导出 projects_list::projects_list，作为上层模块访问该视图能力的稳定入口。
pub use projects_list::projects_list;
/// 重新导出 sessions_panel::project_sessions_panel，作为上层模块访问该视图能力的稳定入口。
pub use sessions_panel::project_sessions_panel;
