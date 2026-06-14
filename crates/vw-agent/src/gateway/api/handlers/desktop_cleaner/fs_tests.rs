use std::path::Path;

use super::super::scan::ScanDetailKind;

fn write_file(path: &Path, content: &[u8]) {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("parent dir should create");
    }
    std::fs::write(path, content).expect("file should write");
}

#[test]
fn directory_size_returns_zero_for_missing_path() {
    let dir = tempfile::tempdir().expect("temp dir");
    let missing = dir.path().join("missing");

    assert_eq!(super::directory_size(&missing.to_string_lossy()), 0);
}

#[test]
fn directory_size_counts_file_and_nested_directory_bytes() {
    let dir = tempfile::tempdir().expect("temp dir");
    let root_file = dir.path().join("root.bin");
    let nested_file = dir.path().join("nested").join("child.bin");
    write_file(&root_file, b"1234");
    write_file(&nested_file, b"abcdef");

    assert_eq!(super::directory_size(&root_file.to_string_lossy()), 4);
    assert_eq!(super::directory_size(&dir.path().to_string_lossy()), 10);
}

#[test]
fn matching_file_size_filters_extensions_recursively_and_case_insensitively() {
    let dir = tempfile::tempdir().expect("temp dir");
    write_file(&dir.path().join("one.LOG"), b"111");
    write_file(&dir.path().join("two.txt"), b"22");
    write_file(&dir.path().join("nested").join("three.log"), b"4444");
    write_file(&dir.path().join("nested").join("skip.tmp"), b"ignored");

    assert_eq!(super::matching_file_size(&dir.path().to_string_lossy(), &["log"]), 7);
    assert_eq!(
        super::matching_file_size(&dir.path().join("two.txt").to_string_lossy(), &["log"]),
        0
    );
    assert_eq!(
        super::matching_file_size(&dir.path().join("two.txt").to_string_lossy(), &["txt"]),
        2
    );
}

#[test]
fn measure_cleanup_target_dispatches_by_detail_kind() {
    let dir = tempfile::tempdir().expect("temp dir");
    write_file(&dir.path().join("cache.bin"), b"12345");
    write_file(&dir.path().join("debug.log"), b"123");

    assert_eq!(super::measure_cleanup_target(dir.path(), ScanDetailKind::Directory), 8);
    assert_eq!(
        super::measure_cleanup_target(dir.path(), ScanDetailKind::FileExtensions(&["log"])),
        3
    );
}

#[test]
fn covers_target_matches_directory_descendants_and_identical_extension_sets() {
    let base = Path::new("/tmp/cache");
    let child = Path::new("/tmp/cache/nested/file.log");
    let sibling = Path::new("/tmp/cache-other/file.log");

    assert!(super::covers_target(
        base,
        ScanDetailKind::Directory,
        child,
        ScanDetailKind::FileExtensions(&["log"])
    ));
    assert!(super::covers_target(base, ScanDetailKind::Directory, base, ScanDetailKind::Directory));
    assert!(!super::covers_target(
        base,
        ScanDetailKind::Directory,
        sibling,
        ScanDetailKind::Directory
    ));
    assert!(super::covers_target(
        base,
        ScanDetailKind::FileExtensions(&["log", "txt"]),
        base,
        ScanDetailKind::FileExtensions(&["log", "txt"])
    ));
    assert!(!super::covers_target(
        base,
        ScanDetailKind::FileExtensions(&["log"]),
        base,
        ScanDetailKind::FileExtensions(&["txt"])
    ));
    assert!(!super::covers_target(
        base,
        ScanDetailKind::FileExtensions(&["log"]),
        child,
        ScanDetailKind::Directory
    ));
}

#[test]
fn expand_env_path_replaces_common_and_percent_style_tokens() {
    let temp_name = format!("VW_FS_TEST_{}", std::process::id());
    unsafe {
        std::env::set_var(&temp_name, "custom-value");
    }

    let expanded = super::expand_env_path(&format!("$HOME/$TMPDIR/${temp_name}/%{temp_name}%"));

    assert!(!expanded.contains("$HOME"));
    assert!(!expanded.contains("$TMPDIR"));
    assert!(expanded.contains("custom-value/custom-value"));
}

#[test]
fn expand_env_path_preserves_unknown_tokens() {
    let expanded = super::expand_env_path("$VIBE_WINDOW_UNKNOWN_TEST_TOKEN/cache");

    assert!(expanded.contains("$VIBE_WINDOW_UNKNOWN_TEST_TOKEN"));
}
