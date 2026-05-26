#[test]
fn tests_module_is_wired() {
    assert!(module_path!().ends_with("tests"));
}
