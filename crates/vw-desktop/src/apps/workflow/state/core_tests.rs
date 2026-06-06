#[test]
fn test_module_is_wired() {
    let module = std::hint::black_box(module_path!());

    assert!(module.ends_with("core_tests"));
}

#[test]
fn title_uses_list_title_without_active_app() {
    let state = super::WorkflowState {
        active_app_id: None,
        source_name: "已打开应用".to_string(),
        ..super::WorkflowState::default()
    };

    assert_eq!(state.title(), "工作流");
}
