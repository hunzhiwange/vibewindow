use crate::app::state::EmbeddingRouteDraft;
use crate::app::{App, Message};
use iced::widget::button;
use iced::{Element, Size, Theme};
use std::collections::BTreeMap;

use super::{
    build_dataset_param_section, build_detail, build_documents_tab, build_documents_table,
    build_embedding_model_section, build_hero, build_indexing_mode_section, build_rerank_toggle,
    build_retrieval_mode_section, build_retrieval_tab, build_settings_tab, build_sidebar,
    build_workspace, chrome_style, chunk_card, chunking_label, compact_input_row,
    dataset_embedding_model_label, dataset_item, document_row, empty_hint, enabled_label,
    format_time, icon_badge, indexing_label, is_dark_palette, item_style,
    knowledge_embedding_model_options, knowledge_embedding_model_selected,
    knowledge_toolbar_button_style, option_card, option_card_style, retrieval_label,
    score_threshold_label, section_header, segment_button, segment_style,
    shared_embedding_model_option, shared_embedding_model_summary, support_label,
};
use vw_gateway_client::{
    KnowledgeChunkDto, KnowledgeChunkingMode, KnowledgeDatasetDto, KnowledgeDocumentDto,
    KnowledgeIndexingMode, KnowledgeRetrievalMode, KnowledgeRuntimeStatus,
};

fn test_app() -> App {
    App::new().0
}

fn keep_element(element: Element<'_, Message>) {
    std::hint::black_box(element);
}

fn dataset(id: &str) -> KnowledgeDatasetDto {
    KnowledgeDatasetDto {
        id: id.to_string(),
        name: format!("dataset {id}"),
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
        document_count: 2,
        chunk_count: 8,
        created_at_ms: 1,
        updated_at_ms: 2,
    }
}

fn document(id: &str) -> KnowledgeDocumentDto {
    KnowledgeDocumentDto {
        id: id.to_string(),
        dataset_id: "dataset-a".to_string(),
        name: format!("document {id}"),
        metadata: serde_json::json!({}),
        enabled: true,
        chunk_count: 3,
        created_at_ms: 1,
        updated_at_ms: 2,
    }
}

fn chunk(id: &str, score: Option<f64>) -> KnowledgeChunkDto {
    KnowledgeChunkDto {
        id: id.to_string(),
        dataset_id: "dataset-a".to_string(),
        document_id: "document-a".to_string(),
        title: format!("chunk {id}"),
        content: "content".to_string(),
        metadata: serde_json::json!({}),
        score,
    }
}

fn runtime_status(vector: bool, rerank: bool) -> KnowledgeRuntimeStatus {
    KnowledgeRuntimeStatus {
        full_text: true,
        vector,
        hybrid: vector,
        rerank,
        notes: BTreeMap::from([("engine".to_string(), "sqlite".to_string())]),
    }
}

#[test]
fn knowledge_labels_match_ui_copy() {
    assert_eq!(indexing_label(&KnowledgeIndexingMode::Economy), "经济");
    assert_eq!(indexing_label(&KnowledgeIndexingMode::HighQuality), "高质量");
    assert_eq!(chunking_label(&KnowledgeChunkingMode::General), "General");
    assert_eq!(retrieval_label(&KnowledgeRetrievalMode::Hybrid), "混合检索");
    assert_eq!(support_label(true), "可用");
    assert_eq!(support_label(false), "不可用");
    assert_eq!(enabled_label(true), "启用");
    assert_eq!(enabled_label(false), "关闭");
}

#[test]
fn detail_tab_titles_match_ui_copy() {
    assert_eq!(crate::app::state::KnowledgeDetailTab::Documents.title(), "文档");
    assert_eq!(crate::app::state::KnowledgeDetailTab::Retrieval.title(), "召回测试");
    assert_eq!(crate::app::state::KnowledgeDetailTab::Settings.title(), "设置");
}

#[test]
fn shared_embedding_summary_resolves_hint_route() {
    let routes = vec![EmbeddingRouteDraft {
        pattern: "semantic".to_string(),
        provider: "alibaba-cn".to_string(),
        model: "text-embedding-v4".to_string(),
        dimensions: "1024".to_string(),
        api_key_input: String::new(),
    }];

    assert_eq!(
        shared_embedding_model_summary("alibaba-cn", "hint:semantic", 1536, &routes),
        "semantic -> text-embedding-v4 / 1024维"
    );
    assert_eq!(
        shared_embedding_model_summary("none", "hint:semantic", 1536, &routes),
        "semantic -> text-embedding-v4 / 1024维"
    );
}

#[test]
fn shared_embedding_summary_handles_plain_models_and_empty_values() {
    assert_eq!(shared_embedding_model_summary("", "", 1536, &[]), "未配置向量化");
    assert_eq!(
        shared_embedding_model_summary("none", "text-embedding-3-small", 1536, &[]),
        "未配置向量化"
    );
    assert_eq!(
        shared_embedding_model_summary("openai", "text-embedding-3-small", 1536, &[]),
        "text-embedding-3-small / 1536维"
    );
    assert_eq!(
        shared_embedding_model_summary("openai", "hint: semantic ", 1024, &[]),
        "hint: semantic / 1024维"
    );
}

#[test]
fn shared_embedding_option_keeps_empty_value_for_memory_config() {
    let option = shared_embedding_model_option("alibaba-cn", "text-embedding-v4", 1024, &[]);

    assert_eq!(option.value, "");
    assert_eq!(option.label, "记忆配置：text-embedding-v4 / 1024维");
}

#[test]
fn shared_embedding_summary_falls_back_for_invalid_hint_route() {
    let routes = vec![EmbeddingRouteDraft {
        pattern: "semantic".to_string(),
        provider: String::new(),
        model: "text-embedding-v4".to_string(),
        dimensions: "0".to_string(),
        api_key_input: String::new(),
    }];

    assert_eq!(
        shared_embedding_model_summary("none", "hint:semantic", 1536, &routes),
        "未配置向量化"
    );
    assert_eq!(
        shared_embedding_model_summary("alibaba-cn", "hint:semantic", 1536, &routes),
        "hint:semantic / 1536维"
    );
}

#[test]
fn knowledge_embedding_model_options_include_current_override() {
    let mut app = test_app();
    app.memory_settings.embedding_provider = "openai".to_string();
    app.memory_settings.embedding_model = "text-embedding-3-small".to_string();
    app.memory_settings.embedding_dimensions = 1536;
    app.knowledge.dataset_embedding_model_input = " custom-embedding ".to_string();

    let options = knowledge_embedding_model_options(&app);
    let selected = knowledge_embedding_model_selected(&app);

    assert_eq!(options.len(), 2);
    assert_eq!(options[0].value, "");
    assert_eq!(options[1].value, "custom-embedding");
    assert_eq!(selected.value, "custom-embedding");
    assert_eq!(selected.label, "当前输入：custom-embedding");
}

#[test]
fn knowledge_embedding_model_selected_uses_shared_when_current_is_empty() {
    let mut app = test_app();
    app.memory_settings.embedding_provider = "openai".to_string();
    app.memory_settings.embedding_model = "text-embedding-3-small".to_string();
    app.memory_settings.embedding_dimensions = 1536;
    app.knowledge.dataset_embedding_model_input = "   ".to_string();

    let options = knowledge_embedding_model_options(&app);
    let selected = knowledge_embedding_model_selected(&app);

    assert_eq!(options.len(), 1);
    assert_eq!(selected.value, "");
    assert!(selected.label.contains("text-embedding-3-small / 1536维"));
}

#[test]
fn dataset_embedding_label_prefers_non_empty_dataset_override() {
    let mut app = test_app();
    app.memory_settings.embedding_provider = "openai".to_string();
    app.memory_settings.embedding_model = "text-embedding-3-small".to_string();

    let mut explicit = dataset("explicit");
    explicit.embedding_model = Some(" bge-large ".to_string());
    assert_eq!(dataset_embedding_model_label(&app, &explicit), "bge-large");

    let mut blank = dataset("blank");
    blank.embedding_model = Some("   ".to_string());
    assert!(dataset_embedding_model_label(&app, &blank).starts_with("记忆配置："));
}

#[test]
fn score_threshold_and_time_labels_cover_enabled_disabled_and_zero() {
    let mut enabled = dataset("enabled");
    enabled.score_threshold_enabled = true;
    enabled.score_threshold = 0.2;
    assert_eq!(score_threshold_label(&enabled), "0.20");

    let disabled = dataset("disabled");
    assert_eq!(score_threshold_label(&disabled), "关闭");
    assert_eq!(format_time(0), "-");
    assert_eq!(format_time(42), "42");
}

#[test]
fn style_helpers_cover_light_dark_active_disabled_states() {
    for theme in [Theme::Light, Theme::Dark] {
        if matches!(theme, Theme::Dark) {
            assert!(is_dark_palette(&theme));
        } else {
            assert!(!is_dark_palette(&theme));
        }

        let selected = item_style(&theme, true);
        let idle = item_style(&theme, false);
        assert!(selected.background.is_some());
        assert!(idle.background.is_some());
        assert_eq!(selected.border.width, 1.0);

        let active = option_card_style(&theme, true, true);
        let disabled = option_card_style(&theme, false, false);
        assert_eq!(active.border.width, 2.0);
        assert!(disabled.text_color.is_some());

        let chrome = chrome_style(&theme);
        assert!(chrome.background.is_some());
        assert_eq!(chrome.border.width, 1.0);

        let active_segment = segment_style(&theme, button::Status::Active, true);
        let hovered_segment = segment_style(&theme, button::Status::Hovered, false);
        assert!(active_segment.background.is_some());
        assert!(hovered_segment.background.is_some());

        for status in [
            button::Status::Active,
            button::Status::Hovered,
            button::Status::Pressed,
            button::Status::Disabled,
        ] {
            let style = knowledge_toolbar_button_style(&theme, status);
            assert!(style.background.is_some());
            assert_eq!(style.border.width, 1.0);
        }
    }
}

#[test]
fn view_builds_empty_and_notification_states() {
    let mut app = test_app();
    keep_element(super::view(&app));
    keep_element(build_hero(&app));
    keep_element(build_workspace(&app, Size::new(1280.0, 800.0)));
    keep_element(build_workspace(&app, Size::new(720.0, 640.0)));
    keep_element(build_sidebar(&app));
    keep_element(build_detail(&app, Size::new(1280.0, 800.0)));

    app.knowledge.notification = Some("ok".to_string());
    keep_element(super::view(&app));
    app.knowledge.notification_is_error = true;
    keep_element(super::view(&app));
}

#[test]
fn sidebar_builds_loading_empty_filtered_and_populated_states() {
    let mut app = test_app();
    app.knowledge.loading_datasets = true;
    keep_element(build_sidebar(&app));

    app.knowledge.loading_datasets = false;
    app.knowledge.datasets = vec![dataset("dataset-a")];
    app.knowledge.dataset_search_query = "missing".to_string();
    keep_element(build_sidebar(&app));

    app.knowledge.dataset_search_query = "DATASET".to_string();
    app.knowledge.selected_dataset_id = Some("dataset-a".to_string());
    keep_element(build_sidebar(&app));
    keep_element(dataset_item(&app, &app.knowledge.datasets[0]));
}

#[test]
fn create_dataset_controls_build_for_vector_and_rerank_capabilities() {
    let mut app = test_app();
    app.knowledge.runtime_status = Some(runtime_status(false, false));
    app.knowledge.dataset_indexing_mode = KnowledgeIndexingMode::Economy;
    app.knowledge.dataset_rerank_enabled = true;
    keep_element(build_indexing_mode_section(&app));
    keep_element(build_retrieval_mode_section(&app));
    keep_element(build_dataset_param_section(&app));
    keep_element(build_rerank_toggle(&app));

    app.knowledge.runtime_status = Some(runtime_status(true, true));
    app.knowledge.dataset_indexing_mode = KnowledgeIndexingMode::HighQuality;
    app.knowledge.dataset_retrieval_mode = KnowledgeRetrievalMode::Hybrid;
    keep_element(build_indexing_mode_section(&app));
    keep_element(build_retrieval_mode_section(&app));
    keep_element(build_dataset_param_section(&app));
    keep_element(build_embedding_model_section(&app));
}

#[test]
fn detail_builds_all_tabs_and_responsive_document_layouts() {
    let mut app = test_app();
    let mut current = dataset("dataset-a");
    current.description = "description".to_string();
    current.chunking_mode = KnowledgeChunkingMode::ParentChild;
    current.indexing_mode = KnowledgeIndexingMode::HighQuality;
    current.retrieval_mode = KnowledgeRetrievalMode::Hybrid;
    current.score_threshold_enabled = true;
    current.rerank_enabled = true;
    current.rerank_model = Some("rerank-v1".to_string());
    app.knowledge.datasets = vec![current];
    app.knowledge.selected_dataset_id = Some("dataset-a".to_string());
    app.knowledge.runtime_status = Some(runtime_status(true, true));
    app.knowledge.documents = vec![document("document-a")];

    app.knowledge.active_tab = crate::app::state::KnowledgeDetailTab::Documents;
    keep_element(build_detail(&app, Size::new(1280.0, 800.0)));
    keep_element(build_documents_tab(&app, Size::new(1280.0, 800.0)));
    keep_element(build_documents_tab(&app, Size::new(900.0, 800.0)));

    app.knowledge.active_tab = crate::app::state::KnowledgeDetailTab::Retrieval;
    keep_element(build_detail(&app, Size::new(1280.0, 800.0)));

    app.knowledge.active_tab = crate::app::state::KnowledgeDetailTab::Settings;
    keep_element(build_detail(&app, Size::new(1280.0, 800.0)));
    keep_element(build_settings_tab(&app, app.knowledge.selected_dataset().unwrap()));
}

#[test]
fn documents_table_builds_loading_empty_filtered_and_row_states() {
    let mut app = test_app();
    app.knowledge.loading_documents = true;
    keep_element(build_documents_table(&app));

    app.knowledge.loading_documents = false;
    keep_element(build_documents_table(&app));

    let mut disabled = document("document-b");
    disabled.enabled = false;
    app.knowledge.documents = vec![document("document-a"), disabled];
    app.knowledge.document_search_query = "missing".to_string();
    keep_element(build_documents_table(&app));

    app.knowledge.document_search_query = "DOCUMENT".to_string();
    keep_element(build_documents_table(&app));
    keep_element(document_row(&app, &app.knowledge.documents[0]));

    app.knowledge.deleting = true;
    keep_element(document_row(&app, &app.knowledge.documents[1]));
}

#[test]
fn retrieval_tab_builds_empty_and_result_states() {
    let mut app = test_app();
    keep_element(build_retrieval_tab(&app));

    app.knowledge.retrieve_results = vec![chunk("scored", Some(0.1234)), chunk("unscored", None)];
    keep_element(build_retrieval_tab(&app));
    keep_element(chunk_card(&app.knowledge.retrieve_results[0]));
    keep_element(chunk_card(&app.knowledge.retrieve_results[1]));

    app.knowledge.retrieving = true;
    keep_element(build_retrieval_tab(&app));
}

#[test]
fn small_element_helpers_build_without_panicking() {
    keep_element(section_header("title", "description"));
    keep_element(compact_input_row("Top K", "10", |_| {
        Message::Knowledge(crate::app::message::KnowledgeToolMessage::RetrieveTopKChanged(
            "10".to_string(),
        ))
    }));
    keep_element(option_card(
        "高质量",
        "生成向量索引。",
        crate::app::assets::Icon::Speedometer2,
        true,
        true,
        true,
        Message::Knowledge(crate::app::message::KnowledgeToolMessage::CreateDataset),
    ));
    keep_element(segment_button(
        "设置",
        true,
        Message::Knowledge(crate::app::message::KnowledgeToolMessage::ClearNotification),
    ));
    keep_element(icon_badge(crate::app::assets::Icon::Journals, true));
    keep_element(empty_hint("暂无文档"));
}
