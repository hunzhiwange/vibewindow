use super::path::{
    normalize_file_reference_to_path, normalize_file_url_to_path, relative_to_project_root,
    resolve_path,
};
use crate::app::App;

#[test]
fn normalize_file_references_strip_wrappers_and_line_suffixes() {
    assert_eq!(normalize_file_url_to_path("file:///tmp/a.rs"), "tmp/a.rs");
    assert_eq!(
        normalize_file_reference_to_path("[main](file:///tmp/a.rs#L10)"),
        Some("tmp/a.rs".to_string())
    );
    assert_eq!(
        normalize_file_reference_to_path("`'/tmp/demo.rs#line-12'`"),
        Some("/tmp/demo.rs".to_string())
    );
    assert_eq!(normalize_file_reference_to_path(""), None);
}

#[test]
fn resolve_and_relativize_paths_against_project_root() {
    let mut app = App::new().0;
    app.project_path = Some("/tmp/project".to_string());

    assert_eq!(resolve_path(&app, "src/main.rs").as_deref(), Some("/tmp/project/src/main.rs"));
    assert_eq!(
        relative_to_project_root(&app, "/tmp/project/src/main.rs").as_deref(),
        Some("src/main.rs")
    );
    assert_eq!(resolve_path(&app, "/tmp/absolute.rs").as_deref(), Some("/tmp/absolute.rs"));
}
