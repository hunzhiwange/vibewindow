use super::*;

fn dataset(id: &str, top_k: usize, threshold_enabled: bool, threshold: f64) -> KnowledgeDatasetDto {
    KnowledgeDatasetDto {
        id: id.to_string(),
        name: format!("Dataset {id}"),
        description: String::new(),
        chunking_mode: KnowledgeChunkingMode::General,
        indexing_mode: KnowledgeIndexingMode::Economy,
        retrieval_mode: KnowledgeRetrievalMode::FullText,
        keyword_count: 10,
        top_k,
        score_threshold_enabled: threshold_enabled,
        score_threshold: threshold,
        rerank_enabled: false,
        embedding_model: None,
        rerank_model: None,
        document_count: 0,
        chunk_count: 0,
        created_at_ms: 0,
        updated_at_ms: 0,
    }
}

#[test]
fn detail_tabs_expose_expected_titles_and_default() {
    assert_eq!(KnowledgeDetailTab::default(), KnowledgeDetailTab::Documents);
    assert_eq!(
        KnowledgeDetailTab::ALL,
        [
            KnowledgeDetailTab::Documents,
            KnowledgeDetailTab::Retrieval,
            KnowledgeDetailTab::Settings
        ]
    );
    assert_eq!(KnowledgeDetailTab::Documents.title(), "文档");
    assert_eq!(KnowledgeDetailTab::Retrieval.title(), "召回测试");
    assert_eq!(KnowledgeDetailTab::Settings.title(), "设置");
}

#[test]
fn default_state_matches_empty_idle_ui_defaults() {
    let state = KnowledgeUiState::default();

    assert!(state.datasets.is_empty());
    assert!(state.documents.is_empty());
    assert!(state.retrieve_results.is_empty());
    assert!(state.runtime_status.is_none());
    assert!(state.selected_dataset_id.is_none());
    assert_eq!(state.active_tab, KnowledgeDetailTab::Documents);
    assert_eq!(state.dataset_chunking_mode, KnowledgeChunkingMode::General);
    assert_eq!(state.dataset_indexing_mode, KnowledgeIndexingMode::Economy);
    assert_eq!(state.dataset_retrieval_mode, KnowledgeRetrievalMode::FullText);
    assert_eq!(state.dataset_keyword_count_input, "10");
    assert_eq!(state.dataset_top_k_input, "10");
    assert_eq!(state.dataset_score_threshold_input, "0.15");
    assert_eq!(state.retrieve_top_k_input, "10");
    assert_eq!(state.retrieve_score_threshold_input, "0.15");
    assert!(!state.dataset_score_threshold_enabled);
    assert!(!state.dataset_rerank_enabled);
    assert!(!state.is_busy());
    assert!(state.notification.is_none());
    assert!(!state.notification_is_error);
}

#[test]
fn selected_dataset_returns_matching_dataset_only() {
    let mut state = KnowledgeUiState {
        datasets: vec![dataset("first", 10, false, 0.15), dataset("second", 12, true, 0.8)],
        selected_dataset_id: Some("second".to_string()),
        ..Default::default()
    };

    assert_eq!(state.selected_dataset().map(|dataset| dataset.id.as_str()), Some("second"));

    state.selected_dataset_id = Some("missing".to_string());
    assert!(state.selected_dataset().is_none());

    state.selected_dataset_id = None;
    assert!(state.selected_dataset().is_none());
}

#[test]
fn busy_state_tracks_all_loading_and_mutating_flags() {
    let idle = KnowledgeUiState::default();
    assert!(!idle.is_busy());

    let mut checks: Vec<Box<dyn Fn(&mut KnowledgeUiState)>> = vec![
        Box::new(|state| state.loading_status = true),
        Box::new(|state| state.loading_datasets = true),
        Box::new(|state| state.loading_documents = true),
        Box::new(|state| state.creating_dataset = true),
        Box::new(|state| state.creating_document = true),
        Box::new(|state| state.deleting = true),
        Box::new(|state| state.retrieving = true),
    ];

    for set_busy in checks.drain(..) {
        let mut state = KnowledgeUiState::default();
        set_busy(&mut state);
        assert!(state.is_busy());
    }
}

#[test]
fn select_first_dataset_keeps_valid_selection_or_falls_back() {
    let mut state = KnowledgeUiState {
        datasets: vec![dataset("first", 10, false, 0.15), dataset("second", 12, true, 0.8)],
        selected_dataset_id: Some("second".to_string()),
        ..Default::default()
    };

    state.select_first_dataset_if_needed();
    assert_eq!(state.selected_dataset_id.as_deref(), Some("second"));

    state.selected_dataset_id = Some("missing".to_string());
    state.select_first_dataset_if_needed();
    assert_eq!(state.selected_dataset_id.as_deref(), Some("first"));

    state.datasets.clear();
    state.select_first_dataset_if_needed();
    assert!(state.selected_dataset_id.is_none());
}

#[test]
fn sync_retrieve_defaults_clamps_selected_dataset_values() {
    let mut state = KnowledgeUiState {
        datasets: vec![dataset("low", 0, true, -0.5), dataset("high", 500, false, 2.0)],
        selected_dataset_id: Some("low".to_string()),
        retrieve_top_k_input: "unchanged".to_string(),
        retrieve_score_threshold_input: "unchanged".to_string(),
        ..Default::default()
    };

    state.sync_retrieve_defaults_from_selected_dataset();
    assert_eq!(state.retrieve_top_k_input, "1");
    assert!(state.retrieve_score_threshold_enabled);
    assert_eq!(state.retrieve_score_threshold_input, "0.0");

    state.selected_dataset_id = Some("high".to_string());
    state.sync_retrieve_defaults_from_selected_dataset();
    assert_eq!(state.retrieve_top_k_input, "50");
    assert!(!state.retrieve_score_threshold_enabled);
    assert_eq!(state.retrieve_score_threshold_input, "1.0");
}

#[test]
fn sync_retrieve_defaults_leaves_inputs_when_no_dataset_is_selected() {
    let mut state = KnowledgeUiState {
        datasets: vec![dataset("first", 10, false, 0.15)],
        selected_dataset_id: Some("missing".to_string()),
        retrieve_top_k_input: "7".to_string(),
        retrieve_score_threshold_enabled: true,
        retrieve_score_threshold_input: "0.7".to_string(),
        ..Default::default()
    };

    state.sync_retrieve_defaults_from_selected_dataset();

    assert_eq!(state.retrieve_top_k_input, "7");
    assert!(state.retrieve_score_threshold_enabled);
    assert_eq!(state.retrieve_score_threshold_input, "0.7");
}

#[test]
fn score_threshold_formatting_removes_redundant_zeroes_but_keeps_decimal() {
    assert_eq!(format_score_threshold(0.0), "0.0");
    assert_eq!(format_score_threshold(1.0), "1.0");
    assert_eq!(format_score_threshold(0.5), "0.5");
    assert_eq!(format_score_threshold(0.15), "0.15");
    assert_eq!(format_score_threshold(0.156), "0.16");
}
