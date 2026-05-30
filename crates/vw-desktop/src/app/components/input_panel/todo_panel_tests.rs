#[test]
fn task_740_test_module_is_wired() {
    let path = std::path::Path::new(file!());
    assert!(path.ends_with("todo_panel_tests.rs"));
}

#[test]
fn todo_panel_placement_labels_match_ui_options() {
    assert_eq!(crate::app::TodoPanelPlacement::ChatTopRight.label(), "右上角");
    assert_eq!(crate::app::TodoPanelPlacement::InputBottom.label(), "输入底部");
}
