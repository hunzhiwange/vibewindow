#[test]
fn launch_tab_tests_module_is_wired() {
    assert!(module_path!().ends_with("launch_tab_tests"));
}
