use super::*;
use std::path::Path;

#[test]
fn server_config_detects_common_extensions() {
    let rust = server_config_for_path(Path::new("src/main.rs")).unwrap();
    assert_eq!(rust.language_id, "rust");
    let ts = server_config_for_path(Path::new("app.ts")).unwrap();
    assert_eq!(ts.language_id, "typescript");
}

#[test]
fn server_config_rejects_unknown_extensions() {
    assert!(server_config_for_path(Path::new("README.unknown")).is_none());
}
