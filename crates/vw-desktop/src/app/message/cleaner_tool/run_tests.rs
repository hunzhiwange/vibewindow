#[test]
fn cleaner_run_module_is_wired() {
    assert!(module_path!().ends_with("run_tests"));
}
