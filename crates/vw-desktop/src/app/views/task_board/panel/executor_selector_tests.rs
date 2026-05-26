#[test]
fn test_module_is_wired() {
    let module = std::hint::black_box(module_path!());

    assert!(module.ends_with("executor_selector_tests"));
}
