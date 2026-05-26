//! 验证 ACP 队列锁文件与 IPC socket 路径的稳定生成规则。
//!
//! 队列路径同时承担跨进程互斥和本机 IPC 寻址职责，因此测试固定 hash 前缀、
//! home 目录归属和平台差异，避免 session id 或用户目录被直接拼进不安全路径。

use std::path::{Path, PathBuf};

use vw_acp::{
    queue_base_dir, queue_key_for_session, queue_lock_file_path, queue_socket_base_dir,
    queue_socket_path,
};

/// session id 对外只暴露固定长度的 SHA-256 前缀，避免原始 id 泄露到路径名。
#[test]
fn queue_key_for_session_matches_sha256_prefix() {
    assert_eq!(queue_key_for_session("session-123"), "b9c84322f82434cb46e239d2");
}

/// 锁文件位于用户 home 下的 VibeWindow ACP 队列目录，便于随用户数据生命周期清理。
#[test]
fn queue_base_dir_and_lock_file_path_use_home_directory() {
    let home_dir = Path::new("/Users/tester");

    assert_eq!(queue_base_dir(home_dir), PathBuf::from("/Users/tester/.vibewindow/acp/queues"));
    assert_eq!(
        queue_lock_file_path("session-123", home_dir),
        PathBuf::from("/Users/tester/.vibewindow/acp/queues/b9c84322f82434cb46e239d2.lock")
    );
}

/// Unix socket 路径放在 `/tmp` 的 home-hash 子目录中，以规避 socket 路径长度限制。
#[cfg(not(windows))]
#[test]
fn unix_queue_socket_paths_are_hashed_under_tmp() {
    let home_dir = Path::new("/Users/tester");

    assert_eq!(queue_socket_base_dir(home_dir), Some(PathBuf::from("/tmp/vwacp-9b643592ef")));
    assert_eq!(
        queue_socket_path("session-123", home_dir),
        PathBuf::from("/tmp/vwacp-9b643592ef/b9c84322f82434cb46e239d2.sock")
    );
}

/// Windows 使用命名管道，不需要单独的 socket 基础目录。
#[cfg(windows)]
#[test]
fn windows_queue_socket_path_uses_named_pipe() {
    assert_eq!(
        queue_socket_path("session-123", Path::new("C:\\Users\\tester")),
        PathBuf::from(r"\\.\pipe\vwacp-b9c84322f82434cb46e239d2")
    );
    assert_eq!(queue_socket_base_dir(Path::new("C:\\Users\\tester")), None);
}
