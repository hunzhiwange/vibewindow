#[test]
fn normalize_task_model_input_uses_auto_for_blank_values() {
    assert_eq!(super::normalize_task_model_input("  "), super::TASK_MODEL_AUTO);
    assert_eq!(super::normalize_task_model_input(" gpt-5 "), "gpt-5");
}

#[test]
fn claude_model_alias_accepts_supported_aliases_only() {
    assert_eq!(super::claude_model_alias(""), Some(super::CLAUDE_DEFAULT_MODEL_ALIAS));
    assert_eq!(super::claude_model_alias("AUTO"), Some(super::CLAUDE_DEFAULT_MODEL_ALIAS));
    assert_eq!(super::claude_model_alias("sonnet"), Some("sonnet"));
    assert_eq!(super::claude_model_alias("unknown"), None);
}

#[test]
fn executor_backend_round_trips_stable_ids() {
    for backend in super::TaskExecutorBackend::all() {
        assert_eq!(super::TaskExecutorBackend::from_id(backend.id()), Some(backend));
        assert!(!backend.label().is_empty());
    }

    assert_eq!(super::TaskExecutorBackend::from_id("missing"), None);
}

#[test]
fn task_status_order_and_keys_are_stable() {
    assert_eq!(super::TaskStatus::Pool.next(), Some(super::TaskStatus::Pending));
    assert_eq!(super::TaskStatus::Paused.next(), None);
    assert_eq!(super::TaskStatus::Archived.next(), None);
    assert_eq!(
        super::TaskStatus::parse_key(super::TaskStatus::CodeComplete.to_string_key()),
        Some(super::TaskStatus::CodeComplete)
    );
}

#[test]
fn task_board_settings_sanitized_clamps_ranges() {
    let settings = super::TaskBoardSettings {
        max_concurrent: 0,
        refresh_interval_seconds: 0,
        scheduler_tick_interval_seconds: 99,
        auto_promote_tick_interval_seconds: 0,
        failed_retry_minutes: 0,
        running_timeout_minutes: 99_999,
        pr_submitted_stall_timeout_seconds: 1,
        ..super::TaskBoardSettings::default()
    }
    .sanitized();

    assert_eq!(settings.max_concurrent, 1);
    assert_eq!(settings.refresh_interval_seconds, 1);
    assert_eq!(settings.scheduler_tick_interval_seconds, 60);
    assert_eq!(settings.auto_promote_tick_interval_seconds, 1);
    assert_eq!(settings.failed_retry_minutes, 1);
    assert_eq!(settings.running_timeout_minutes, 1440);
    assert_eq!(settings.pr_submitted_stall_timeout_seconds, 5);
}

#[test]
fn task_running_duration_uses_started_time_and_saturates() {
    let mut task = super::Task::new(10);
    task.status = super::TaskStatus::Running;
    task.execution_started_at_ms = Some(100);

    assert_eq!(task.running_duration_ms(150), Some(50));
    assert_eq!(task.running_duration_ms(50), Some(0));

    task.status = super::TaskStatus::Paused;
    assert_eq!(task.running_duration_ms(150), None);
}

#[test]
fn task_index_initializes_every_status_bucket() {
    let index = super::TaskIndex::new();

    for status in super::TaskStatus::all() {
        assert!(index.order_by_status.contains_key(status.to_string_key()));
    }
}
