use super::*;
use tempfile::TempDir;

#[test]
fn home_dir_from_env_prefers_home_and_ignores_empty_values() {
    let old_home = std::env::var_os("HOME");
    let old_userprofile = std::env::var_os("USERPROFILE");

    unsafe {
        std::env::set_var("HOME", "/tmp/vw-home");
        std::env::set_var("USERPROFILE", "/tmp/vw-userprofile");
    }
    assert_eq!(home_dir_from_env().as_deref(), Some(Path::new("/tmp/vw-home")));

    unsafe {
        std::env::set_var("HOME", "");
        std::env::set_var("USERPROFILE", "/tmp/vw-userprofile");
    }
    assert_eq!(home_dir_from_env().as_deref(), Some(Path::new("/tmp/vw-userprofile")));

    unsafe {
        std::env::remove_var("HOME");
        std::env::remove_var("USERPROFILE");
    }
    assert!(home_dir_from_env().is_none());

    unsafe {
        match old_home {
            Some(value) => std::env::set_var("HOME", value),
            None => std::env::remove_var("HOME"),
        }
        match old_userprofile {
            Some(value) => std::env::set_var("USERPROFILE", value),
            None => std::env::remove_var("USERPROFILE"),
        }
    }
}

#[test]
fn writable_dir_probe_creates_nested_directory_and_cleans_probe() {
    let temp = TempDir::new().expect("tempdir should create");
    let nested = temp.path().join("one").join("two");

    check_writable_dir(&nested).expect("temp directory should be writable");

    assert!(nested.is_dir());
    let leftovers =
        std::fs::read_dir(&nested).expect("nested directory should be readable").count();
    assert_eq!(leftovers, 0);
}

#[test]
fn writable_dir_reports_create_errors_for_file_path() {
    let temp = TempDir::new().expect("tempdir should create");
    let file_path = temp.path().join("not-a-dir");
    std::fs::write(&file_path, b"file").expect("file should write");

    let err = check_writable_dir(&file_path).expect_err("file path cannot become a directory");

    assert!(err.contains("无法为能力探测创建目录"));
    assert!(err.contains("not-a-dir"));
}

#[test]
fn loopback_bind_is_available_or_returns_contextual_error() {
    if let Err(err) = check_loopback_bind() {
        assert!(err.contains("回环地址绑定不可用"));
    }
}
