#[test]
fn permission_modal_tests_module_is_wired() {
    assert!(module_path!().ends_with("permission_modal_tests"));
}
