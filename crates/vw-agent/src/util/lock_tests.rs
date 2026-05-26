use super::lock::{read, write};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

#[tokio::test]
async fn write_waits_for_active_reader() {
    let key = format!("lock-test-{}", std::process::id());
    let reader = read(&key).await;
    let acquired = Arc::new(AtomicBool::new(false));
    let acquired_for_task = acquired.clone();
    let key_for_task = key.clone();

    let task = tokio::spawn(async move {
        let _writer = write(&key_for_task).await;
        acquired_for_task.store(true, Ordering::SeqCst);
    });
    tokio::task::yield_now().await;
    assert!(!acquired.load(Ordering::SeqCst));
    drop(reader);
    task.await.expect("writer task");
    assert!(acquired.load(Ordering::SeqCst));
}
