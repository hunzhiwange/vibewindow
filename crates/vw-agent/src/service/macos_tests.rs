use super::macos::{SERVICE_LABEL, config_dir_arg, macos_service_file};
use crate::app::agent::config::Config;

#[test]
fn service_label_matches_launchd_identifier() {
    assert_eq!(SERVICE_LABEL, "com.vibewindow.daemon");
}

#[test]
fn config_dir_arg_returns_config_parent_without_file_name() {
    let mut config = Config::default();
    config.config_path = "/Users/me/Library/Application Support/VibeWindow/vibewindow.json".into();

    let dir = config_dir_arg(&config).unwrap();

    assert_eq!(dir, "/Users/me/Library/Application Support/VibeWindow");
    assert!(!dir.contains("vibewindow.json"));
}

#[test]
fn config_dir_arg_returns_none_when_config_path_has_no_parent() {
    let mut config = Config::default();
    config.config_path = "/".into();

    assert_eq!(config_dir_arg(&config), None);
}

#[test]
fn macos_service_file_uses_launch_agents_label() {
    let file = macos_service_file().unwrap();

    assert!(file.ends_with("Library/LaunchAgents/com.vibewindow.daemon.plist"));
}
