use super::identity::{default_workspace_identity_root, format_file_size, workspace_identity_hint};

#[test]
fn default_workspace_identity_root_uses_agent_key() {
    let config_root = format!("~/{}", vw_config_types::paths::HOME_CONFIG_DIR_NAME);

    assert_eq!(default_workspace_identity_root("main"), format!("{config_root}/workspace"));
    assert_eq!(default_workspace_identity_root("coder"), format!("{config_root}/workspace-coder"));
}

#[test]
fn workspace_identity_hint_prefers_custom_root() {
    assert_eq!(
        workspace_identity_hint(Some("profiles/reviewer"), "reviewer", "AGENTS.md"),
        "编辑身份文件 `AGENTS.md`，保存会直接写回 `profiles/reviewer/AGENTS.md`。"
    );
}

#[test]
fn workspace_identity_hint_uses_default_root_for_main_and_workers() {
    assert_eq!(
        workspace_identity_hint(None, "main", "AGENTS.md"),
        format!(
            "编辑身份文件 `AGENTS.md`，保存会直接写回 `~/{}/workspace/AGENTS.md`。",
            vw_config_types::paths::HOME_CONFIG_DIR_NAME
        )
    );
    assert_eq!(
        workspace_identity_hint(None, "reviewer", "AGENTS.md"),
        format!(
            "编辑身份文件 `AGENTS.md`，保存会直接写回 `~/{}/workspace-reviewer/AGENTS.md`。",
            vw_config_types::paths::HOME_CONFIG_DIR_NAME
        )
    );
}

#[test]
fn format_file_size_uses_human_readable_units() {
    assert_eq!(format_file_size(None), None);
    assert_eq!(format_file_size(Some(512)), Some("512 B".to_string()));
    assert_eq!(format_file_size(Some(1023)), Some("1023 B".to_string()));
    assert_eq!(format_file_size(Some(1024)), Some("1.0 KB".to_string()));
    assert_eq!(format_file_size(Some(2048)), Some("2.0 KB".to_string()));
    assert_eq!(format_file_size(Some(1024 * 1024)), Some("1.0 MB".to_string()));
    assert_eq!(format_file_size(Some(2 * 1024 * 1024 + 512 * 1024)), Some("2.5 MB".to_string()));
}
