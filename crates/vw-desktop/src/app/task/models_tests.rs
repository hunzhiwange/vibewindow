#[test]
fn test_module_is_wired() {
    let module = std::hint::black_box(module_path!());

    assert!(module.ends_with("models_tests"));
}

#[test]
fn normalize_model_and_agent_inputs_trim_and_map_defaults() {
    assert_eq!(super::normalize_task_model_input("  "), super::TASK_MODEL_AUTO);
    assert_eq!(super::normalize_task_model_input(" gpt-5 "), "gpt-5");

    assert_eq!(super::claude_model_alias(""), Some(super::CLAUDE_DEFAULT_MODEL_ALIAS));
    assert_eq!(super::claude_model_alias(" AUTO "), Some(super::CLAUDE_DEFAULT_MODEL_ALIAS));
    assert_eq!(super::claude_model_alias("Sonnet"), Some("sonnet"));
    assert_eq!(super::claude_model_alias("unknown"), None);

    assert_eq!(super::normalize_task_acp_agent_input(" default "), None);
    assert_eq!(super::normalize_task_acp_agent_input("ACP 网关"), None);
    assert_eq!(
        super::normalize_task_acp_agent_input(" custom-agent "),
        Some("custom-agent".into())
    );
}

#[test]
fn executor_backend_ids_labels_and_legacy_mapping_are_stable() {
    let backends = super::TaskExecutorBackend::all();
    assert_eq!(backends.len(), 4);
    assert_eq!(super::TaskExecutorBackend::Internal.id(), "internal");
    assert_eq!(super::TaskExecutorBackend::OpenCode.label(), "OpenCode");
    assert_eq!(
        super::TaskExecutorBackend::from_id("Claude Code"),
        Some(super::TaskExecutorBackend::Claude)
    );
    assert_eq!(super::TaskExecutorBackend::from_id("missing"), None);

    assert_eq!(
        super::legacy_executor_to_task_acp_agent(super::TaskExecutorBackend::Internal),
        None
    );
    assert_eq!(
        super::legacy_executor_to_task_acp_agent(super::TaskExecutorBackend::Codex),
        Some("codex".into())
    );
}

#[test]
fn task_status_keys_labels_and_next_states_cover_all_variants() {
    let statuses = super::TaskStatus::all();
    assert_eq!(statuses.len(), 11);
    for status in statuses {
        let key = status.to_string_key();
        assert_eq!(super::TaskStatus::parse_key(key), Some(status));
        assert!(!status.label().is_empty());
    }

    assert_eq!(super::TaskStatus::Pool.next(), Some(super::TaskStatus::Pending));
    assert_eq!(super::TaskStatus::Failed.next(), Some(super::TaskStatus::Pending));
    assert_eq!(super::TaskStatus::Paused.next(), None);
    assert_eq!(super::TaskStatus::Archived.next(), None);
    assert_eq!(super::TaskStatus::parse_key("bogus"), None);
}

#[test]
fn subtask_execution_lifecycle_tracks_status_and_duration() {
    let mut subtask = super::SubTask::new("write tests".into());
    assert!(subtask.id.starts_with("SUB-"));
    assert_eq!(subtask.status, super::SubTaskStatus::Pending);
    assert_eq!(subtask.display_execution_duration_ms(subtask.created_at_ms + 10), None);

    subtask.start_execution();
    let started_at = subtask.execution_started_at_ms.expect("start timestamp");
    assert_eq!(subtask.status, super::SubTaskStatus::Running);
    assert_eq!(subtask.display_execution_duration_ms(started_at + 25), Some(25));

    subtask.mark_completed();
    assert_eq!(subtask.status, super::SubTaskStatus::Completed);
    assert!(subtask.completed);
    assert!(subtask.last_execution_duration_ms.is_some());

    subtask.start_execution();
    subtask.mark_failed();
    assert_eq!(subtask.status, super::SubTaskStatus::Failed);
    assert!(!subtask.completed);
}

#[test]
fn task_lifecycle_clears_and_sets_execution_fields() {
    let mut task = super::Task::new(7);
    assert_eq!(task.priority, 7);
    assert_eq!(task.status, super::TaskStatus::Pool);
    assert_eq!(task.logs.len(), 1);

    task.last_error = Some("previous".into());
    task.start_execution("run now".into());
    assert_eq!(task.status, super::TaskStatus::Running);
    assert_eq!(task.retry_count, 1);
    assert!(task.execution_started_at_ms.is_some());
    assert!(task.logs.iter().any(|log| log.message == "run now"));

    task.mark_execution_failed("boom".into());
    assert_eq!(task.status, super::TaskStatus::Failed);
    assert_eq!(task.last_error.as_deref(), Some("boom"));
    assert!(task.last_execution_duration_ms.is_some());
    assert!(task.logs.iter().any(|log| log.message == "失败原因: boom"));

    task.start_execution("retry".into());
    task.mark_execution_succeeded();
    task.set_status(super::TaskStatus::CodeComplete);
    assert_eq!(task.last_error, None);
    assert_eq!(task.pause_reason, None);
    assert_eq!(task.status, super::TaskStatus::CodeComplete);
}

#[test]
fn task_pause_merge_and_duration_helpers_use_expected_fallbacks() {
    let mut task = super::Task::default();
    assert!(!task.should_auto_merge());
    task.merge_source_branch = Some(" feature ".into());
    task.merge_target_branch = Some("main".into());
    assert!(task.should_auto_merge());

    task.start_merge_execution("merge".into());
    assert!(task.execution_started_at_ms.is_some());

    task.mark_paused("waiting".into());
    assert_eq!(task.status, super::TaskStatus::Paused);
    assert_eq!(task.pause_reason.as_deref(), Some("waiting"));
    assert_eq!(task.running_duration_ms(super::now_ms()), None);
    assert!(task.display_execution_duration_ms(super::now_ms()).is_some());

    task.status = super::TaskStatus::Running;
    task.execution_started_at_ms = None;
    task.last_active_at_ms = 100;
    assert_eq!(task.running_duration_ms(125), Some(25));
    assert_eq!(task.running_duration_ms(75), Some(0));
}

#[test]
fn task_index_and_board_settings_defaults_are_sanitized() {
    let index = super::TaskIndex::new();
    assert_eq!(index.order_by_status.len(), super::TaskStatus::all().len());
    assert!(index.order_by_status.contains_key("pending"));

    let settings = super::TaskBoardSettings {
        max_concurrent: 99,
        refresh_interval_seconds: 0,
        scheduler_tick_interval_seconds: 99,
        auto_promote_tick_interval_seconds: 0,
        failed_retry_minutes: 0,
        running_timeout_minutes: 9999,
        pr_submitted_stall_timeout_seconds: 1,
        ..super::TaskBoardSettings::new()
    }
    .sanitized();

    assert_eq!(settings.max_concurrent, 10);
    assert_eq!(settings.refresh_interval_seconds, 1);
    assert_eq!(settings.scheduler_tick_interval_seconds, 60);
    assert_eq!(settings.auto_promote_tick_interval_seconds, 1);
    assert_eq!(settings.failed_retry_minutes, 1);
    assert_eq!(settings.running_timeout_minutes, 1440);
    assert_eq!(settings.pr_submitted_stall_timeout_seconds, 5);
}

#[test]
fn task_draft_defaults_match_task_creation_defaults() {
    let draft = super::TaskDraft::default();
    assert_eq!(draft.priority, "999");
    assert_eq!(draft.assignee, "VibeWindow");
    assert_eq!(draft.model, super::TASK_MODEL_AUTO);
    assert_eq!(draft.agent.as_deref(), Some(super::TASK_AGENT_MAIN));
    assert_eq!(draft.subtasks.len(), 3);
    assert_eq!(draft.auto_promote_delay_seconds, "0");
}
