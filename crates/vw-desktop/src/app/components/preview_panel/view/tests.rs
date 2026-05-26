use std::path::Path;

use super::{preview_breadcrumb_segments, relative_segments_from_root};

#[test]
fn relative_segments_from_root_rejects_paths_outside_project() {
    let segments =
        relative_segments_from_root(Path::new("/tmp/other/file.rs"), Path::new("/tmp/project"));

    assert!(segments.is_none());
}

#[test]
fn preview_breadcrumb_segments_keep_file_name_without_project() {
    let segments = preview_breadcrumb_segments(Path::new("notes/today.md"), None);

    assert_eq!(segments, vec!["notes/today.md".to_string()]);
}
