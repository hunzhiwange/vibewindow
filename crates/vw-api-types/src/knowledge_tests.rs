use super::knowledge::{
    KnowledgeDatasetCreateRequest, KnowledgeIndexingMode, KnowledgeRetrievalMode,
    KnowledgeRetrieveRequest,
};

#[test]
fn knowledge_dataset_create_defaults_to_local_full_text() {
    let request: KnowledgeDatasetCreateRequest =
        serde_json::from_value(serde_json::json!({ "name": "Docs" })).expect("valid request");

    assert_eq!(request.indexing_mode, KnowledgeIndexingMode::Economy);
    assert_eq!(request.retrieval_mode, KnowledgeRetrievalMode::FullText);
    assert_eq!(request.description, "");
}

#[test]
fn knowledge_retrieve_defaults_top_k() {
    let request: KnowledgeRetrieveRequest =
        serde_json::from_value(serde_json::json!({ "query": "refund policy" }))
            .expect("valid request");

    assert_eq!(request.top_k, 10);
    assert!(request.dataset_ids.is_empty());
}
