#[cfg(not(target_arch = "wasm32"))]
#[test]
fn format_runtime_is_singleton() {
    let first = super::runtime::format_runtime() as *const _;
    let second = super::runtime::format_runtime() as *const _;

    assert_eq!(first, second);
}

#[cfg(target_arch = "wasm32")]
#[test]
fn runtime_test_module_is_loaded_on_wasm() {
    assert!(true);
}
