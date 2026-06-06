//! Workflow 网关客户端。

use vw_api_types::workflow::{
    WorkflowRecord, WorkflowRecordDeleteResponse, WorkflowRecordSummary, WorkflowRecordUpsertBody,
};

use super::GatewayClient;

impl GatewayClient {
    /// 列出本地保存的 Dify Workflow。
    pub async fn workflow_applications_list(&self) -> Result<Vec<WorkflowRecordSummary>, String> {
        self.get_json("/v1/workflow/applications", &[]).await
    }

    /// 读取本地保存的 Dify Workflow。
    pub async fn workflow_application_get(&self, uuid: &str) -> Result<WorkflowRecord, String> {
        self.get_json(&format!("/v1/workflow/applications/{uuid}"), &[]).await
    }

    /// 新增本地 Dify Workflow。
    pub async fn workflow_application_create(
        &self,
        body: &WorkflowRecordUpsertBody,
    ) -> Result<WorkflowRecord, String> {
        self.post_json("/v1/workflow/applications", &[], body).await
    }

    /// 更新本地 Dify Workflow。
    pub async fn workflow_application_update(
        &self,
        uuid: &str,
        body: &WorkflowRecordUpsertBody,
    ) -> Result<WorkflowRecord, String> {
        self.put_json(&format!("/v1/workflow/applications/{uuid}"), &[], body).await
    }

    /// 删除本地 Dify Workflow。
    pub async fn workflow_application_delete(
        &self,
        uuid: &str,
    ) -> Result<WorkflowRecordDeleteResponse, String> {
        self.delete_json(&format!("/v1/workflow/applications/{uuid}"), &[], &serde_json::json!({}))
            .await
    }
}

#[cfg(test)]
#[path = "workflow_api_tests.rs"]
mod workflow_api_tests;
