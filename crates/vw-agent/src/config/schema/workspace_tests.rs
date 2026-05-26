use super::workspace::{default_config_dir, resolve_config_dir_for_workspace};

#[test]
fn workspace_config_resolution_returns_config_and_workspace_dirs() {
    let workspace = std::path::Path::new("/tmp/vw-workspace");
    let (config_dir, workspace_dir) = resolve_config_dir_for_workspace(workspace);

    assert_eq!(config_dir, workspace);
    assert_eq!(workspace_dir, workspace.join("workspace"));
}

#[test]
fn default_config_dir_can_be_resolved() {
    assert!(default_config_dir().is_ok());
}
