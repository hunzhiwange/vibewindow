//! 处理聊天输入区的局部消息。
//! 本模块将编辑器操作、文件检索和工具细节限制在输入面板边界内。

use super::file_search::{DroppedPath, format_drop_mentions};

#[test]
fn format_drop_mentions_appends_trailing_slash_for_directories() {
    let mentions = format_drop_mentions(
        Some("/workspace"),
        &[DroppedPath { path: "/workspace/src/components".to_string(), is_dir: true }],
        None,
    );

    assert_eq!(mentions, vec!["src/components/".to_string()]);
}

#[test]
fn format_drop_mentions_keeps_single_file_position() {
    let mentions = format_drop_mentions(
        Some("/workspace"),
        &[DroppedPath { path: "/workspace/src/main.rs".to_string(), is_dir: false }],
        Some((12, 4)),
    );

    assert_eq!(mentions, vec!["src/main.rs:12:4".to_string()]);
}

#[test]
fn format_drop_mentions_ignores_position_for_multiple_paths() {
    let mentions = format_drop_mentions(
        Some("/workspace"),
        &[
            DroppedPath { path: "/workspace/src/main.rs".to_string(), is_dir: false },
            DroppedPath { path: "/workspace/src/lib.rs".to_string(), is_dir: false },
        ],
        Some((12, 4)),
    );

    assert_eq!(mentions, vec!["src/main.rs".to_string(), "src/lib.rs".to_string()]);
}