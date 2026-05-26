use super::json_format::MINDMAP_JSON_FORMAT;

#[test]
fn file_ops_module_exposes_json_format_constant() {
    assert_eq!(MINDMAP_JSON_FORMAT, "vibe-window-mindmap");
}
