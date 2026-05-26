use super::abort::{AbortController, AbortSignal, abort_after};
use std::time::Duration;

#[tokio::test]
async fn abort_controller_signals_cancellation() {
    let (controller, mut signal) = AbortController::new();
    assert!(!signal.aborted());
    controller.abort();
    signal.cancelled().await;
    assert!(signal.aborted());
}

#[tokio::test]
async fn any_aborts_when_one_signal_aborts() {
    let (controller, signal) = AbortController::new();
    let mut combined = AbortSignal::any([signal]);
    controller.abort();
    combined.cancelled().await;
    assert!(combined.aborted());
}

#[tokio::test]
async fn clear_timeout_prevents_auto_abort() {
    let mut timeout = abort_after(1);
    timeout.clear_timeout();
    tokio::time::sleep(Duration::from_millis(5)).await;
    assert!(!timeout.signal.aborted());
}
