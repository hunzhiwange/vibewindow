//! 会话 UI 类型重导出层，向旧调用点暴露共享 crate 中的会话展示模型。

/// 重导出 vw_shared::session::ui_types::*，保持外部调用路径稳定。
pub use vw_shared::session::ui_types::*;
