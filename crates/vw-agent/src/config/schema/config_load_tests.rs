use super::config_load::{
    config_json_path, load_config_from_json_root, load_config_from_toml_path,
};
use super::workspace::ConfigResolutionSource;

#[test]
fn config_json_path_appends_stable_filename() {
    assert_eq!(
        config_json_path(std::path::Path::new("/tmp/vw")),
        std::path::Path::new("/tmp/vw/vibewindow.json")
    );
}

#[tokio::test]
async fn load_config_from_json_root_returns_none_for_missing_or_empty_file() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("vibewindow.json");

    let loaded = load_config_from_json_root(
        &path,
        tmp.path().join("workspace"),
        tmp.path(),
        ConfigResolutionSource::DefaultConfigDir,
    )
    .await
    .unwrap();

    assert!(loaded.is_none());
}

#[tokio::test]
async fn load_config_from_toml_path_sets_runtime_paths() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("legacy.toml");
    tokio::fs::write(&path, "default_model = \"model\"\ndefault_temperature = 0.7\n")
        .await
        .unwrap();
    let workspace = tmp.path().join("workspace");

    let loaded = load_config_from_toml_path(
        &path,
        workspace.clone(),
        tmp.path(),
        ConfigResolutionSource::DefaultConfigDir,
    )
    .await
    .unwrap();

    assert_eq!(loaded.config_path, path);
    assert_eq!(loaded.workspace_dir, workspace);
    assert_eq!(loaded.default_model.as_deref(), Some("model"));
}
