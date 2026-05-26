//! 验证会话运行时辅助函数的超时与中断语义。
//!
//! 这些测试聚焦异步边界：无超时、零超时、真实超时，以及内部错误在中断包装器中
//! 如何向调用方暴露。

use std::io;
use std::time::Duration;

use vw_acp::{WithInterruptError, with_interrupt, with_timeout};

#[tokio::test]
async fn with_timeout_returns_result_without_timeout_limit() {
    let value = with_timeout(async { 42_u32 }, None).await.expect("future should complete");

    assert_eq!(value, 42);
}

#[tokio::test]
async fn with_timeout_ignores_zero_timeout() {
    let value = with_timeout(async { "ok" }, Some(0))
        .await
        .expect("zero timeout should be treated as disabled");

    assert_eq!(value, "ok");
}

#[tokio::test]
async fn with_timeout_returns_timeout_error_with_duration() {
    let error = with_timeout(
        async {
            tokio::time::sleep(Duration::from_millis(20)).await;
        },
        Some(1),
    )
    .await
    .expect_err("future should time out");

    assert_eq!(error.timeout_ms, 1);
    assert_eq!(error.to_string(), "Timed out after 1ms");
}

#[tokio::test]
async fn with_interrupt_returns_inner_result_when_run_completes() {
    let result = with_interrupt(|| async { Ok::<_, io::Error>("done") }, || async {})
        .await
        .expect("run should complete before interrupt");

    assert_eq!(result, "done");
}

#[tokio::test]
async fn with_interrupt_wraps_inner_error() {
    let error = with_interrupt(|| async { Err::<(), _>(io::Error::other("boom")) }, || async {})
        .await
        .expect_err("inner error should be surfaced");

    match error {
        WithInterruptError::Inner(error) => assert_eq!(error.to_string(), "boom"),
        other => panic!("unexpected error variant: {other}"),
    }
}
