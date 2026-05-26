use super::*;

#[test]
fn router_builds_with_unit_state() {
    let _ = router::<()>();
}

#[test]
fn normalize_path_trims_slashes_and_current_directory() {
    assert_eq!(normalize_path("./src/main.rs"), "src/main.rs");
    assert_eq!(normalize_path("/src/main.rs"), "src/main.rs");
}

#[test]
fn validate_repo_relative_path_rejects_escape_segments() {
    assert!(validate_repo_relative_path("src/lib.rs").is_ok());
    assert!(validate_repo_relative_path("../secret").is_err());
    assert!(validate_repo_relative_path("/absolute").is_err());
}
