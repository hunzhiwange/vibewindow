use super::tabs::mindmap_app_tab_id;

#[test]
fn mindmap_app_tab_id_keeps_stable_prefix() {
    assert_eq!(mindmap_app_tab_id("mindmap-3"), "mindmap:mindmap-3");
}
