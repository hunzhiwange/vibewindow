use std::sync::atomic::Ordering;

use axum::Json;
use axum::http::StatusCode;
use vw_api_types::cleaner::CleanerCleanupRequest;

use super::*;

fn reset_cleaner_state(output: &str) {
    CLEANER_CANCEL_FLAG.store(false, Ordering::Relaxed);
    CLEANER_RUNNING.store(false, Ordering::Relaxed);
    *CLEANER_OUTPUT.lock() = output.to_string();
}

#[tokio::test]
async fn info_get_reports_current_platform_support() {
    let Json(info) = info_get().await;

    assert_eq!(info.platform, host_platform());
    assert_eq!(info.supported, matches!(host_platform(), "macos" | "windows"));
}

#[tokio::test]
async fn status_get_reflects_global_running_and_output_state() {
    reset_cleaner_state("halfway");
    CLEANER_RUNNING.store(true, Ordering::Relaxed);

    let Json(status) = status_get().await;

    assert!(status.running);
    assert_eq!(status.output, "halfway");
    reset_cleaner_state("");
}

#[tokio::test]
async fn cancel_post_sets_flag_and_acknowledges_even_when_idle() {
    reset_cleaner_state("idle");

    let Json(ack) = cancel_post().await;

    assert!(ack.ok);
    assert_eq!(ack.message.as_deref(), Some("cleanup cancellation requested"));
    assert!(CLEANER_CANCEL_FLAG.load(Ordering::Relaxed));
    assert_eq!(CLEANER_OUTPUT.lock().as_str(), "idle");
    reset_cleaner_state("");
}

#[tokio::test]
async fn cancel_post_appends_progress_message_once_while_running() {
    reset_cleaner_state("正在执行");
    CLEANER_RUNNING.store(true, Ordering::Relaxed);

    let Json(first) = cancel_post().await;
    let after_first = CLEANER_OUTPUT.lock().clone();
    let Json(second) = cancel_post().await;
    let after_second = CLEANER_OUTPUT.lock().clone();

    assert!(first.ok);
    assert!(second.ok);
    assert!(after_first.contains("正在执行\n\n正在取消清理，请稍候..."));
    assert_eq!(after_first, after_second);
    reset_cleaner_state("");
}

#[tokio::test]
async fn run_post_default_request_finishes_or_reports_unsupported_platform() {
    reset_cleaner_state("");

    let result = run_post(Json(CleanerCleanupRequest::default())).await;

    if matches!(host_platform(), "macos" | "windows") {
        let Json(response) = result.expect("default cleanup should finish on supported platforms");
        assert!(response.output.contains("本次预计清理垃圾数据"));
        assert!(!CLEANER_RUNNING.load(Ordering::Relaxed));
        assert_eq!(CLEANER_OUTPUT.lock().as_str(), response.output);
    } else {
        let err = result.expect_err("unsupported platform should fail");
        assert_eq!(err.status, StatusCode::BAD_REQUEST);
        assert!(!CLEANER_RUNNING.load(Ordering::Relaxed));
        assert!(CLEANER_OUTPUT.lock().contains("暂不支持"));
    }

    reset_cleaner_state("");
}

#[tokio::test]
async fn scan_post_reports_unsupported_platform_without_scan_on_linux() {
    if matches!(host_platform(), "macos" | "windows") {
        return;
    }

    let err = scan_post().await.expect_err("linux scan should be unsupported");

    assert_eq!(err.status, StatusCode::BAD_REQUEST);
    assert!(err.to_string().contains("暂不支持"));
}

#[test]
fn cleaner_cancel_flag_defaults_to_false_after_reset() {
    reset_cleaner_state("");

    assert!(!CLEANER_CANCEL_FLAG.load(Ordering::Relaxed));
}

#[test]
fn router_can_be_constructed_for_unit_state() {
    let _router = router::<()>();
}
