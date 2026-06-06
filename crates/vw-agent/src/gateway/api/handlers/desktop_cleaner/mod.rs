//! Gateway-hosted desktop cleaner endpoints.

mod fs;
mod run;
mod scan;

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use axum::Json;
use axum::Router;
use axum::routing::{get, post};
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use vw_api_types::cleaner::{
    CleanerCleanupRequest, CleanerInfoResponse, CleanerRunResponse, CleanerScanReport,
    CleanerStatusResponse,
};
use vw_api_types::common::OperationAck;

use crate::app::agent::gateway::ApiError;

static CLEANER_CANCEL_FLAG: Lazy<Arc<AtomicBool>> = Lazy::new(|| Arc::new(AtomicBool::new(false)));
static CLEANER_RUNNING: AtomicBool = AtomicBool::new(false);
static CLEANER_OUTPUT: Lazy<Arc<Mutex<String>>> = Lazy::new(|| Arc::new(Mutex::new(String::new())));

pub(crate) fn router<S>() -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    Router::new()
        .route("/desktop/cleaner/info", get(info_get))
        .route("/desktop/cleaner/scan", post(scan_post))
        .route("/desktop/cleaner/run", post(run_post))
        .route("/desktop/cleaner/status", get(status_get))
        .route("/desktop/cleaner/cancel", post(cancel_post))
}

async fn info_get() -> Json<CleanerInfoResponse> {
    let platform = host_platform().to_string();
    Json(CleanerInfoResponse {
        supported: matches!(platform.as_str(), "macos" | "windows"),
        platform,
    })
}

async fn scan_post() -> Result<Json<CleanerScanReport>, ApiError> {
    let report = tokio::task::spawn_blocking(scan::scan_cleanup_targets)
        .await
        .map_err(|err| ApiError::internal(err.to_string()))?
        .map_err(ApiError::bad_request)?;
    Ok(Json(report))
}

fn host_platform() -> &'static str {
    #[cfg(target_os = "macos")]
    {
        "macos"
    }

    #[cfg(windows)]
    {
        "windows"
    }

    #[cfg(all(not(target_os = "macos"), not(windows)))]
    {
        "linux"
    }
}

async fn run_post(
    Json(request): Json<CleanerCleanupRequest>,
) -> Result<Json<CleanerRunResponse>, ApiError> {
    CLEANER_CANCEL_FLAG.store(false, Ordering::Relaxed);
    CLEANER_RUNNING.store(true, Ordering::Relaxed);
    {
        let mut output = CLEANER_OUTPUT.lock();
        *output = "清理任务已提交，正在准备执行...".to_string();
    }
    let cancel_flag = CLEANER_CANCEL_FLAG.clone();
    let progress_output = CLEANER_OUTPUT.clone();
    let output = tokio::task::spawn_blocking(move || {
        run::execute_cleanup(request, cancel_flag, progress_output)
    })
    .await
    .map_err(|err| ApiError::internal(err.to_string()))?
    .map_err(|err| {
        CLEANER_RUNNING.store(false, Ordering::Relaxed);
        let mut output = CLEANER_OUTPUT.lock();
        *output = err.clone();
        ApiError::bad_request(err)
    })?;
    CLEANER_RUNNING.store(false, Ordering::Relaxed);
    {
        let mut progress = CLEANER_OUTPUT.lock();
        *progress = output.clone();
    }
    Ok(Json(CleanerRunResponse { output }))
}

async fn status_get() -> Json<CleanerStatusResponse> {
    Json(CleanerStatusResponse {
        running: CLEANER_RUNNING.load(Ordering::Relaxed),
        output: CLEANER_OUTPUT.lock().clone(),
    })
}

async fn cancel_post() -> Json<OperationAck> {
    CLEANER_CANCEL_FLAG.store(true, Ordering::Relaxed);
    if CLEANER_RUNNING.load(Ordering::Relaxed) {
        let mut output = CLEANER_OUTPUT.lock();
        if !output.contains("正在取消清理") {
            if !output.trim().is_empty() {
                output.push_str("\n\n");
            }
            output.push_str("正在取消清理，请稍候...");
        }
    }
    Json(OperationAck { ok: true, message: Some("cleanup cancellation requested".to_string()) })
}

#[cfg(test)]
#[path = "mod_tests.rs"]
mod mod_tests;
