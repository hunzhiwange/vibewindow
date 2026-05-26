//! 验证 ACP 队列 owner 租约文件的获取、刷新、读取与释放。
//!
//! 租约记录是多进程队列协调的最小共享状态；这些测试固定磁盘记录中的 pid、
//! session、heartbeat 和队列深度，确保刷新逻辑只写入当前进程可证明拥有的字段。

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use vw_acp::{
    QueueOwnerLease, QueueOwnerRecord, read_queue_owner_record, read_queue_owner_status,
    refresh_queue_owner_lease_with_now, release_queue_owner_lease, try_acquire_queue_owner_lease,
};

/// 为每个测试构造独立 home 目录，避免租约文件在并发运行时互相污染。
fn unique_home_dir() -> PathBuf {
    static UNIQUE_COUNTER: AtomicU64 = AtomicU64::new(0);
    let unique = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_nanos();
    let counter = UNIQUE_COUNTER.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!("vw-acp-queue-lease-store-{unique}-{counter}"))
}

/// 租约应能完整经历获取、刷新、状态读取与释放流程。
#[tokio::test]
async fn queue_owner_lease_roundtrip_updates_and_releases() {
    let home_dir = unique_home_dir();
    let session_id = "session-lease-roundtrip";

    let lease = try_acquire_queue_owner_lease(session_id, &home_dir)
        .await
        .expect("lease acquisition should succeed")
        .expect("lease should be acquired");
    assert_eq!(lease.session_id, session_id);

    let initial_record =
        read_queue_owner_record(session_id, &home_dir).await.expect("record should exist");
    let heartbeat = initial_record.created_at.clone();
    assert_eq!(initial_record.session_id, session_id);
    assert_eq!(initial_record.queue_depth, 0);

    refresh_queue_owner_lease_with_now(&lease, 3, || heartbeat.clone())
        .await
        .expect("lease refresh should succeed");

    let refreshed_record =
        read_queue_owner_record(session_id, &home_dir).await.expect("record should still exist");
    assert_eq!(refreshed_record.heartbeat_at, heartbeat);
    assert_eq!(refreshed_record.queue_depth, 3);

    let status =
        read_queue_owner_status(session_id, &home_dir).await.expect("status lookup should succeed");
    assert!(status.is_none());

    release_queue_owner_lease(&lease).await.expect("lease release should succeed");
    assert!(read_queue_owner_record(session_id, &home_dir).await.is_none());

    let _ = std::fs::remove_dir_all(home_dir);
}

/// 刷新租约时 pid 必须来自当前进程，不能信任调用者提供的外部 owner 字段。
#[tokio::test]
async fn refresh_queue_owner_lease_clamps_to_current_process_fields() {
    let home_dir = unique_home_dir();
    let session_id = "session-manual-refresh";
    let lock_path = home_dir.join(".vwacp/queues/manual.lock");
    let socket_path = home_dir.join(".vwacp/queues/manual.sock");
    std::fs::create_dir_all(lock_path.parent().expect("lock path should have parent"))
        .expect("queue dir should be created");

    let lease = QueueOwnerLease {
        session_id: session_id.to_string(),
        lock_path: lock_path.clone(),
        socket_path: socket_path.clone(),
        created_at: "2026-04-03T10:00:00Z".to_string(),
        owner_generation: 42,
    };

    refresh_queue_owner_lease_with_now(&lease, 7, || "2026-04-03T10:01:00Z".to_string())
        .await
        .expect("manual refresh should create the record");

    let payload =
        tokio::fs::read_to_string(&lock_path).await.expect("lease file should be readable");
    let refreshed: QueueOwnerRecord =
        serde_json::from_str(&payload).expect("lease file should contain valid json");
    assert_eq!(refreshed.pid, std::process::id());
    assert_eq!(refreshed.session_id, session_id);
    assert_eq!(refreshed.socket_path, socket_path);
    assert_eq!(refreshed.created_at, "2026-04-03T10:00:00Z");
    assert_eq!(refreshed.heartbeat_at, "2026-04-03T10:01:00Z");
    assert_eq!(refreshed.owner_generation, 42);
    assert_eq!(refreshed.queue_depth, 7);

    let _ = std::fs::remove_dir_all(home_dir);
}
