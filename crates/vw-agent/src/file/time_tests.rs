use super::{
    Error, assert as assert_async, assert_sync, get, normalize, read, with_lock, with_lock_sync,
};
use std::fs;
use std::path::PathBuf;
use std::time::Duration;

fn filetime_check_disabled() -> bool {
    *crate::app::agent::flag::VIBEWINDOW_DISABLE_FILETIME_CHECK
}

#[test]
fn read_records_file_time_by_session() {
    read("session-a", "/tmp/example.txt");

    assert!(get("session-a", "/tmp/example.txt").is_some());
    assert!(get("session-b", "/tmp/example.txt").is_none());
}

#[test]
fn display_errors_include_target_path() {
    let err = Error::MustReadFirst { filepath: "/tmp/example.txt".to_string() };

    assert!(err.to_string().contains("/tmp/example.txt"));
    assert_eq!(with_lock_sync("/tmp/example.txt", || 7), 7);
}

#[test]
fn display_covers_all_error_variants_and_from_io() {
    let modified = Error::ModifiedSinceRead { filepath: "/tmp/changed.txt".to_string() };
    let io = Error::Io(std::io::Error::new(std::io::ErrorKind::NotFound, "missing"));
    let from_io: Error = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "denied").into();

    assert!(modified.to_string().contains("/tmp/changed.txt"));
    assert!(io.to_string().contains("missing"));
    assert!(matches!(from_io, Error::Io(_)));
}

#[test]
fn assert_sync_accepts_unmodified_file() {
    if filetime_check_disabled() {
        return;
    }

    let temp = tempfile::tempdir().expect("tempdir");
    let path = temp.path().join("note.txt");
    fs::write(&path, "hello").expect("write note");
    let path_str = path.to_string_lossy().to_string();

    read("time-sync-ok", &path_str);

    assert_sync("time-sync-ok", &path).expect("file was read and not modified");
}

#[test]
fn assert_sync_requires_read_first_and_propagates_metadata_errors() {
    if filetime_check_disabled() {
        return;
    }

    let temp = tempfile::tempdir().expect("tempdir");
    let path = temp.path().join("missing.txt");
    let path_str = path.to_string_lossy().to_string();

    let must_read = assert_sync("time-sync-must-read", &path).expect_err("read first");
    assert!(matches!(must_read, Error::MustReadFirst { .. }));

    read("time-sync-io", &path_str);
    let io = assert_sync("time-sync-io", &path).expect_err("missing file metadata");
    assert!(matches!(io, Error::Io(_)));
}

#[test]
fn assert_sync_detects_modified_file_after_read() {
    if filetime_check_disabled() {
        return;
    }

    let temp = tempfile::tempdir().expect("tempdir");
    let path = temp.path().join("changed.txt");
    fs::write(&path, "before").expect("write before");
    let path_str = path.to_string_lossy().to_string();

    read("time-sync-modified", &path_str);
    std::thread::sleep(Duration::from_millis(20));
    fs::write(&path, "after").expect("write after");

    let err = assert_sync("time-sync-modified", &path).expect_err("modified after read");
    assert!(matches!(err, Error::ModifiedSinceRead { .. }));
}

#[tokio::test]
async fn assert_async_accepts_unmodified_file() {
    if filetime_check_disabled() {
        return;
    }

    let temp = tempfile::tempdir().expect("tempdir");
    let path = temp.path().join("async-note.txt");
    fs::write(&path, "hello").expect("write note");
    let path_str = path.to_string_lossy().to_string();

    read("time-async-ok", &path_str);

    assert_async("time-async-ok", &path).await.expect("file was read and not modified");
}

#[tokio::test]
async fn assert_async_reports_read_metadata_and_modified_errors() {
    if filetime_check_disabled() {
        return;
    }

    let temp = tempfile::tempdir().expect("tempdir");
    let missing = temp.path().join("async-missing.txt");
    let missing_str = missing.to_string_lossy().to_string();

    let must_read = assert_async("time-async-must-read", &missing).await.expect_err("read first");
    assert!(matches!(must_read, Error::MustReadFirst { .. }));

    read("time-async-io", &missing_str);
    let io = assert_async("time-async-io", &missing).await.expect_err("missing file metadata");
    assert!(matches!(io, Error::Io(_)));

    let changed = temp.path().join("async-changed.txt");
    fs::write(&changed, "before").expect("write before");
    let changed_str = changed.to_string_lossy().to_string();
    read("time-async-modified", &changed_str);
    std::thread::sleep(Duration::from_millis(20));
    fs::write(&changed, "after").expect("write after");

    let modified =
        assert_async("time-async-modified", &changed).await.expect_err("modified after read");
    assert!(matches!(modified, Error::ModifiedSinceRead { .. }));
}

#[tokio::test]
async fn with_lock_runs_async_future_and_returns_value() {
    let value = with_lock("/tmp/example-async-lock.txt", async { 11 }).await;

    assert_eq!(value, 11);
}

#[test]
fn normalize_returns_lossy_path_string() {
    let relative = PathBuf::from("relative.txt");
    let absolute = std::env::temp_dir().join("absolute.txt");

    assert_eq!(normalize(&relative), "relative.txt");
    assert_eq!(normalize(&absolute), absolute.to_string_lossy().to_string());
}
