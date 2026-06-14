use super::*;

#[test]
fn looks_like_path_detects_absolute_relative_and_home_paths() {
    assert!(looks_like_path("/tmp/file"));
    assert!(looks_like_path("./file"));
    assert!(looks_like_path("~/file"));
    assert!(looks_like_path("../file"));
    assert!(looks_like_path("."));
    assert!(looks_like_path(".."));
    assert!(looks_like_path("src/main.rs"));
    assert!(!looks_like_path("not-a-path"));
}

#[test]
fn home_dir_and_expand_user_path_follow_home_environment() {
    let home = home_dir().expect("HOME should be available");

    assert_eq!(expand_user_path("~"), home);
    assert_eq!(expand_user_path("~/project"), home.join("project"));
    assert_eq!(expand_user_path("~other/project"), std::path::PathBuf::from("~other/project"));
    assert_eq!(expand_user_path("/tmp/project"), std::path::PathBuf::from("/tmp/project"));
}
