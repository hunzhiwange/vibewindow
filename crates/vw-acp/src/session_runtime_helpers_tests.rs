use super::*;
use std::io;
#[cfg(unix)]
use std::sync::Arc;
#[cfg(unix)]
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::time::{Duration, sleep};

#[cfg(unix)]
static SIGNAL_TEST_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

#[derive(Debug, thiserror::Error)]
#[error("inner failed")]
struct InnerError;

#[tokio::test]
async fn with_timeout_returns_value_when_timeout_is_missing_or_zero() {
    assert_eq!(with_timeout(async { 7 }, None).await.unwrap(), 7);
    assert_eq!(with_timeout(async { 8 }, Some(0)).await.unwrap(), 8);
}

#[tokio::test]
async fn with_timeout_returns_value_before_configured_deadline() {
    let value = with_timeout(
        async {
            sleep(Duration::from_millis(1)).await;
            9
        },
        Some(100),
    )
    .await
    .unwrap();

    assert_eq!(value, 9);
}

#[tokio::test]
async fn with_timeout_reports_configured_timeout() {
    let error = with_timeout(
        async {
            sleep(Duration::from_millis(50)).await;
            1
        },
        Some(1),
    )
    .await
    .unwrap_err();

    assert_eq!(error, TimeoutError { timeout_ms: 1 });
    assert_eq!(error.to_string(), "Timed out after 1ms");
}

#[tokio::test]
async fn with_interrupt_returns_inner_result_when_run_finishes_first() {
    let result = with_interrupt(
        || async { Ok::<_, InnerError>(9) },
        || async {
            panic!("interrupt handler should not run");
        },
    )
    .await
    .unwrap();

    assert_eq!(result, 9);
}

#[cfg(unix)]
#[tokio::test]
async fn with_interrupt_runs_handler_and_returns_interrupted_on_signal() {
    let _guard = SIGNAL_TEST_LOCK.lock().await;
    let handler_called = Arc::new(AtomicBool::new(false));
    let handler_called_for_callback = handler_called.clone();

    let error = with_interrupt(
        || async {
            sleep(Duration::from_millis(20)).await;
            assert_eq!(unsafe { libc::raise(libc::SIGHUP) }, 0);
            sleep(Duration::from_secs(5)).await;
            Ok::<_, InnerError>(())
        },
        || {
            let handler_called = handler_called_for_callback.clone();
            async move {
                handler_called.store(true, Ordering::SeqCst);
            }
        },
    )
    .await
    .unwrap_err();

    assert!(matches!(error, WithInterruptError::Interrupted(_)));
    assert!(handler_called.load(Ordering::SeqCst));
}

#[test]
fn with_interrupt_error_display_preserves_variants() {
    let inner: WithInterruptError<InnerError> = WithInterruptError::Inner(InnerError);
    let signal: WithInterruptError<InnerError> =
        WithInterruptError::SignalHandler(io::Error::other("signal"));

    assert_eq!(inner.to_string(), "inner failed");
    assert_eq!(
        WithInterruptError::<InnerError>::Interrupted(InterruptedError).to_string(),
        "Interrupted"
    );
    assert!(signal.to_string().contains("failed to install signal handler"));
}
