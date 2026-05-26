use super::*;

#[test]
fn looks_like_path_detects_absolute_relative_and_home_paths() {
    assert!(looks_like_path("/tmp/file"));
    assert!(looks_like_path("./file"));
    assert!(looks_like_path("~/file"));
    assert!(!looks_like_path("not-a-path"));
}

