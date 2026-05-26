use super::*;

#[test]
fn writable_dir_probe_creates_and_cleans_probe_file() {
    let dir = tempfile::tempdir().expect("temp dir");
    check_writable_dir(dir.path()).expect("temp dir should be writable");

    let leftover = std::fs::read_dir(dir.path())
        .unwrap()
        .filter_map(Result::ok)
        .any(|entry| entry.file_name().to_string_lossy().starts_with(".vibewindow-capability-probe"));
    assert!(!leftover);
}
