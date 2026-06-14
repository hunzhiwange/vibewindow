#[test]
fn task_611_test_module_is_wired() {
    let path = std::path::Path::new(file!());
    assert!(path.ends_with("main_tests.rs"));
}

#[test]
fn binary_main_is_tokio_entry_source() {
    let source = include_str!("main.rs");

    assert!(source.contains("#[tokio::main]"));
    assert!(source.contains("vw_cli::run().await"));
}
