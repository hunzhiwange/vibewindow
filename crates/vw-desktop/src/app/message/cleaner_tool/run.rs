//! 通过 gateway 承接系统清理工具的执行阶段。

use super::types::CleanerCleanupRequest;

pub(super) async fn cleaner_info() -> Result<vw_gateway_client::CleanerInfoResponse, String> {
    let client = crate::app::config::gateway_client()?;
    client.cleaner_info().await
}

pub(super) async fn execute_cleanup(request: CleanerCleanupRequest) -> Result<String, String> {
    let client = crate::app::config::gateway_client()?;
    client.cleaner_run(&request).await
}

pub(super) async fn cancel_cleanup() -> Result<(), String> {
    let client = crate::app::config::gateway_client()?;
    client.cleaner_cancel().await
}

pub(super) async fn cleanup_status() -> Result<super::types::CleanerStatusResponse, String> {
    let client = crate::app::config::gateway_client()?;
    client.cleaner_status().await
}

#[cfg(test)]
#[path = "run_tests.rs"]
mod run_tests;
