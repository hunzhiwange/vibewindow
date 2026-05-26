#[test]
fn test_module_is_wired() {
    let module = std::hint::black_box(module_path!());

    assert!(module.ends_with("logs_modal_tests"));
}
