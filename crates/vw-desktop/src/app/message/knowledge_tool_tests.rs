//! 覆盖知识库工具消息处理的本地状态转换与参数校验。

use super::{
    KnowledgeToolMessage, dataset_embedding_model_override, parse_score_threshold,
    parse_usize_input, trim_to_option, update,
};
use crate::app::{App, state::KnowledgeDetailTab};
use iced::widget::text_editor;
use serde_json::Value;
use std::collections::BTreeMap;
use vw_gateway_client::{
    KnowledgeChunkDto, KnowledgeChunkingMode, KnowledgeDatasetDto, KnowledgeDocumentDto,
    KnowledgeIndexingMode, KnowledgeRetrievalMode, KnowledgeRetrieveResponse,
    KnowledgeRuntimeStatus,
};

fn test_app() -> App {
    App::new().0
}

fn dataset(id: &str, top_k: usize, threshold_enabled: bool, threshold: f64) -> KnowledgeDatasetDto {
    KnowledgeDatasetDto {
        id: id.to_string(),
        name: format!("dataset {id}"),
        description: "desc".to_string(),
        chunking_mode: KnowledgeChunkingMode::General,
        indexing_mode: KnowledgeIndexingMode::HighQuality,
        retrieval_mode: KnowledgeRetrievalMode::Hybrid,
        keyword_count: 12,
        top_k,
        score_threshold_enabled: threshold_enabled,
        score_threshold: threshold,
        rerank_enabled: false,
        embedding_model: None,
        rerank_model: None,
        document_count: 0,
        chunk_count: 0,
        created_at_ms: 1,
        updated_at_ms: 2,
    }
}

fn document(id: &str, dataset_id: &str) -> KnowledgeDocumentDto {
    KnowledgeDocumentDto {
        id: id.to_string(),
        dataset_id: dataset_id.to_string(),
        name: format!("doc {id}"),
        metadata: Value::Null,
        enabled: true,
        chunk_count: 1,
        created_at_ms: 1,
        updated_at_ms: 2,
    }
}

fn chunk(id: &str) -> KnowledgeChunkDto {
    KnowledgeChunkDto {
        id: id.to_string(),
        dataset_id: "ds".to_string(),
        document_id: "doc".to_string(),
        title: "title".to_string(),
        content: "content".to_string(),
        metadata: Value::Null,
        score: Some(0.9),
    }
}

#[test]
fn parse_helpers_trim_clamp_and_reject_non_finite_thresholds() {
    assert_eq!(parse_usize_input(" 42 ", 7), 42);
    assert_eq!(parse_usize_input("bad", 7), 7);
    assert_eq!(parse_score_threshold("0.42"), Some(0.42));
    assert_eq!(parse_score_threshold("-1"), Some(0.0));
    assert_eq!(parse_score_threshold("9"), Some(1.0));
    assert_eq!(parse_score_threshold("nan"), None);
    assert_eq!(trim_to_option("  model  "), Some("model".to_string()));
    assert_eq!(trim_to_option("  "), None);
}

#[test]
fn dataset_embedding_override_only_returns_non_default_value() {
    let mut app = test_app();
    app.memory_settings.embedding_model = "base-embed".to_string();

    app.knowledge.dataset_embedding_model_input = String::new();
    assert_eq!(dataset_embedding_model_override(&app), None);

    app.knowledge.dataset_embedding_model_input = " base-embed ".to_string();
    assert_eq!(dataset_embedding_model_override(&app), None);

    app.knowledge.dataset_embedding_model_input = "custom-embed".to_string();
    assert_eq!(dataset_embedding_model_override(&app), Some("custom-embed".to_string()));
}

#[test]
fn update_mutates_simple_inputs_and_mode_constraints() {
    let mut app = test_app();

    let _ = update(&mut app, KnowledgeToolMessage::SelectTab(KnowledgeDetailTab::Retrieval));
    let _ = update(&mut app, KnowledgeToolMessage::DatasetSearchChanged("data".to_string()));
    let _ = update(&mut app, KnowledgeToolMessage::DocumentSearchChanged("doc".to_string()));
    let _ = update(&mut app, KnowledgeToolMessage::DatasetNameChanged("name".to_string()));
    let _ = update(&mut app, KnowledgeToolMessage::DatasetDescriptionChanged("desc".to_string()));
    let _ = update(
        &mut app,
        KnowledgeToolMessage::DatasetChunkingModeChanged(KnowledgeChunkingMode::Qa),
    );
    let _ = update(
        &mut app,
        KnowledgeToolMessage::DatasetRetrievalModeChanged(KnowledgeRetrievalMode::Vector),
    );

    assert_eq!(app.knowledge.active_tab, KnowledgeDetailTab::Retrieval);
    assert_eq!(app.knowledge.dataset_search_query, "data");
    assert_eq!(app.knowledge.document_search_query, "doc");
    assert_eq!(app.knowledge.dataset_name_input, "name");
    assert_eq!(app.knowledge.dataset_description_input, "desc");
    assert_eq!(app.knowledge.dataset_chunking_mode, KnowledgeChunkingMode::Qa);
    assert_eq!(app.knowledge.dataset_indexing_mode, KnowledgeIndexingMode::HighQuality);
    assert_eq!(app.knowledge.dataset_retrieval_mode, KnowledgeRetrievalMode::Vector);

    app.knowledge.dataset_rerank_enabled = true;
    app.knowledge.dataset_score_threshold_enabled = true;
    let _ = update(
        &mut app,
        KnowledgeToolMessage::DatasetIndexingModeChanged(KnowledgeIndexingMode::Economy),
    );

    assert_eq!(app.knowledge.dataset_retrieval_mode, KnowledgeRetrievalMode::FullText);
    assert!(!app.knowledge.dataset_rerank_enabled);
    assert!(!app.knowledge.dataset_score_threshold_enabled);

    let _ = update(&mut app, KnowledgeToolMessage::DatasetRerankEnabledChanged(true));
    assert_eq!(app.knowledge.dataset_rerank_model_input, "local-rerank-v1");
}

#[test]
fn update_handles_load_success_error_and_stale_documents() {
    let mut app = test_app();

    let mut notes = BTreeMap::new();
    notes.insert("engine".to_string(), "ok".to_string());
    let status = KnowledgeRuntimeStatus {
        full_text: true,
        vector: true,
        hybrid: true,
        rerank: false,
        notes,
    };

    app.knowledge.loading_status = true;
    let _ = update(&mut app, KnowledgeToolMessage::StatusLoaded(Ok(status.clone())));
    assert!(!app.knowledge.loading_status);
    assert_eq!(app.knowledge.runtime_status, Some(status));

    app.knowledge.loading_status = true;
    let _ = update(&mut app, KnowledgeToolMessage::StatusLoaded(Err("offline".to_string())));
    assert_eq!(app.knowledge.notification.as_deref(), Some("offline"));
    assert!(app.knowledge.notification_is_error);

    let _ = update(
        &mut app,
        KnowledgeToolMessage::DatasetsLoaded(Ok(vec![dataset("ds1", 99, true, 0.234)])),
    );
    assert_eq!(app.knowledge.selected_dataset_id.as_deref(), Some("ds1"));
    assert_eq!(app.knowledge.retrieve_top_k_input, "50");
    assert_eq!(app.knowledge.retrieve_score_threshold_input, "0.23");
    assert!(app.knowledge.loading_documents);

    let _ = update(
        &mut app,
        KnowledgeToolMessage::DocumentsLoaded("other".to_string(), Ok(vec![document("d1", "ds1")])),
    );
    assert!(app.knowledge.documents.is_empty());

    let _ = update(
        &mut app,
        KnowledgeToolMessage::DocumentsLoaded("ds1".to_string(), Ok(vec![document("d1", "ds1")])),
    );
    assert_eq!(app.knowledge.documents.len(), 1);
}

#[test]
fn update_validation_paths_set_local_errors_without_gateway() {
    let mut app = test_app();

    let _ = update(&mut app, KnowledgeToolMessage::CreateDataset);
    assert_eq!(app.knowledge.notification.as_deref(), Some("知识库名称不能为空"));

    app.knowledge.dataset_name_input = "docs".to_string();
    app.knowledge.dataset_score_threshold_input = "bad".to_string();
    let _ = update(&mut app, KnowledgeToolMessage::CreateDataset);
    assert_eq!(app.knowledge.notification.as_deref(), Some("Score 阈值必须是 0 到 1 之间的数字"));

    let _ = update(&mut app, KnowledgeToolMessage::CreateDocument);
    assert_eq!(app.knowledge.notification.as_deref(), Some("请先选择知识库"));

    app.knowledge.selected_dataset_id = Some("ds".to_string());
    let _ = update(&mut app, KnowledgeToolMessage::CreateDocument);
    assert_eq!(app.knowledge.notification.as_deref(), Some("文档名称不能为空"));

    app.knowledge.document_name_input = "doc".to_string();
    let _ = update(&mut app, KnowledgeToolMessage::CreateDocument);
    assert_eq!(app.knowledge.notification.as_deref(), Some("文档内容不能为空"));

    let _ = update(&mut app, KnowledgeToolMessage::Retrieve);
    assert_eq!(app.knowledge.notification.as_deref(), Some("请输入召回测试文本"));
}

#[test]
fn update_completion_messages_reset_flags_and_clear_drafts() {
    let mut app = test_app();
    app.knowledge.datasets = vec![dataset("ds1", 8, false, 0.1), dataset("ds2", 4, true, 0.5)];
    app.knowledge.documents = vec![document("d1", "ds1"), document("d2", "ds1")];
    app.knowledge.retrieve_results = vec![chunk("old")];

    let _ =
        update(&mut app, KnowledgeToolMessage::DatasetDeleted(Ok(dataset("ds1", 8, false, 0.1))));
    assert!(!app.knowledge.deleting);
    assert_eq!(app.knowledge.datasets.len(), 1);
    assert_eq!(app.knowledge.selected_dataset_id.as_deref(), Some("ds2"));
    assert!(app.knowledge.documents.is_empty());
    assert!(app.knowledge.retrieve_results.is_empty());

    app.knowledge.documents = vec![document("d1", "ds2")];
    let _ = update(&mut app, KnowledgeToolMessage::DocumentDeleted(Ok(document("d1", "ds2"))));
    assert!(app.knowledge.documents.is_empty());
    assert_eq!(app.knowledge.notification.as_deref(), Some("文档已删除"));

    let _ = update(
        &mut app,
        KnowledgeToolMessage::RetrieveFinished(Ok(KnowledgeRetrieveResponse {
            chunks: vec![chunk("c1")],
        })),
    );
    assert!(!app.knowledge.retrieving);
    assert_eq!(app.knowledge.retrieve_results.len(), 1);

    app.knowledge.document_name_input = "draft".to_string();
    app.knowledge.document_content_editor = text_editor::Content::with_text("content");
    let _ = update(&mut app, KnowledgeToolMessage::ClearDocumentDraft);
    assert!(app.knowledge.document_name_input.is_empty());
    assert!(app.knowledge.document_content_editor.text().is_empty());

    let _ = update(&mut app, KnowledgeToolMessage::ClearNotification);
    assert!(app.knowledge.notification.is_none());
    assert!(!app.knowledge.notification_is_error);
}
