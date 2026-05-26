use super::brief_view::{compact_attachment_path, format_attachment_size, format_sent_at};

#[test]
fn compact_attachment_path_keeps_file_name_for_long_paths() {
    assert_eq!(compact_attachment_path("/tmp/project/src/main.rs"), "src/main.rs");
}

#[test]
fn format_attachment_size_uses_human_units() {
    assert_eq!(format_attachment_size(512), "512 B");
    assert_eq!(format_attachment_size(2048), "2.0 KB");
}

#[test]
fn format_sent_at_rejects_empty_values() {
    assert_eq!(format_sent_at(""), None);
}
