use super::SqliteKnowledgeStore;
use super::chunker::chunk_text_with_limits;
use crate::memory::embeddings::EmbeddingProvider;
use crate::workflow::{WorkflowKnowledgeProvider, WorkflowKnowledgeRequest};
use async_trait::async_trait;
use serde_json::json;
use std::sync::Arc;
use vw_api_types::knowledge::{
    KnowledgeDatasetCreateRequest, KnowledgeDocumentCreateRequest, KnowledgeIndexingMode,
    KnowledgeRetrievalMode, KnowledgeRetrieveRequest,
};

fn store() -> SqliteKnowledgeStore {
    let dir = tempfile::tempdir().expect("tempdir");
    SqliteKnowledgeStore::new(dir.keep().join("knowledge.sqlite"))
}

fn vector_store() -> SqliteKnowledgeStore {
    let dir = tempfile::tempdir().expect("tempdir");
    SqliteKnowledgeStore::with_embedder(
        dir.keep().join("knowledge.sqlite"),
        Arc::new(FakeEmbedding),
        Some("fake-embedding".to_string()),
        0.8,
        0.2,
        128,
    )
}

struct FakeEmbedding;

#[async_trait]
impl EmbeddingProvider for FakeEmbedding {
    fn name(&self) -> &str {
        "fake"
    }

    fn dimensions(&self) -> usize {
        3
    }

    async fn embed(&self, texts: &[&str]) -> anyhow::Result<Vec<Vec<f32>>> {
        Ok(texts.iter().map(|text| fake_vector(text)).collect())
    }
}

fn fake_vector(text: &str) -> Vec<f32> {
    let text = text.to_ascii_lowercase();
    if text.contains("refund") || text.contains("return") {
        vec![1.0, 0.0, 0.0]
    } else if text.contains("image") || text.contains("category") {
        vec![0.0, 1.0, 0.0]
    } else {
        vec![0.0, 0.0, 1.0]
    }
}

#[test]
fn chunk_text_prefers_sentence_boundary() {
    let chunks = chunk_text_with_limits("alpha beta. gamma delta. epsilon.", 18, 0);

    assert_eq!(chunks.len(), 2);
    assert_eq!(chunks[0].content, "alpha beta.");
}

#[tokio::test]
async fn sqlite_store_retrieves_uploaded_document() {
    let store = store();
    let dataset = store
        .create_dataset(KnowledgeDatasetCreateRequest {
            name: "Support".to_string(),
            description: String::new(),
            indexing_mode: KnowledgeIndexingMode::Economy,
            retrieval_mode: KnowledgeRetrievalMode::FullText,
            embedding_model: None,
            rerank_model: None,
        })
        .await
        .expect("dataset");
    store
        .create_document(
            dataset.id.clone(),
            KnowledgeDocumentCreateRequest {
                name: "Return policy".to_string(),
                content: "Return policy allows refunds within seven days.".to_string(),
                metadata: json!({ "tag": "support" }),
                enabled: true,
            },
        )
        .await
        .expect("document");

    let response = store
        .retrieve(KnowledgeRetrieveRequest {
            query: "refunds".to_string(),
            dataset_ids: vec![dataset.id],
            top_k: 3,
            score_threshold: None,
            metadata_filter: Some(json!({ "tag": "support" })),
        })
        .await
        .expect("retrieve");

    assert_eq!(response.chunks.len(), 1);
    assert!(response.chunks[0].content.contains("refunds"));
    assert_eq!(response.chunks[0].metadata["tag"], json!("support"));
}

#[tokio::test]
async fn workflow_provider_returns_chunks() {
    let store = store();
    let dataset = store
        .create_dataset(KnowledgeDatasetCreateRequest {
            name: "Workflow KB".to_string(),
            description: String::new(),
            indexing_mode: KnowledgeIndexingMode::Economy,
            retrieval_mode: KnowledgeRetrievalMode::FullText,
            embedding_model: None,
            rerank_model: None,
        })
        .await
        .expect("dataset");
    store
        .create_document(
            dataset.id.clone(),
            KnowledgeDocumentCreateRequest {
                name: "Menu setup".to_string(),
                content: "Menu images can be configured on the product category page.".to_string(),
                metadata: json!({ "scope": "menu" }),
                enabled: true,
            },
        )
        .await
        .expect("document");

    let chunks = WorkflowKnowledgeProvider::retrieve(
        &store,
        WorkflowKnowledgeRequest {
            query: "product category".to_string(),
            dataset_ids: vec![dataset.id],
            top_k: 2,
            score_threshold: None,
            metadata_filter: None,
        },
    )
    .await
    .expect("workflow retrieve");

    assert_eq!(chunks.len(), 1);
    assert_eq!(chunks[0].title, "Menu setup");
}

#[tokio::test]
async fn sqlite_store_uses_vector_retrieval_for_high_quality_dataset() {
    let store = vector_store();
    let dataset = store
        .create_dataset(KnowledgeDatasetCreateRequest {
            name: "Vector KB".to_string(),
            description: String::new(),
            indexing_mode: KnowledgeIndexingMode::HighQuality,
            retrieval_mode: KnowledgeRetrievalMode::Vector,
            embedding_model: Some("fake-embedding".to_string()),
            rerank_model: None,
        })
        .await
        .expect("dataset");
    store
        .create_document(
            dataset.id.clone(),
            KnowledgeDocumentCreateRequest {
                name: "Refunds".to_string(),
                content: "Refund requests are handled by the support team.".to_string(),
                metadata: json!({ "kind": "policy" }),
                enabled: true,
            },
        )
        .await
        .expect("refund document");
    store
        .create_document(
            dataset.id.clone(),
            KnowledgeDocumentCreateRequest {
                name: "Images".to_string(),
                content: "Product category images are configured in catalog settings.".to_string(),
                metadata: json!({ "kind": "media" }),
                enabled: true,
            },
        )
        .await
        .expect("image document");

    let response = store
        .retrieve(KnowledgeRetrieveRequest {
            query: "How do returns work?".to_string(),
            dataset_ids: vec![dataset.id],
            top_k: 1,
            score_threshold: None,
            metadata_filter: None,
        })
        .await
        .expect("retrieve");

    assert_eq!(response.chunks.len(), 1);
    assert_eq!(response.chunks[0].title, "Refunds");
    assert!(response.chunks[0].score.unwrap_or_default() > 0.7);
}

#[tokio::test]
async fn unsupported_vector_dataset_is_explicit() {
    let error = store()
        .create_dataset(KnowledgeDatasetCreateRequest {
            name: "Vector".to_string(),
            description: String::new(),
            indexing_mode: KnowledgeIndexingMode::HighQuality,
            retrieval_mode: KnowledgeRetrievalMode::Hybrid,
            embedding_model: Some("text-embedding-v4".to_string()),
            rerank_model: None,
        })
        .await
        .expect_err("unsupported");

    assert!(error.to_string().contains("embedding provider"));
}
