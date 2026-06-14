use super::*;

#[test]
fn test_module_is_wired() {
    let module = std::hint::black_box(module_path!());

    assert!(module.ends_with("provider_model_tests"));
}

#[test]
fn provider_connect_header_and_custom_drafts_have_empty_defaults() {
    let connect = ProviderConnectState::default();
    assert!(connect.provider_id.is_empty());
    assert!(connect.provider_name.is_empty());
    assert!(connect.api_key.is_empty());

    let header = ProviderHeaderDraft::default();
    assert!(header.key.is_empty());
    assert!(header.value.is_empty());

    let model = CustomProviderModelDraft::default();
    assert!(model.model_id.is_empty());
    assert!(model.display_name.is_empty());

    let provider = CustomProviderDraft::default();
    assert!(provider.provider_id.is_empty());
    assert!(provider.display_name.is_empty());
    assert!(provider.base_url.is_empty());
    assert!(provider.api_key.is_empty());
    assert_eq!(provider.headers.len(), 1);
    assert_eq!(provider.models.len(), 1);
}

#[test]
fn provider_settings_default_populates_popular_patterns_and_modal_state() {
    let state = ProviderSettingsState::default();

    assert!(!state.loading);
    assert!(!state.models_syncing);
    assert_eq!(state.models_sync_progress, 0.0);
    assert!(state.models_sync_label.is_empty());
    assert!(state.providers.is_empty());
    assert_eq!(
        state.popular_patterns,
        vec![
            "OpenCode Zen".to_string(),
            "Anthropic".to_string(),
            "GitHub Copilot".to_string(),
            "OpenAI".to_string(),
            "Google".to_string(),
            "OpenRouter".to_string(),
            "Vercel AI Gateway".to_string(),
        ]
    );
    assert!(!state.catalog_loading);
    assert!(!state.catalog_open);
    assert!(state.catalog_query.is_empty());
    assert!(state.catalog_items.is_empty());
    assert!(state.connect_modal.is_none());
    assert!(state.connect_error.is_none());
    assert!(state.disconnect_confirm_provider_id.is_none());
    assert!(!state.custom_provider_modal_open);
    assert!(state.custom_editing_provider_id.is_none());
    assert!(state.custom_model_modal.is_none());
    assert!(state.save_error.is_none());
}

#[test]
fn provider_and_model_summary_structs_preserve_public_fields() {
    let provider = ProviderSummary {
        id: "openai".to_string(),
        name: "OpenAI".to_string(),
        source_label: "config".to_string(),
        connected: true,
    };
    assert_eq!(provider.id, "openai");
    assert_eq!(provider.name, "OpenAI");
    assert_eq!(provider.source_label, "config");
    assert!(provider.connected);

    let catalog = ModelCatalogEntry {
        provider_id: "anthropic".to_string(),
        provider_name: "Anthropic".to_string(),
        model_id: "claude".to_string(),
        model_name: "Claude".to_string(),
    };
    assert_eq!(catalog.provider_id, "anthropic");
    assert_eq!(catalog.provider_name, "Anthropic");
    assert_eq!(catalog.model_id, "claude");
    assert_eq!(catalog.model_name, "Claude");

    let model = ModelSummary {
        id: "gpt".to_string(),
        name: "GPT".to_string(),
        enabled: true,
        toolcall: true,
        attachment: false,
        context_limit: 128_000,
        detail: serde_json::json!({"family": "test"}),
    };
    assert_eq!(model.id, "gpt");
    assert_eq!(model.name, "GPT");
    assert!(model.enabled);
    assert!(model.toolcall);
    assert!(!model.attachment);
    assert_eq!(model.context_limit, 128_000);
    assert_eq!(model.detail["family"], "test");

    let provider_models = ProviderModelsSummary {
        id: "openai".to_string(),
        name: "OpenAI".to_string(),
        models: vec![model],
    };
    assert_eq!(provider_models.models.len(), 1);
}

#[test]
fn model_detail_and_model_settings_defaults_are_stable() {
    let row = ModelDetailRow { label: "Context".to_string(), value: "128k".to_string() };
    let modal = ModelDetailModalState {
        provider_id: "openai".to_string(),
        provider_name: "OpenAI".to_string(),
        model_id: "gpt".to_string(),
        model_name: "GPT".to_string(),
        rows: vec![row],
        raw_json: "{}".to_string(),
        show_raw: true,
    };
    assert_eq!(modal.provider_id, "openai");
    assert_eq!(modal.rows[0].label, "Context");
    assert_eq!(modal.rows[0].value, "128k");
    assert!(modal.show_raw);

    let state = ModelSettingsState::default();
    assert!(!state.loading);
    assert!(state.query.is_empty());
    assert!(state.providers.is_empty());
    assert!(state.save_error.is_none());
    assert!(state.detail_modal.is_none());
}

#[test]
fn route_classification_heartbeat_and_goal_loop_defaults_are_ready_to_edit() {
    let embedding = EmbeddingRouteDraft::default();
    assert!(embedding.pattern.is_empty());
    assert!(embedding.provider.is_empty());
    assert!(embedding.model.is_empty());
    assert!(embedding.dimensions.is_empty());
    assert!(embedding.api_key_input.is_empty());
    assert!(EmbeddingRoutesSettingsState::default().routes.is_empty());

    let route = ModelRoute::default();
    assert!(route.pattern.is_empty());
    assert!(route.provider.is_empty());
    assert!(route.model.is_empty());
    assert!(route.priority_input.is_empty());
    assert!(ModelRoutesSettingsState::default().routes.is_empty());

    let rule = QueryClassificationRuleInput::default();
    assert!(rule.pattern.is_empty());
    assert!(rule.category.is_empty());
    assert_eq!(rule.priority_input, "0");
    assert!(!QueryClassificationSettingsState::default().enabled);

    let heartbeat = HeartbeatSettingsState::default();
    assert!(!heartbeat.enabled);
    assert_eq!(heartbeat.interval_minutes, 30);
    assert!(heartbeat.message_input.is_empty());
    assert!(heartbeat.target_input.is_empty());
    assert!(heartbeat.to_input.is_empty());
    assert!(!heartbeat.show_help_modal);

    let goal_loop = GoalLoopSettingsState::default();
    assert!(!goal_loop.enabled);
    assert_eq!(goal_loop.interval_minutes_input, "10");
    assert_eq!(goal_loop.step_timeout_secs_input, "120");
    assert_eq!(goal_loop.max_steps_per_cycle_input, "3");
    assert!(goal_loop.channel_input.is_empty());
    assert!(goal_loop.target_input.is_empty());
}

#[test]
fn cron_tabs_job_types_and_draft_defaults_match_api_contract() {
    assert_eq!(CronSettingsTab::default(), CronSettingsTab::Jobs);
    assert_eq!(CronAddJobType::default(), CronAddJobType::Shell);
    assert_eq!(CronAddJobType::Shell.as_api_value(), "shell");
    assert_eq!(CronAddJobType::Agent.as_api_value(), "agent");
    assert_eq!(CronAddScheduleKind::default(), CronAddScheduleKind::Cron);
    assert_eq!(CronAddScheduleKind::Cron.as_api_value(), "cron");
    assert_eq!(CronAddScheduleKind::At.as_api_value(), "at");
    assert_eq!(CronAddScheduleKind::Every.as_api_value(), "every");

    let draft = CronJobDraft::default();
    assert!(draft.name.is_empty());
    assert_eq!(draft.job_type, CronAddJobType::Shell);
    assert_eq!(draft.schedule_kind, CronAddScheduleKind::Cron);
    assert_eq!(draft.session_target, "isolated");
    assert_eq!(draft.agent, "main");
    assert!(draft.command_editor.text().is_empty());
    assert!(draft.prompt_editor.text().is_empty());
    assert!(!draft.wake);
    assert!(!draft.full_access);
    assert!(!draft.task_pool);
    assert!(!draft.delivery_enabled);
    assert!(draft.delivery_best_effort);
    assert!(!draft.delete_after_run);

    let state = CronSettingsState::default();
    assert!(state.enabled);
    assert_eq!(state.max_run_history, 50);
    assert_eq!(state.active_tab, CronSettingsTab::Jobs);
    assert!(!state.jobs_loading);
    assert!(state.jobs.is_empty());
    assert!(state.selected_job_ids.is_empty());
    assert!(state.editing_job_id.is_none());
    assert!(state.runs_modal_job_id.is_none());
    assert!(!state.runs_modal_loading);
    assert!(state.runs_modal_error.is_none());
    assert!(state.runs_modal.is_empty());
    assert!(state.runs_modal_editor.text().is_empty());
    assert!(!state.show_help_modal);
    assert!(state.save_error.is_none());
    assert!(state.action_status.is_none());
}
