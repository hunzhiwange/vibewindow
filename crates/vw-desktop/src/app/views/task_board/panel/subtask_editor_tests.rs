#[test]
fn test_module_is_wired() {
    let module = std::hint::black_box(module_path!());

    assert!(module.ends_with("subtask_editor_tests"));
}

#[test]
fn subtask_scrollbar_constants_match_panel_contract() {
    assert_eq!(super::SUBTASK_SCROLLBAR_WIDTH, 4.0);
    assert!(super::SUBTASK_LIST_MAX_HEIGHT > 0.0);
    assert!(super::SUBTASK_SCROLLBAR_GUTTER >= super::SUBTASK_SCROLLBAR_WIDTH);
}
