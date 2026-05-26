use super::*;
use std::io;
use tokio::time::{Duration, sleep};

#[derive(Debug, thiserror::Error)]
#[error("inner failed")]
struct InnerError;

#[tokio::test]
async fn with_timeout_returns_value_when_timeout_is_missing_or_zero() {
    assert_eq!(with_timeout(async { 7 }, None).await.unwrap(), 7);
    assert_eq!(with_timeout(async { 8 }, Some(0)).await.unwrap(), 8);
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
