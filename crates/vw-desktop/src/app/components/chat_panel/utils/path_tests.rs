use super::path::{normalize_file_reference_to_path, normalize_file_url_to_path};

#[test]
fn normalize_file_references_strip_wrappers_and_line_suffixes() {
    assert_eq!(normalize_file_url_to_path("file:///tmp/a.rs"), "tmp/a.rs");
    assert_eq!(
        normalize_file_reference_to_path("[main](file:///tmp/a.rs#L10)"),
        Some("tmp/a.rs".to_string())
    );
    assert_eq!(normalize_file_reference_to_path(""), None);
}
