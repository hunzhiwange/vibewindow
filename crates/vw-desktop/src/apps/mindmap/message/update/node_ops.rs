//! 节点操作模块。
//!
//! 该模块保留 `node_ops::*` 作为统一入口，具体实现按职责拆分到独立文件：
//! - `node_ops_text`: 节点文本编辑
//! - `node_ops_history`: 撤销重做
//! - `node_ops_structure`: 节点结构、折叠、上下文菜单
//! - `node_ops_clipboard`: 复制、剪切、粘贴、复制节点

pub(super) use super::node_ops_clipboard::{
    copy_node, cut_node, delete_node, duplicate_node, paste_node,
};
pub(super) use super::node_ops_history::{redo_node, undo_node};
pub(super) use super::node_ops_structure::{
    add_child, add_child_at, add_sibling, add_sibling_at, close_context_menu, open_context_menu,
    toggle_collapse_at,
};
pub(super) use super::node_ops_text::{
    commit_text_editor_if_needed, node_text_changed, node_text_editor_action,
    node_text_editor_enter, save_node_text, toggle_node_text_editor,
};
