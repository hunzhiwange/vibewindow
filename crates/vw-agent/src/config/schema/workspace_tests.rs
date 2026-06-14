use super::workspace::{
    ConfigResolutionSource, clear_active_workspace_marker, default_config_dir,
    persist_active_workspace_marker, resolve_config_dir_for_workspace, resolve_runtime_config_dirs,
};

#[test]
fn workspace_config_resolution_returns_config_and_workspace_dirs() {
    let workspace = std::path::Path::new("/tmp/vw-workspace");
    let (config_dir, workspace_dir) = resolve_config_dir_for_workspace(workspace);

    assert_eq!(config_dir, workspace);
    assert_eq!(workspace_dir, workspace.join("workspace"));
}

#[test]
fn workspace_resolution_detects_config_files_and_legacy_workspace_layout() {
    let tmp = tempfile::tempdir().unwrap();
    let config_dir = tmp.path().join("project");
    std::fs::create_dir_all(&config_dir).unwrap();
    std::fs::write(config_dir.join("vibewindow.json"), "{}").unwrap();

    let (resolved_config, resolved_workspace) = resolve_config_dir_for_workspace(&config_dir);
    assert_eq!(resolved_config, config_dir);
    assert_eq!(resolved_workspace, resolved_config.join("workspace"));

    let workspace = tmp.path().join("workspace");
    let legacy = tmp.path().join(".vibewindow");
    std::fs::create_dir_all(&workspace).unwrap();
    std::fs::create_dir_all(&legacy).unwrap();
    std::fs::write(legacy.join("vibewindow.json"), "{}").unwrap();

    let (resolved_config, resolved_workspace) = resolve_config_dir_for_workspace(&workspace);
    assert_eq!(resolved_config, legacy);
    assert_eq!(resolved_workspace, workspace);
}

#[test]
fn default_config_dir_can_be_resolved() {
    assert!(default_config_dir().is_ok());
}

#[test]
fn config_resolution_source_strings_are_stable() {
    assert_eq!(ConfigResolutionSource::EnvConfigDir.as_str(), "VIBEWINDOW_CONFIG_DIR");
    assert_eq!(ConfigResolutionSource::EnvWorkspace.as_str(), "VIBEWINDOW_WORKSPACE");
    assert_eq!(ConfigResolutionSource::ActiveWorkspaceMarker.as_str(), "active_workspace.toml");
    assert_eq!(ConfigResolutionSource::DefaultConfigDir.as_str(), "default");
}

#[tokio::test]
async fn active_workspace_marker_round_trip_and_runtime_resolution_priority() {
    let tmp = tempfile::tempdir().unwrap();
    let default_config = tmp.path().join("default-config");
    let default_workspace = default_config.join("workspace");
    let selected_config = tmp.path().join("selected-config");
    tokio::fs::create_dir_all(&default_config).await.unwrap();
    tokio::fs::create_dir_all(&selected_config).await.unwrap();

    persist_active_workspace_marker(&default_config, &selected_config).await.unwrap();
    let (config_dir, workspace_dir, source) =
        resolve_runtime_config_dirs(&default_config, &default_workspace).await.unwrap();

    assert_eq!(config_dir, selected_config);
    assert_eq!(workspace_dir, config_dir.join("workspace"));
    assert_eq!(source, ConfigResolutionSource::ActiveWorkspaceMarker);

    clear_active_workspace_marker(&default_config).await.unwrap();
    let (_, _, source) =
        resolve_runtime_config_dirs(&default_config, &default_workspace).await.unwrap();
    assert_eq!(source, ConfigResolutionSource::DefaultConfigDir);
}
