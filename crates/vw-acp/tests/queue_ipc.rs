//! 验证 ACP 队列 owner 的本机 IPC 连接与健康探测。
//!
//! 这些测试只在 Unix 上覆盖 Unix socket 行为，确保缺失 socket 被视为可恢复状态，
//! 可连接 socket 被识别为健康 owner。队列 owner 负责跨进程排队，健康判断不能
//! 因短暂 socket 竞态而错误提升为致命错误。

use std::time::{SystemTime, UNIX_EPOCH};

#[cfg(unix)]
use std::path::PathBuf;
#[cfg(unix)]
use std::process::{Child, Command};

#[cfg(unix)]
use time::OffsetDateTime;
#[cfg(unix)]
use time::format_description::well_known::Rfc3339;
#[cfg(unix)]
use tokio::fs;
#[cfg(unix)]
use tokio::net::UnixListener;
#[cfg(unix)]
use vw_acp::{
    QueueOwnerRecord, connect_to_queue_owner, default_home_dir, probe_queue_owner_health,
    queue_lock_file_path, queue_socket_path,
};

/// 生成临时 session 与目录后缀，降低并发测试之间的路径冲突概率。
fn unique_suffix() -> String {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_nanos().to_string()
}

/// 返回 RFC3339 时间戳，匹配队列 owner 记录在磁盘上的持久化格式。
#[cfg(unix)]
fn now_iso() -> String {
    OffsetDateTime::now_utc().format(&Rfc3339).expect("current time should format")
}

/// 启动一个短生命周期进程，用于模拟仍存活的队列 owner pid。
#[cfg(unix)]
fn spawn_sleep_process() -> Child {
    Command::new("sh").arg("-c").arg("sleep 5").spawn().expect("sleep process should spawn")
}

/// 清理 lock/socket 文件；测试清理阶段忽略不存在文件，避免掩盖主断言。
#[cfg(unix)]
async fn cleanup_queue_paths(lock_path: &PathBuf, socket_path: &PathBuf) {
    let _ = fs::remove_file(lock_path).await;
    let _ = fs::remove_file(socket_path).await;
}

/// owner socket 可连接时应返回连接对象，证明 IPC 寻址路径可用。
#[cfg(unix)]
#[tokio::test]
async fn connect_to_queue_owner_returns_socket_when_listener_is_available() {
    let session_id = format!("transport-connect-{}", unique_suffix());
    let home_dir = std::env::temp_dir().join(format!("vw-acp-transport-{}", unique_suffix()));
    let socket_path = queue_socket_path(&session_id, &home_dir);
    if let Some(parent) = socket_path.parent() {
        std::fs::create_dir_all(parent).expect("socket parent should exist");
    }

    let listener = UnixListener::bind(&socket_path).expect("listener should bind");
    let accept_task =
        tokio::spawn(async move { listener.accept().await.expect("accept should succeed") });
    let owner = QueueOwnerRecord {
        pid: 4242,
        session_id: session_id.clone(),
        socket_path: socket_path.clone(),
        created_at: now_iso(),
        heartbeat_at: now_iso(),
        owner_generation: 1,
        queue_depth: 0,
    };

    let connection = connect_to_queue_owner(&owner, Some(1))
        .await
        .expect("connect should succeed")
        .expect("connection should be returned");
    drop(connection);

    let _ = accept_task.await.expect("accept task should join");
    let _ = std::fs::remove_file(&socket_path);
    let _ = std::fs::remove_dir_all(home_dir);
}

/// 缺失 socket 常见于 owner 刚退出或尚未创建监听器，应返回 `None` 而非错误。
#[cfg(unix)]
#[tokio::test]
async fn connect_to_queue_owner_retries_missing_socket_and_returns_none() {
    let session_id = format!("transport-missing-{}", unique_suffix());
    let home_dir = std::env::temp_dir().join(format!("vw-acp-transport-{}", unique_suffix()));
    let owner = QueueOwnerRecord {
        pid: 4242,
        session_id,
        socket_path: queue_socket_path("missing", &home_dir),
        created_at: now_iso(),
        heartbeat_at: now_iso(),
        owner_generation: 1,
        queue_depth: 0,
    };

    let connection = connect_to_queue_owner(&owner, Some(1))
        .await
        .expect("missing socket should not surface as an error");
    assert!(connection.is_none());
}

/// 健康探测需要同时确认租约文件、pid 存活和 socket 可达。
#[cfg(unix)]
#[tokio::test]
async fn probe_queue_owner_health_reports_reachable_owner() {
    let home_dir = default_home_dir().expect("default home dir should exist");
    let session_id = format!("health-{}", unique_suffix());
    let lock_path = queue_lock_file_path(&session_id, &home_dir);
    let socket_path = queue_socket_path(&session_id, &home_dir);
    if let Some(parent) = lock_path.parent() {
        std::fs::create_dir_all(parent).expect("lock path parent should exist");
    }
    if let Some(parent) = socket_path.parent() {
        std::fs::create_dir_all(parent).expect("socket path parent should exist");
    }

    let listener = UnixListener::bind(&socket_path).expect("listener should bind");
    let accept_task =
        tokio::spawn(async move { listener.accept().await.expect("accept should succeed") });
    let mut child = spawn_sleep_process();
    let owner = QueueOwnerRecord {
        pid: child.id(),
        session_id: session_id.clone(),
        socket_path: socket_path.clone(),
        created_at: now_iso(),
        heartbeat_at: now_iso(),
        owner_generation: 7,
        queue_depth: 3,
    };
    let payload = serde_json::to_vec_pretty(&owner).expect("owner record should serialize");
    fs::write(&lock_path, [&payload[..], b"\n"].concat())
        .await
        .expect("lock file should be written");

    let health = probe_queue_owner_health(&session_id).await;

    assert!(health.has_lease);
    assert!(health.healthy);
    assert!(health.socket_reachable);
    assert!(health.pid_alive);
    assert_eq!(health.pid, Some(owner.pid));
    assert_eq!(health.socket_path, Some(socket_path.clone()));
    assert_eq!(health.owner_generation, Some(7));
    assert_eq!(health.queue_depth, Some(3));

    let _ = accept_task.await.expect("accept task should join");
    let _ = child.kill();
    let _ = child.wait();
    cleanup_queue_paths(&lock_path, &socket_path).await;
}
