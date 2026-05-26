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
fn queue_paths_are_scoped_under_vibewindow_home() {
    let home = Path::new("/tmp/home");
    let key = queue_key_for_session("session-1");

    assert_eq!(queue_base_dir(home), PathBuf::from("/tmp/home/.vibewindow/acp/queues"));
    assert_eq!(
        queue_lock_file_path("session-1", home),
        PathBuf::from(format!("/tmp/home/.vibewindow/acp/queues/{key}.lock"))
    );

    if !cfg!(windows) {
        assert!(queue_socket_path("session-1", home).ends_with(format!("{key}.sock")));
    }
}
