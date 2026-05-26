#[test]
#[cfg(not(target_arch = "wasm32"))]
fn directory_size_path_returns_zero_for_missing_path() {
    let missing = std::env::temp_dir().join("vibe-window-cleaner-missing-path-for-test");
    assert_eq!(super::directory_size_path(&missing), 0);
}

#[test]
#[cfg(not(target_arch = "wasm32"))]
fn matching_file_size_path_filters_extensions_case_insensitively() {
    let root = std::env::temp_dir().join(format!(
        "vibe-window-cleaner-fs-test-{}",
        std::process::id()
    ));
    let nested = root.join("nested");
    std::fs::create_dir_all(&nested).unwrap();
    std::fs::write(root.join("keep.LOG"), b"1234").unwrap();
    std::fs::write(nested.join("skip.txt"), b"abcdef").unwrap();

    assert_eq!(super::matching_file_size_path(&root, &["log"]), 4);

    std::fs::remove_dir_all(root).unwrap();
}

#[test]
#[cfg(not(target_arch = "wasm32"))]
fn covers_target_treats_directories_as_containing_children() {
    let base = std::path::Path::new("/tmp/root");
    let child = std::path::Path::new("/tmp/root/child/file.log");
    assert!(super::covers_target(
        base,
        super::ScanDetailKind::Directory,
        child,
        super::ScanDetailKind::Directory
    ));
    assert!(!super::covers_target(
        child,
        super::ScanDetailKind::Directory,
        base,
        super::ScanDetailKind::Directory
    ));
}
