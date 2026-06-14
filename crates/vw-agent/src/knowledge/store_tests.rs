use super::*;
use crate::memory::embeddings::EmbeddingProvider;
use crate::memory::vector;
use crate::workflow::{WorkflowKnowledgeProvider, WorkflowKnowledgeRequest};
use async_trait::async_trait;
use axum::http::StatusCode;
use rusqlite::Connection;
use serde_json::{Value, json};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use vw_api_types::knowledge::{
    KnowledgeChunkDto, KnowledgeChunkingMode, KnowledgeDatasetCreateRequest, KnowledgeDatasetDto,
    KnowledgeDocumentCreateRequest, KnowledgeIndexingMode, KnowledgeRetrievalMode,
    KnowledgeRetrieveRequest,
};

fn db_path() -> PathBuf {
    tempfile::tempdir().expect("tempdir").keep().join("knowledge.sqlite")
}

fn store() -> SqliteKnowledgeStore {
    SqliteKnowledgeStore::new(db_path())
}

fn vector_store(provider: Arc<CountingEmbedding>, db_path: PathBuf) -> SqliteKnowledgeStore {
    SqliteKnowledgeStore::with_embedder(
        db_path,
        provider,
        Some("fake-embedding".to_string()),
        0.75,
        0.25,
        64,
    )
}

fn dataset_request(name: &str) -> KnowledgeDatasetCreateRequest {
    KnowledgeDatasetCreateRequest {
        name: name.to_string(),
        description: String::new(),
        chunking_mode: KnowledgeChunkingMode::General,
        indexing_mode: KnowledgeIndexingMode::Economy,
        retrieval_mode: KnowledgeRetrievalMode::FullText,
        keyword_count: 10,
        top_k: 10,
        score_threshold_enabled: false,
        score_threshold: 0.15,
        rerank_enabled: false,
        embedding_model: None,
        rerank_model: None,
    }
}

fn document_request(name: &str, content: &str, metadata: Value) -> KnowledgeDocumentCreateRequest {
    KnowledgeDocumentCreateRequest {
        name: name.to_string(),
        content: content.to_string(),
        metadata,
        enabled: true,
    }
}

fn dataset_dto(chunking_mode: KnowledgeChunkingMode) -> KnowledgeDatasetDto {
    KnowledgeDatasetDto {
        id: uuid::Uuid::new_v4().to_string(),
        name: "Dataset".to_string(),
        description: String::new(),
        chunking_mode,
        indexing_mode: KnowledgeIndexingMode::Economy,
        retrieval_mode: KnowledgeRetrievalMode::FullText,
        keyword_count: 10,
        top_k: 10,
        score_threshold_enabled: false,
        score_threshold: 0.15,
        rerank_enabled: false,
        embedding_model: None,
        rerank_model: None,
        document_count: 0,
        chunk_count: 0,
        created_at_ms: 1,
        updated_at_ms: 1,
    }
}

fn chunk(id: &str, dataset_id: &str, title: &str, content: &str, score: f64) -> KnowledgeChunkDto {
    KnowledgeChunkDto {
        id: id.to_string(),
        dataset_id: dataset_id.to_string(),
        document_id: uuid::Uuid::new_v4().to_string(),
        title: title.to_string(),
        content: content.to_string(),
        metadata: json!({}),
        score: Some(score),
    }
}

fn assert_api_error(error: ApiError, status: StatusCode, needle: &str) {
    assert_eq!(error.status, status);
    assert!(error.to_string().contains(needle), "expected '{needle}' in '{}'", error);
}

#[derive(Default)]
struct CountingEmbedding {
    calls: AtomicUsize,
}

#[async_trait]
impl EmbeddingProvider for CountingEmbedding {
    fn name(&self) -> &str {
        "counting"
    }

    fn dimensions(&self) -> usize {
        3
    }

    async fn embed(&self, texts: &[&str]) -> anyhow::Result<Vec<Vec<f32>>> {
        self.calls.fetch_add(texts.len(), Ordering::SeqCst);
        Ok(texts.iter().map(|text| fake_vector(text)).collect())
    }
}

fn fake_vector(text: &str) -> Vec<f32> {
    let text = text.to_ascii_lowercase();
    if text.contains("refund") || text.contains("return") {
        vec![1.0, 0.0, 0.0]
    } else if text.contains("image") || text.contains("catalog") {
        vec![0.0, 1.0, 0.0]
    } else {
        vec![0.0, 0.0, 1.0]
    }
}

#[test]
fn status_reflects_embedding_provider_capabilities() {
    let plain = store().status();
    assert!(plain.full_text);
    assert!(!plain.vector);
    assert!(!plain.hybrid);
    assert!(plain.notes.contains_key("vector"));

    let provider = Arc::new(CountingEmbedding::default());
    let vector = vector_store(provider, db_path()).status();
    assert!(vector.vector);
    assert!(vector.hybrid);
    assert!(vector.rerank);
}

#[test]
fn validation_rejects_invalid_dataset_document_retrieve_and_ids() {
    let mut body = dataset_request("Dataset");

    body.name = "  ".to_string();
    assert_api_error(
        validate_dataset_request(&body, false, None).unwrap_err(),
        StatusCode::BAD_REQUEST,
        "name is required",
    );

    body = dataset_request("Dataset");
    body.keyword_count = 0;
    assert_api_error(
        validate_dataset_request(&body, false, None).unwrap_err(),
        StatusCode::BAD_REQUEST,
        "keyword_count",
    );

    body = dataset_request("Dataset");
    body.top_k = 0;
    assert_api_error(
        validate_dataset_request(&body, false, None).unwrap_err(),
        StatusCode::BAD_REQUEST,
        "top_k",
    );

    body = dataset_request("Dataset");
    body.score_threshold = 1.5;
    assert_api_error(
        validate_dataset_request(&body, false, None).unwrap_err(),
        StatusCode::BAD_REQUEST,
        "score_threshold",
    );

    body = dataset_request("Dataset");
    body.indexing_mode = KnowledgeIndexingMode::HighQuality;
    assert_api_error(
        validate_dataset_request(&body, false, None).unwrap_err(),
        StatusCode::NOT_IMPLEMENTED,
        "embedding provider",
    );

    body = dataset_request("Dataset");
    body.retrieval_mode = KnowledgeRetrievalMode::Hybrid;
    assert_api_error(
        validate_dataset_request(&body, false, None).unwrap_err(),
        StatusCode::NOT_IMPLEMENTED,
        "vector backend",
    );

    body = dataset_request("Dataset");
    body.embedding_model = Some("other-model".to_string());
    assert_api_error(
        validate_dataset_request(&body, true, Some("fake-embedding")).unwrap_err(),
        StatusCode::BAD_REQUEST,
        "configured model",
    );

    body = dataset_request("Dataset");
    body.rerank_model = Some("remote-reranker".to_string());
    assert_api_error(
        validate_dataset_request(&body, true, Some("fake-embedding")).unwrap_err(),
        StatusCode::BAD_REQUEST,
        "local-rerank-v1",
    );

    body = dataset_request("Dataset");
    body.embedding_model = Some("fake-embedding".to_string());
    body.rerank_model = Some("local-rerank-v1".to_string());
    assert!(validate_dataset_request(&body, true, Some("fake-embedding")).is_ok());

    assert_api_error(
        validate_document_request(&document_request(" ", "content", json!({}))).unwrap_err(),
        StatusCode::BAD_REQUEST,
        "document name",
    );
    assert_api_error(
        validate_document_request(&document_request("Doc", "  ", json!({}))).unwrap_err(),
        StatusCode::BAD_REQUEST,
        "content",
    );
    assert_api_error(
        validate_document_request(&document_request("Doc", "content", json!(["not-object"])))
            .unwrap_err(),
        StatusCode::BAD_REQUEST,
        "metadata",
    );

    assert_api_error(
        validate_retrieve_request(&KnowledgeRetrieveRequest {
            query: " ".to_string(),
            dataset_ids: vec![],
            top_k: 1,
            score_threshold: None,
            metadata_filter: None,
        })
        .unwrap_err(),
        StatusCode::BAD_REQUEST,
        "query",
    );
    assert_api_error(
        validate_retrieve_request(&KnowledgeRetrieveRequest {
            query: "refund".to_string(),
            dataset_ids: vec![],
            top_k: 0,
            score_threshold: None,
            metadata_filter: None,
        })
        .unwrap_err(),
        StatusCode::BAD_REQUEST,
        "top_k",
    );
    assert_api_error(
        validate_retrieve_request(&KnowledgeRetrieveRequest {
            query: "refund".to_string(),
            dataset_ids: vec![],
            top_k: 1,
            score_threshold: Some(-0.1),
            metadata_filter: None,
        })
        .unwrap_err(),
        StatusCode::BAD_REQUEST,
        "score_threshold",
    );
    assert_api_error(
        validate_retrieve_request(&KnowledgeRetrieveRequest {
            query: "refund".to_string(),
            dataset_ids: vec![],
            top_k: 1,
            score_threshold: None,
            metadata_filter: Some(json!(["bad"])),
        })
        .unwrap_err(),
        StatusCode::BAD_REQUEST,
        "metadata_filter",
    );
    assert_api_error(
        validate_id("not-a-uuid", "dataset_id").unwrap_err(),
        StatusCode::BAD_REQUEST,
        "dataset_id",
    );
}

#[test]
fn chunk_preparation_covers_general_parent_child_and_qa_modes() {
    let general = prepare_chunks_for_dataset(
        &dataset_dto(KnowledgeChunkingMode::General),
        &document_request("Doc", "alpha beta", json!({ "source": "test" })),
    )
    .expect("general chunks");
    assert_eq!(general.len(), 1);
    assert_eq!(general[0].title, "Doc");
    assert_eq!(general[0].metadata["chunking_mode"], json!("general"));

    let parent_child = prepare_chunks_for_dataset(
        &dataset_dto(KnowledgeChunkingMode::ParentChild),
        &document_request("Manual", &"parent context. child detail. ".repeat(80), Value::Null),
    )
    .expect("parent child chunks");
    assert!(!parent_child.is_empty());
    assert!(parent_child[0].title.starts_with("Manual · "));
    assert_eq!(parent_child[0].metadata["chunking_mode"], json!("parent_child"));
    assert!(parent_child[0].metadata.get("child_content").is_some());

    let qa = prepare_chunks_for_dataset(
        &dataset_dto(KnowledgeChunkingMode::Qa),
        &document_request(
            "FAQ",
            "问题：如何退款？\n答案：七天内处理。\n\nQuestion: Images?\nAnswer: Use catalog settings.",
            Value::Null,
        ),
    )
    .expect("qa chunks");
    assert_eq!(qa.len(), 2);
    assert_eq!(qa[0].title, "如何退款？");
    assert_eq!(qa[0].content, "七天内处理。");

    let paragraph_pairs =
        parse_qa_pairs("First question?\nFirst answer.\n\nSecond question?\nSecond answer.");
    assert_eq!(paragraph_pairs.len(), 2);

    assert_api_error(
        prepare_qa_chunks(&document_request("Bad FAQ", "only one line", Value::Null)).unwrap_err(),
        StatusCode::BAD_REQUEST,
        "question and answer",
    );
}

#[test]
fn metadata_query_and_score_helpers_handle_edge_cases() {
    assert_eq!(metadata_or_object(Value::Null), json!({}));
    assert_eq!(metadata_or_object(json!({ "a": 1 })), json!({ "a": 1 }));

    let metadata = chunk_metadata(
        &json!({ "tag": "support" }),
        "Doc",
        3,
        &json!({ "chunking_mode": "qa", "tag": "override" }),
    );
    assert_eq!(metadata["document_name"], json!("Doc"));
    assert_eq!(metadata["chunk_index"], json!(3));
    assert_eq!(metadata["tag"], json!("override"));

    assert!(metadata_matches(&metadata, &None));
    assert!(metadata_matches(&metadata, &Some(json!({ "tag": "override" }))));
    assert!(!metadata_matches(&metadata, &Some(json!({ "tag": "support" }))));
    assert!(metadata_matches(&json!("x"), &Some(json!("x"))));

    assert_eq!(dataset_filter_sql("c", 0), "");
    assert_eq!(dataset_filter_sql("c", 2), " AND c.dataset_id IN (?, ?)");
    assert_eq!(quote_fts_term(r#"hello"world"#), r#""hello""world""#);
    assert_eq!(fts_query_from_text("alpha  beta"), r#""alpha" OR "beta""#);
    assert_eq!(fts_query_from_text("   "), "");
    assert_eq!(like_pattern(r#"50%_done\ok"#), r#"%50\%\_done\\ok%"#);

    let scored = KnowledgeChunkDto {
        id: "id".into(),
        dataset_id: "dataset".into(),
        document_id: "doc".into(),
        title: "Refund policy".into(),
        content: "Content body".into(),
        metadata: json!({}),
        score: None,
    };
    assert_eq!(like_score(&scored, "refund"), 0.65);
    assert_eq!(like_score(&scored, "body"), 0.55);
    assert_eq!(like_score(&scored, "missing"), 0.1);
    assert!(fts_score(0.0) > fts_score(3.0));
}

#[test]
fn mode_hash_bool_and_time_helpers_are_stable() {
    assert_eq!(bool_to_i64(true), 1);
    assert_eq!(bool_to_i64(false), 0);
    assert_eq!(chunking_mode_as_str(&KnowledgeChunkingMode::General), "general");
    assert_eq!(chunking_mode_as_str(&KnowledgeChunkingMode::ParentChild), "parent_child");
    assert_eq!(chunking_mode_as_str(&KnowledgeChunkingMode::Qa), "qa");
    assert_eq!(indexing_mode_as_str(&KnowledgeIndexingMode::Economy), "economy");
    assert_eq!(indexing_mode_as_str(&KnowledgeIndexingMode::HighQuality), "high_quality");
    assert_eq!(retrieval_mode_as_str(&KnowledgeRetrievalMode::FullText), "full_text");
    assert_eq!(retrieval_mode_as_str(&KnowledgeRetrievalMode::Vector), "vector");
    assert_eq!(retrieval_mode_as_str(&KnowledgeRetrievalMode::Hybrid), "hybrid");
    assert_eq!(parse_chunking_mode("parent_child"), KnowledgeChunkingMode::ParentChild);
    assert_eq!(parse_chunking_mode("qa"), KnowledgeChunkingMode::Qa);
    assert_eq!(parse_chunking_mode("unknown"), KnowledgeChunkingMode::General);
    assert_eq!(parse_indexing_mode("high_quality"), KnowledgeIndexingMode::HighQuality);
    assert_eq!(parse_indexing_mode("unknown"), KnowledgeIndexingMode::Economy);
    assert_eq!(parse_retrieval_mode("vector"), KnowledgeRetrievalMode::Vector);
    assert_eq!(parse_retrieval_mode("hybrid"), KnowledgeRetrievalMode::Hybrid);
    assert_eq!(parse_retrieval_mode("unknown"), KnowledgeRetrievalMode::FullText);
    assert_eq!(content_hash("abc").len(), 64);
    assert!(now_ms() > 0);
}

#[test]
fn retrieval_score_helpers_merge_and_rerank_results() {
    let vector = vec![("a".to_string(), 0.9), ("b".to_string(), 0.1)];
    let keyword = vec![("b".to_string(), 0.8), ("c".to_string(), 0.7)];

    let vector_only = merge_retrieval_scores(&vector, &[], 0.7, 0.3, 10);
    assert_eq!(vector_only[0].id, "a");
    assert_eq!(vector_only[0].vector_score, Some(0.9));

    let keyword_only = merge_retrieval_scores(&[], &keyword, 0.7, 0.3, 10);
    assert_eq!(keyword_only[0].id, "b");
    assert_eq!(keyword_only[0].keyword_score, Some(0.8));

    let hybrid = merge_retrieval_scores(&vector, &keyword, 0.7, 0.3, 2);
    assert_eq!(hybrid.len(), 2);

    let dataset_id = uuid::Uuid::new_v4().to_string();
    let other_dataset = uuid::Uuid::new_v4().to_string();
    let mut chunks = vec![
        chunk("low", &dataset_id, "General", "no exact words", 0.9),
        chunk("lexical", &dataset_id, "Refund policy", "refund terms", 0.4),
        chunk("other", &other_dataset, "Refund policy", "refund terms", 0.2),
    ];
    chunks[1].metadata = json!({ "question": "How do refunds work?" });
    rerank_chunks("refund", &[dataset_id.clone()], &mut chunks);
    assert_eq!(chunks[0].id, "low");
    assert!(chunks[0].score.unwrap_or_default() > 0.4);
    let other = chunks.iter().find(|chunk| chunk.id == "other").expect("other chunk");
    assert_eq!(rerank_score("refund", &[dataset_id], other), other.score.unwrap());
    assert_eq!(lexical_match_score("", &chunks[0]), 0.0);
}

#[test]
fn open_db_migrates_legacy_schema_and_embedding_cache_prunes_old_entries() {
    let path = db_path();
    std::fs::create_dir_all(path.parent().expect("parent")).expect("create parent");
    {
        let conn = Connection::open(&path).expect("legacy db");
        conn.execute_batch(
            r#"
            CREATE TABLE knowledge_datasets (
                id TEXT PRIMARY KEY NOT NULL,
                name TEXT NOT NULL,
                description TEXT NOT NULL,
                indexing_mode TEXT NOT NULL,
                retrieval_mode TEXT NOT NULL,
                embedding_model TEXT,
                rerank_model TEXT,
                created_at_ms INTEGER NOT NULL,
                updated_at_ms INTEGER NOT NULL
            );
            CREATE TABLE knowledge_chunks (
                id TEXT PRIMARY KEY NOT NULL,
                dataset_id TEXT NOT NULL,
                document_id TEXT NOT NULL,
                ordinal INTEGER NOT NULL,
                title TEXT NOT NULL,
                content TEXT NOT NULL,
                metadata_json TEXT NOT NULL,
                enabled INTEGER NOT NULL,
                created_at_ms INTEGER NOT NULL,
                updated_at_ms INTEGER NOT NULL
            );
            "#,
        )
        .expect("legacy schema");
    }

    let conn = open_db(&path).expect("open migrated db");
    assert!(column_exists(&conn, "knowledge_datasets", "chunking_mode").unwrap());
    assert!(column_exists(&conn, "knowledge_datasets", "keyword_count").unwrap());
    assert!(column_exists(&conn, "knowledge_chunks", "embedding").unwrap());
    assert!(!column_exists(&conn, "knowledge_chunks", "missing_column").unwrap());

    write_embedding_cache_blocking(path.clone(), "a", vector::vec_to_bytes(&[1.0, 0.0]), 2)
        .expect("write a");
    std::thread::sleep(std::time::Duration::from_millis(2));
    write_embedding_cache_blocking(path.clone(), "b", vector::vec_to_bytes(&[0.0, 1.0]), 1)
        .expect("write b");
    assert!(read_embedding_cache_blocking(path.clone(), "a").unwrap().is_none());
    assert_eq!(read_embedding_cache_blocking(path, "b").unwrap().unwrap(), vec![0.0, 1.0]);
}

#[tokio::test]
async fn dataset_document_lifecycle_trims_counts_lists_and_deletes() {
    let store = store();
    let mut request = dataset_request("  Support  ");
    request.description = "  Help center  ".to_string();
    request.top_k = 500;
    request.score_threshold_enabled = true;
    request.score_threshold = 0.2;
    request.rerank_enabled = true;
    request.rerank_model = Some("local-rerank-v1".to_string());
    let dataset = store.create_dataset(request).await.expect("dataset");

    assert_eq!(dataset.name, "Support");
    assert_eq!(dataset.description, "Help center");
    assert_eq!(dataset.top_k, MAX_TOP_K);
    assert!(dataset.rerank_enabled);
    assert_eq!(dataset.document_count, 0);

    assert_eq!(store.list_datasets().await.unwrap().len(), 1);
    assert_eq!(store.get_dataset(dataset.id.clone()).await.unwrap().id, dataset.id);

    let document = store
        .create_document(
            dataset.id.clone(),
            document_request(
                "  Return policy  ",
                "Refunds are available within seven days.",
                Value::Null,
            ),
        )
        .await
        .expect("document");
    assert_eq!(document.name, "Return policy");
    assert_eq!(document.metadata, json!({}));
    assert_eq!(document.chunk_count, 1);

    let documents = store.list_documents(dataset.id.clone()).await.expect("documents");
    assert_eq!(documents.len(), 1);
    assert_eq!(store.get_dataset(dataset.id.clone()).await.unwrap().document_count, 1);

    let deleted_doc = store.delete_document(document.id.clone()).await.expect("delete doc");
    assert_eq!(deleted_doc.id, document.id);
    assert!(store.list_documents(dataset.id.clone()).await.unwrap().is_empty());

    let deleted_dataset = store.delete_dataset(dataset.id.clone()).await.expect("delete dataset");
    assert_eq!(deleted_dataset.id, dataset.id);
    assert_api_error(
        store.get_dataset(dataset.id).await.unwrap_err(),
        StatusCode::NOT_FOUND,
        "dataset not found",
    );
}

#[tokio::test]
async fn disabled_documents_are_not_retrieved_and_empty_scope_searches_all_datasets() {
    let store = store();
    let disabled_dataset = store.create_dataset(dataset_request("Disabled")).await.unwrap();
    let all_dataset = store.create_dataset(dataset_request("All")).await.unwrap();

    let mut disabled_doc =
        document_request("Hidden", "secret refund policy", json!({ "tag": "hidden" }));
    disabled_doc.enabled = false;
    store.create_document(disabled_dataset.id.clone(), disabled_doc).await.unwrap();
    store
        .create_document(
            all_dataset.id.clone(),
            document_request("Visible", "visible refund policy", json!({ "tag": "visible" })),
        )
        .await
        .unwrap();

    let hidden = store
        .retrieve(KnowledgeRetrieveRequest {
            query: "secret refund".to_string(),
            dataset_ids: vec![disabled_dataset.id],
            top_k: 5,
            score_threshold: None,
            metadata_filter: None,
        })
        .await
        .unwrap();
    assert!(hidden.chunks.is_empty());

    let visible = store
        .retrieve(KnowledgeRetrieveRequest {
            query: "visible refund".to_string(),
            dataset_ids: vec![],
            top_k: 5,
            score_threshold: Some(0.0),
            metadata_filter: Some(json!({ "tag": "visible" })),
        })
        .await
        .unwrap();
    assert_eq!(visible.chunks.len(), 1);
    assert_eq!(visible.chunks[0].title, "Visible");
}

#[tokio::test]
async fn vector_retrieval_uses_embedding_cache_metadata_and_thresholds() {
    let provider = Arc::new(CountingEmbedding::default());
    let store = vector_store(provider.clone(), db_path());
    let mut request = dataset_request("Vector");
    request.indexing_mode = KnowledgeIndexingMode::HighQuality;
    request.retrieval_mode = KnowledgeRetrievalMode::Hybrid;
    request.embedding_model = Some("fake-embedding".to_string());
    let dataset = store.create_dataset(request).await.expect("dataset");

    store
        .create_document(
            dataset.id.clone(),
            document_request(
                "Refunds",
                "Refund requests return money to customers.",
                json!({ "kind": "support" }),
            ),
        )
        .await
        .expect("refund doc");
    store
        .create_document(
            dataset.id.clone(),
            document_request(
                "Images",
                "Catalog image setup belongs to media settings.",
                json!({ "kind": "media" }),
            ),
        )
        .await
        .expect("image doc");

    let calls_before_query = provider.calls.load(Ordering::SeqCst);
    let request = KnowledgeRetrieveRequest {
        query: "refund".to_string(),
        dataset_ids: vec![dataset.id.clone()],
        top_k: 1,
        score_threshold: Some(0.0),
        metadata_filter: Some(json!({ "kind": "support" })),
    };
    let response = store.retrieve(request.clone()).await.expect("retrieve");
    assert_eq!(response.chunks.len(), 1);
    assert_eq!(response.chunks[0].title, "Refunds");
    assert!(provider.calls.load(Ordering::SeqCst) > calls_before_query);

    let calls_after_first_query = provider.calls.load(Ordering::SeqCst);
    let cached = store.retrieve(request).await.expect("cached retrieve");
    assert_eq!(cached.chunks.len(), 1);
    assert_eq!(provider.calls.load(Ordering::SeqCst), calls_after_first_query);

    let filtered = store
        .retrieve(KnowledgeRetrieveRequest {
            query: "refund".to_string(),
            dataset_ids: vec![dataset.id],
            top_k: 1,
            score_threshold: Some(1.1_f64.min(1.0)),
            metadata_filter: Some(json!({ "kind": "media" })),
        })
        .await
        .expect("filtered retrieve");
    assert!(filtered.chunks.is_empty());
}

#[tokio::test]
async fn vector_scope_without_embedder_returns_not_implemented() {
    let path = db_path();
    let provider = Arc::new(CountingEmbedding::default());
    let vector = vector_store(provider, path.clone());
    let mut request = dataset_request("Vector only");
    request.retrieval_mode = KnowledgeRetrievalMode::Vector;
    request.embedding_model = Some("fake-embedding".to_string());
    let dataset = vector.create_dataset(request).await.expect("vector dataset");

    let plain = SqliteKnowledgeStore::new(path);
    assert_api_error(
        plain
            .retrieve(KnowledgeRetrieveRequest {
                query: "refund".to_string(),
                dataset_ids: vec![dataset.id],
                top_k: 1,
                score_threshold: None,
                metadata_filter: None,
            })
            .await
            .unwrap_err(),
        StatusCode::NOT_IMPLEMENTED,
        "embedding provider",
    );
}

#[tokio::test]
async fn invalid_and_missing_records_return_stable_api_errors() {
    let store = store();

    assert_api_error(
        store.list_documents("not-a-uuid".to_string()).await.unwrap_err(),
        StatusCode::BAD_REQUEST,
        "dataset_id",
    );
    assert_api_error(
        store.delete_document(uuid::Uuid::new_v4().to_string()).await.unwrap_err(),
        StatusCode::NOT_FOUND,
        "document not found",
    );
    assert_api_error(
        store.delete_dataset(uuid::Uuid::new_v4().to_string()).await.unwrap_err(),
        StatusCode::NOT_FOUND,
        "dataset not found",
    );
}

#[tokio::test]
async fn workflow_provider_maps_retrieval_errors_to_strings() {
    let store = store();
    let err = match WorkflowKnowledgeProvider::retrieve(
        &store,
        WorkflowKnowledgeRequest {
            query: " ".to_string(),
            dataset_ids: vec![],
            top_k: 1,
            score_threshold: None,
            metadata_filter: None,
        },
    )
    .await
    {
        Ok(_) => panic!("invalid workflow query should fail"),
        Err(error) => error,
    };

    assert!(err.contains("query is required"));
}
