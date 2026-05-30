use vw_api_types::task::{TaskPoolScheduleRequest, TaskPoolScheduleResponse};

use super::GatewayClient;

impl GatewayClient {
    /// 通过网关获取任务池调度决策。
    pub async fn task_pool_schedule(
        &self,
        request: &TaskPoolScheduleRequest,
    ) -> Result<TaskPoolScheduleResponse, String> {
        self.post_json("/v1/task-pool/schedule", &[], request).await
    }
}

#[cfg(test)]
#[path = "task_pool_api_tests.rs"]
mod task_pool_api_tests;
