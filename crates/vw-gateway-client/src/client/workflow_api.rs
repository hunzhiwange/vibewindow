//! Workflow 网关客户端。

use vw_api_types::workflow::{WorkflowRunRequest, WorkflowRunResponse};

use super::GatewayClient;

impl GatewayClient {
    /// 执行 Dify Workflow。
    pub async fn workflow_run(
        &self,
        body: &WorkflowRunRequest,
    ) -> Result<WorkflowRunResponse, String> {
        self.post_json("/v1/workflow/run", &[], body).await
    }
}

#[cfg(test)]
#[path = "workflow_api_tests.rs"]
mod workflow_api_tests;
