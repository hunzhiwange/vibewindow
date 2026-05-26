use super::open_save::is_json_path;

#[test]
fn is_json_path_matches_extension_case_insensitively() {
    assert!(is_json_path("/tmp/map.JSON"));
    assert!(!is_json_path("/tmp/map.md"));
    assert!(!is_json_path("/tmp/json"));
}
