use serde_json::json;
use vw_api_types::knowledge::{
    KnowledgeChunkingMode, KnowledgeDatasetCreateRequest, KnowledgeDocumentCreateRequest,
    KnowledgeIndexingMode, KnowledgeRetrievalMode, KnowledgeRetrieveRequest,
};

use crate::client::test_support;

fn dataset() -> serde_json::Value {
    json!({
        "id": "ds",
        "name": "Docs",
        "description": "",
        "document_count": 1,
        "chunk_count": 2,
        "created_at_ms": 1,
        "updated_at_ms": 2
    })
}

fn document() -> serde_json::Value {
    json!({
        "id": "doc",
        "dataset_id": "ds",
        "name": "Readme",
        "metadata": {},
        "enabled": true,
        "chunk_count": 2,
        "created_at_ms": 1,
        "updated_at_ms": 2
    })
}

#[tokio::test]
async fn knowledge_api_routes_all_dataset_document_and_retrieval_calls() {
    let server = test_support::server(vec![
        (
            200,
            json!({"full_text": true, "vector": false, "hybrid": false, "rerank": false, "notes": {}}),
        ),
        (200, json!([dataset()])),
        (200, dataset()),
        (200, dataset()),
        (200, dataset()),
        (200, json!([document()])),
        (200, document()),
        (200, document()),
        (
            200,
            json!({"chunks": [{
                "id": "chunk",
                "dataset_id": "ds",
                "document_id": "doc",
                "title": "Readme",
                "content": "hello",
                "metadata": {},
                "score": 0.9
            }]}),
        ),
    ]);

    assert!(server.client().knowledge_status().await.expect("status").full_text);
    assert_eq!(server.client().knowledge_datasets_list().await.expect("datasets").len(), 1);
    assert_eq!(server.client().knowledge_dataset_get("ds").await.expect("dataset").id, "ds");
    assert_eq!(
        server
            .client()
            .knowledge_dataset_create(&KnowledgeDatasetCreateRequest {
                name: "Docs".to_string(),
                description: String::new(),
                chunking_mode: KnowledgeChunkingMode::General,
                indexing_mode: KnowledgeIndexingMode::Economy,
                retrieval_mode: KnowledgeRetrievalMode::FullText,
                keyword_count: 10,
                top_k: 5,
                score_threshold_enabled: false,
                score_threshold: 0.15,
                rerank_enabled: false,
                embedding_model: None,
                rerank_model: None,
            })
            .await
            .expect("create")
            .name,
        "Docs"
    );
    assert_eq!(server.client().knowledge_dataset_delete("ds").await.expect("delete").id, "ds");
    assert_eq!(server.client().knowledge_documents_list("ds").await.expect("docs").len(), 1);
    assert_eq!(
        server
            .client()
            .knowledge_document_create(
                "ds",
                &KnowledgeDocumentCreateRequest {
                    name: "Readme".to_string(),
                    content: "hello".to_string(),
                    metadata: json!({}),
                    enabled: true,
                },
            )
            .await
            .expect("doc create")
            .id,
        "doc"
    );
    assert_eq!(
        server.client().knowledge_document_delete("doc").await.expect("doc delete").id,
        "doc"
    );
    assert_eq!(
        server
            .client()
            .knowledge_retrieve(&KnowledgeRetrieveRequest {
                query: "hello".to_string(),
                dataset_ids: vec!["ds".to_string()],
                top_k: 5,
                score_threshold: Some(0.1),
                metadata_filter: None,
            })
            .await
            .expect("retrieve")
            .chunks[0]
            .id,
        "chunk"
    );

    assert_eq!(server.take_request().path, "/v1/knowledge/status");
    assert_eq!(server.take_request().path, "/v1/knowledge/datasets");
    assert_eq!(server.take_request().path, "/v1/knowledge/datasets/ds");
    assert_eq!(server.take_request().path, "/v1/knowledge/datasets");
    assert_eq!(server.take_request().path, "/v1/knowledge/datasets/ds");
    assert_eq!(server.take_request().path, "/v1/knowledge/datasets/ds/documents");
    assert_eq!(server.take_request().path, "/v1/knowledge/datasets/ds/documents");
    assert_eq!(server.take_request().path, "/v1/knowledge/documents/doc");
    assert_eq!(server.take_request().path, "/v1/knowledge/retrieve");
    server.join();
}
