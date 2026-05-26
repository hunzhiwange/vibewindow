use super::identity::{default_workspace_identity_root, format_file_size, workspace_identity_hint};

#[test]
fn default_workspace_identity_root_uses_agent_key() {
    assert_eq!(default_workspace_identity_root("main"), "~/.vibewindow/workspace");
    assert_eq!(default_workspace_identity_root("coder"), "~/.vibewindow/workspace-coder");
}

#[test]
fn workspace_identity_hint_prefers_custom_root() {
    assert_eq!(
        workspace_identity_hint(Some("profiles/reviewer"), "reviewer", "AGENTS.md"),
        "编辑身份文件 `AGENTS.md`，保存会直接写回 `profiles/reviewer/AGENTS.md`。"
    );
}

#[test]
fn format_file_size_uses_human_readable_units() {
    assert_eq!(format_file_size(None), None);
    assert_eq!(format_file_size(Some(512)), Some("512 B".to_string()));
    assert_eq!(format_file_size(Some(2048)), Some("2.0 KB".to_string()));
}
