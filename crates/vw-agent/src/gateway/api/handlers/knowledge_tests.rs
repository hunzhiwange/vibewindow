use super::*;
use axum::Json;
use axum::extract::Path;
use std::sync::Once;
use tokio::sync::Mutex;
use vw_api_types::knowledge::{
    KnowledgeChunkingMode, KnowledgeDatasetCreateRequest, KnowledgeDocumentCreateRequest,
    KnowledgeIndexingMode, KnowledgeRetrievalMode, KnowledgeRetrieveRequest,
};

static TEST_HOME: Once = Once::new();
static KNOWLEDGE_TEST_LOCK: Mutex<()> = Mutex::const_new(());

fn ensure_test_home() {
    TEST_HOME.call_once(|| {
        let dir =
            std::env::temp_dir().join(format!("vw-knowledge-handler-tests-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("test home should be created");
        unsafe {
            std::env::set_var("VIBEWINDOW_TEST_HOME", dir);
        }
    });
}

fn dataset_request(name: &str) -> KnowledgeDatasetCreateRequest {
    KnowledgeDatasetCreateRequest {
        name: name.to_string(),
        description: "handler test dataset".to_string(),
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

fn document_request(name: &str, content: &str) -> KnowledgeDocumentCreateRequest {
    KnowledgeDocumentCreateRequest {
        name: name.to_string(),
        content: content.to_string(),
        metadata: serde_json::json!({ "source": "handler-test" }),
        enabled: true,
    }
}

#[test]
fn router_builds_with_app_state() {
    let _: axum::Router<()> = router();
}

#[tokio::test]
async fn knowledge_status_reports_local_capabilities() {
    ensure_test_home();

    let Json(status) = knowledge_status().await.expect("status should succeed");

    assert!(status.full_text);
    assert!(status.rerank);
    assert_eq!(status.vector, status.hybrid);
    assert!(status.notes.contains_key("rerank"));
}

#[tokio::test]
async fn knowledge_dataset_document_and_retrieve_lifecycle() {
    let _guard = KNOWLEDGE_TEST_LOCK.lock().await;
    ensure_test_home();
    let suffix = uuid::Uuid::new_v4();

    let Json(dataset) =
        knowledge_dataset_create(Json(dataset_request(&format!("dataset-{suffix}"))))
            .await
            .expect("dataset create should succeed");

    let Json(listed) = knowledge_datasets_list().await.expect("dataset list should succeed");
    assert!(listed.iter().any(|item| item.id == dataset.id));

    let Json(fetched) =
        knowledge_dataset_get(Path(dataset.id.clone())).await.expect("dataset get should succeed");
    assert_eq!(fetched.name, dataset.name);

    let Json(document) = knowledge_document_create(
        Path(dataset.id.clone()),
        Json(document_request(
            &format!("document-{suffix}"),
            "Rustacean handlers can retrieve full text knowledge chunks.",
        )),
    )
    .await
    .expect("document create should succeed");
    assert_eq!(document.dataset_id, dataset.id);
    assert!(document.chunk_count > 0);

    let Json(documents) = knowledge_documents_list(Path(dataset.id.clone()))
        .await
        .expect("document list should succeed");
    assert!(documents.iter().any(|item| item.id == document.id));

    let Json(retrieved) = knowledge_retrieve(Json(KnowledgeRetrieveRequest {
        query: "Rustacean".to_string(),
        dataset_ids: vec![dataset.id.clone()],
        top_k: 3,
        score_threshold: None,
        metadata_filter: Some(serde_json::json!({ "source": "handler-test" })),
    }))
    .await
    .expect("retrieve should succeed");
    assert!(retrieved.chunks.iter().any(|chunk| chunk.document_id == document.id));

    let Json(deleted_document) = knowledge_document_delete(Path(document.id.clone()))
        .await
        .expect("document delete should succeed");
    assert_eq!(deleted_document.id, document.id);

    let Json(deleted_dataset) = knowledge_dataset_delete(Path(dataset.id.clone()))
        .await
        .expect("dataset delete should succeed");
    assert_eq!(deleted_dataset.id, dataset.id);
}

#[tokio::test]
async fn knowledge_handlers_return_validation_errors() {
    ensure_test_home();

    let err = knowledge_dataset_create(Json(dataset_request("   ")))
        .await
        .expect_err("blank dataset name should fail");
    assert_eq!(err.status, axum::http::StatusCode::BAD_REQUEST);

    let err = knowledge_dataset_get(Path("not-a-uuid".to_string()))
        .await
        .expect_err("invalid dataset id should fail");
    assert_eq!(err.status, axum::http::StatusCode::BAD_REQUEST);

    let err = knowledge_document_create(
        Path(uuid::Uuid::new_v4().to_string()),
        Json(document_request("doc", "   ")),
    )
    .await
    .expect_err("blank document content should fail");
    assert_eq!(err.status, axum::http::StatusCode::BAD_REQUEST);

    let err = knowledge_retrieve(Json(KnowledgeRetrieveRequest {
        query: " ".to_string(),
        dataset_ids: Vec::new(),
        top_k: 1,
        score_threshold: None,
        metadata_filter: None,
    }))
    .await
    .expect_err("blank retrieve query should fail");
    assert_eq!(err.status, axum::http::StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn knowledge_store_helper_uses_state_config() {
    let (event_tx, _) = tokio::sync::broadcast::channel(4);
    let state = AppState {
        config: std::sync::Arc::new(parking_lot::Mutex::new(
            crate::app::agent::config::Config::default(),
        )),
        provider: std::sync::Arc::new(StaticProvider),
        model: "test".to_string(),
        temperature: 0.0,
        mem: std::sync::Arc::new(crate::app::agent::memory::NoneMemory::new()),
        auto_save: false,
        webhook_secret_hash: None,
        pairing: std::sync::Arc::new(crate::app::agent::security::PairingGuard::new(false, &[])),
        trust_forwarded_headers: false,
        rate_limiter: std::sync::Arc::new(crate::app::agent::gateway::GatewayRateLimiter::new(
            10, 10, 10,
        )),
        idempotency_store: std::sync::Arc::new(crate::app::agent::gateway::IdempotencyStore::new(
            std::time::Duration::from_secs(60),
            10,
        )),
        whatsapp: None,
        whatsapp_app_secret: None,
        linq: None,
        linq_signing_secret: None,
        nextcloud_talk: None,
        nextcloud_talk_webhook_secret: None,
        wati: None,
        qq: None,
        qq_webhook_enabled: false,
        observer: std::sync::Arc::new(crate::app::agent::observability::NoopObserver),
        tools_registry: std::sync::Arc::new(Vec::new()),
        tools_registry_exec: std::sync::Arc::new(Vec::new()),
        multimodal: crate::app::agent::config::MultimodalConfig::default(),
        max_tool_iterations: 10,
        event_tx,
        session_query_engines: Default::default(),
    };

    let store = knowledge_store(&state);
    assert!(store.status().full_text);
    assert!(knowledge_db_path().ends_with("knowledge/knowledge.sqlite"));
}

struct StaticProvider;

#[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
impl crate::app::agent::providers::Provider for StaticProvider {
    async fn chat_with_system(
        &self,
        _system_prompt: Option<&str>,
        message: &str,
        _model: &str,
        _temperature: f64,
    ) -> anyhow::Result<String> {
        Ok(message.to_string())
    }
}
