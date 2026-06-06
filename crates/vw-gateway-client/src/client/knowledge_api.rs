//! Knowledge base gateway client.

use vw_api_types::knowledge::{
    KnowledgeDatasetCreateRequest, KnowledgeDatasetDto, KnowledgeDocumentCreateRequest,
    KnowledgeDocumentDto, KnowledgeRetrieveRequest, KnowledgeRetrieveResponse,
    KnowledgeRuntimeStatus,
};

use super::GatewayClient;

impl GatewayClient {
    pub async fn knowledge_status(&self) -> Result<KnowledgeRuntimeStatus, String> {
        self.get_json("/v1/knowledge/status", &[]).await
    }

    pub async fn knowledge_datasets_list(&self) -> Result<Vec<KnowledgeDatasetDto>, String> {
        self.get_json("/v1/knowledge/datasets", &[]).await
    }

    pub async fn knowledge_dataset_get(
        &self,
        dataset_id: &str,
    ) -> Result<KnowledgeDatasetDto, String> {
        self.get_json(&format!("/v1/knowledge/datasets/{dataset_id}"), &[]).await
    }

    pub async fn knowledge_dataset_create(
        &self,
        body: &KnowledgeDatasetCreateRequest,
    ) -> Result<KnowledgeDatasetDto, String> {
        self.post_json("/v1/knowledge/datasets", &[], body).await
    }

    pub async fn knowledge_dataset_delete(
        &self,
        dataset_id: &str,
    ) -> Result<KnowledgeDatasetDto, String> {
        self.delete_json(
            &format!("/v1/knowledge/datasets/{dataset_id}"),
            &[],
            &serde_json::json!({}),
        )
        .await
    }

    pub async fn knowledge_documents_list(
        &self,
        dataset_id: &str,
    ) -> Result<Vec<KnowledgeDocumentDto>, String> {
        self.get_json(&format!("/v1/knowledge/datasets/{dataset_id}/documents"), &[]).await
    }

    pub async fn knowledge_document_create(
        &self,
        dataset_id: &str,
        body: &KnowledgeDocumentCreateRequest,
    ) -> Result<KnowledgeDocumentDto, String> {
        self.post_json(&format!("/v1/knowledge/datasets/{dataset_id}/documents"), &[], body).await
    }

    pub async fn knowledge_document_delete(
        &self,
        document_id: &str,
    ) -> Result<KnowledgeDocumentDto, String> {
        self.delete_json(
            &format!("/v1/knowledge/documents/{document_id}"),
            &[],
            &serde_json::json!({}),
        )
        .await
    }

    pub async fn knowledge_retrieve(
        &self,
        body: &KnowledgeRetrieveRequest,
    ) -> Result<KnowledgeRetrieveResponse, String> {
        self.post_json("/v1/knowledge/retrieve", &[], body).await
    }
}

#[cfg(test)]
#[path = "knowledge_api_tests.rs"]
mod knowledge_api_tests;
