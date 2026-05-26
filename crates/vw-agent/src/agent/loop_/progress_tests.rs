use serde_json::json;

use super::progress::{
    tool_progress_actions, tool_progress_label, truncate_tool_args_for_progress,
};

#[test]
fn truncate_tool_args_uses_tool_specific_fields() {
    assert_eq!(
        truncate_tool_args_for_progress("shell", &json!({"command": "echo hello"}), 30),
        "echo hello"
    );
    assert_eq!(
        truncate_tool_args_for_progress("file_edit", &json!({"filePath": "src/main.rs"}), 30),
        "src/main.rs"
    );
}

#[test]
fn progress_label_includes_hint_when_available() {
    let args = json!({"command": "cargo clippy --all-targets"});

    assert!(tool_progress_label("shell", &args).contains("cargo clippy"));
    assert_eq!(tool_progress_actions("file_read"), ("读取中", "执行完毕"));
}
