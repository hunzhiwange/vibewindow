//! 通过 gateway 承接系统清理工具的扫描阶段。

use super::types::CleanerScanReport;

pub(super) async fn scan_cleanup_targets() -> Result<CleanerScanReport, String> {
    let client = crate::app::config::gateway_client()?;
    client.cleaner_scan().await
}

#[cfg(test)]
#[path = "scan_tests.rs"]
mod scan_tests;
