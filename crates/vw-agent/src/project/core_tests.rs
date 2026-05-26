use super::*;
use std::path::Path;

#[test]
fn local_project_id_is_deterministic_for_path() {
    let path = Path::new("/tmp/vibe-window-project");
    assert_eq!(local_project_id_from_path(path), local_project_id_from_path(path));
}

#[test]
fn now_ms_returns_millisecond_timestamp() {
    assert!(now_ms() > 1_000_000_000_000);
}

