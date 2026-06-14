use super::*;
use std::path::{Path, PathBuf};

#[test]
fn queue_key_for_session_is_stable_short_hash() {
    let key = queue_key_for_session("session-1");

    assert_eq!(key.len(), 24);
    assert_eq!(key, queue_key_for_session("session-1"));
    assert_ne!(key, queue_key_for_session("session-2"));
}

#[test]
fn queue_key_for_empty_session_matches_sha256_prefix() {
    assert_eq!(queue_key_for_session(""), "e3b0c44298fc1c149afbf4c8");
}

#[test]
fn queue_key_for_session_hides_path_like_input() {
    let key = queue_key_for_session("session-with/slashes and spaces");

    assert_eq!(key, "c708ae709a254169332b25d6");
    assert!(!key.contains('/'));
    assert!(!key.contains(' '));
}

#[test]
fn queue_paths_are_scoped_under_vibewindow_home() {
    let home = Path::new("/tmp/home");
    let key = queue_key_for_session("session-1");
    let base = vw_config_types::paths::home_config_dir(home).join("acp").join("queues");

    assert_eq!(queue_base_dir(home), base);
    assert_eq!(queue_lock_file_path("session-1", home), base.join(format!("{key}.lock")));

    if !cfg!(windows) {
        assert!(queue_socket_path("session-1", home).ends_with(format!("{key}.sock")));
    }
}

#[test]
fn queue_base_dir_accepts_relative_home_directory() {
    assert_eq!(
        queue_base_dir(Path::new("relative-home")),
        vw_config_types::paths::home_config_dir("relative-home").join("acp").join("queues")
    );
}

#[cfg(not(windows))]
#[test]
fn unix_queue_socket_base_dir_hashes_home_directory_under_system_temp_dir() {
    assert_eq!(
        queue_socket_base_dir(Path::new("/tmp/home")),
        Some(std::env::temp_dir().join("vwacp-1b49f8aaad"))
    );
}

#[cfg(windows)]
#[test]
fn windows_queue_socket_base_dir_is_not_used() {
    assert_eq!(queue_socket_base_dir(Path::new(r"C:\Users\tester")), None);
}

#[cfg(windows)]
#[test]
fn windows_queue_socket_path_uses_named_pipe() {
    assert_eq!(
        queue_socket_path("session-1", Path::new(r"C:\Users\tester")),
        PathBuf::from(format!(r"\\.\pipe\vwacp-{}", queue_key_for_session("session-1")))
    );
}

#[test]
fn default_home_dir_matches_platform_environment() {
    #[cfg(windows)]
    {
        let expected = std::env::var_os("USERPROFILE").map(PathBuf::from).or_else(|| {
            let home_drive = std::env::var_os("HOMEDRIVE")?;
            let home_path = std::env::var_os("HOMEPATH")?;
            let mut joined = PathBuf::from(home_drive);
            joined.push(home_path);
            Some(joined)
        });

        assert_eq!(default_home_dir(), expected);
    }

    #[cfg(not(windows))]
    {
        assert_eq!(default_home_dir(), std::env::var_os("HOME").map(PathBuf::from));
    }
}

#[test]
fn default_queue_paths_are_derived_from_default_home() {
    let session_id = "session-1";
    let expected_home = default_home_dir();

    assert_eq!(default_queue_base_dir(), expected_home.as_ref().map(queue_base_dir));
    assert_eq!(
        default_queue_lock_file_path(session_id),
        expected_home.as_ref().map(|home_dir| queue_lock_file_path(session_id, home_dir))
    );

    #[cfg(not(windows))]
    {
        assert_eq!(
            default_queue_socket_base_dir(),
            expected_home.as_ref().and_then(queue_socket_base_dir)
        );
        assert_eq!(
            default_queue_socket_path(session_id),
            expected_home.as_ref().map(|home_dir| queue_socket_path(session_id, home_dir))
        );
    }

    #[cfg(windows)]
    {
        assert_eq!(default_queue_socket_base_dir(), None);
        assert_eq!(
            default_queue_socket_path(session_id),
            Some(queue_socket_path(session_id, PathBuf::new()))
        );
    }
}
