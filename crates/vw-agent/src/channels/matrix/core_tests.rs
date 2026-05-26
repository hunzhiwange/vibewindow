#[test]
fn test_module_is_linked() {
    let module = module_path!();
    assert!(module.contains("tests"));
}
