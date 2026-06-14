#[cfg(not(target_arch = "wasm32"))]
#[test]
fn format_runtime_is_singleton() {
    let first = super::runtime::format_runtime() as *const _;
    let second = super::runtime::format_runtime() as *const _;

    assert_eq!(first, second);
}

#[cfg(not(target_arch = "wasm32"))]
#[tokio::test]
async fn format_file_without_extension_returns_ok_without_instance_state() {
    assert_eq!(super::runtime::format_file("README").await, Ok(()));
    assert_eq!(super::runtime::format_file("").await, Ok(()));
}

#[cfg(not(target_arch = "wasm32"))]
#[tokio::test]
async fn format_file_with_unknown_extension_returns_ok_in_project_instance() {
    let temp = tempfile::TempDir::new().expect("temp dir");
    let result = crate::app::agent::project::instance::provide(temp.path(), None, || {
        Box::pin(async move { super::runtime::format_file("notes.unknown").await })
    })
    .await
    .expect("instance should be provided");

    assert_eq!(result, Ok(()));
}

#[cfg(not(target_arch = "wasm32"))]
#[tokio::test]
async fn status_returns_builtin_formatter_statuses_in_project_instance() {
    let temp = tempfile::TempDir::new().expect("temp dir");
    let statuses = crate::app::agent::project::instance::provide(temp.path(), None, || {
        Box::pin(async move { super::runtime::status().await })
    })
    .await
    .expect("instance should be provided");

    assert!(statuses.iter().any(|item| item.name == "prettier"));
    assert!(statuses.iter().any(|item| item.name == "rustfmt"));
    assert!(statuses.iter().all(|item| !item.extensions.is_empty()));
}

#[cfg(target_arch = "wasm32")]
#[test]
fn runtime_test_module_is_loaded_on_wasm() {
    assert!(true);
}
