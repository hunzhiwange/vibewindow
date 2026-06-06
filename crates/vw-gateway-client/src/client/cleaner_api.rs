use vw_api_types::cleaner::{
    CleanerCleanupRequest, CleanerInfoResponse, CleanerRunResponse, CleanerScanReport,
    CleanerStatusResponse,
};
use vw_api_types::common::OperationAck;

use super::GatewayClient;

impl GatewayClient {
    /// 读取清理工具对应的 gateway 宿主平台信息。
    pub async fn cleaner_info(&self) -> Result<CleanerInfoResponse, String> {
        self.get_json("/v1/desktop/cleaner/info", &[]).await
    }

    /// 扫描当前 gateway 所在主机的可清理项目。
    pub async fn cleaner_scan(&self) -> Result<CleanerScanReport, String> {
        self.post_json("/v1/desktop/cleaner/scan", &[], &serde_json::json!({})).await
    }

    /// 按桌面端勾选项执行系统垃圾清理。
    pub async fn cleaner_run(&self, request: &CleanerCleanupRequest) -> Result<String, String> {
        let response: CleanerRunResponse =
            self.post_json("/v1/desktop/cleaner/run", &[], request).await?;
        Ok(response.output)
    }

    /// 读取当前清理任务的实时日志状态。
    pub async fn cleaner_status(&self) -> Result<CleanerStatusResponse, String> {
        self.get_json("/v1/desktop/cleaner/status", &[]).await
    }

    /// 请求 gateway 取消正在执行的清理流程。
    pub async fn cleaner_cancel(&self) -> Result<(), String> {
        let _: OperationAck =
            self.post_json("/v1/desktop/cleaner/cancel", &[], &serde_json::json!({})).await?;
        Ok(())
    }
}

#[cfg(test)]
#[path = "cleaner_api_tests.rs"]
mod cleaner_api_tests;
