#[test]
fn test_module_is_wired() {
    let module = module_path!();

    assert!(module.ends_with("edit_tests"));
}

fn app() -> crate::app::App {
    crate::app::App::new().0
}

fn meta(path: &str, name: &str) -> crate::app::state::RecentProjectMeta {
    crate::app::state::RecentProjectMeta {
        path: path.to_string(),
        name: name.to_string(),
        task_board_settings: None,
        session_auto_refresh: true,
        session_refresh_interval_seconds: 120,
        icon: Some("rocket".to_string()),
        icon_color: Some("#336699".to_string()),
        worktree_start_command: Some("pnpm dev".to_string()),
    }
}

#[test]
fn tools_menu_toggles_and_closes() {
    let mut app = app();
    let path = "/tmp/project".to_string();

    let _ = super::handle(
        &mut app,
        crate::app::message::project::ProjectMessage::ProjectToolsMenuToggled(path.clone()),
    );
    assert_eq!(app.project_tools_menu_path, Some(path.clone()));

    let _ = super::handle(
        &mut app,
        crate::app::message::project::ProjectMessage::ProjectToolsMenuToggled(path),
    );
    assert!(app.project_tools_menu_path.is_none());

    app.project_tools_menu_path = Some("/tmp/project".to_string());
    let _ = super::handle(
        &mut app,
        crate::app::message::project::ProjectMessage::ProjectToolsMenuClosed,
    );
    assert!(app.project_tools_menu_path.is_none());
}

#[test]
fn edit_opened_prefers_recent_edit_and_loads_meta_fields() {
    let mut app = app();
    let path = "/tmp/project".to_string();
    app.recent_projects = vec![path.clone()];
    app.recent_projects_edits = vec!["Edited Name".to_string()];
    app.recent_projects_meta = vec![meta(&path, "Meta Name")];
    app.project_worktree_enabled.insert(path.clone(), true);
    app.project_tools_menu_path = Some(path.clone());

    let _ = super::handle(
        &mut app,
        crate::app::message::project::ProjectMessage::ProjectEditOpened(path.clone()),
    );

    assert_eq!(app.project_edit_path, Some(path));
    assert_eq!(app.project_edit_tab, crate::app::state::ProjectEditTab::General);
    assert_eq!(app.project_edit_name, "Edited Name");
    assert_eq!(app.project_edit_icon, "rocket");
    assert_eq!(app.project_edit_icon_color, "#336699");
    assert_eq!(app.project_edit_start_script, "pnpm dev");
    assert!(app.project_edit_worktree_enabled);
    assert_eq!(app.project_edit_session_refresh_interval_seconds_input, "120");
    assert!(app.project_tools_menu_path.is_none());
}

#[test]
fn edit_opened_falls_back_to_meta_name_then_path_file_name() {
    let mut app = app();
    let path = "/tmp/project-a".to_string();
    app.recent_projects_meta = vec![meta(&path, "Meta Name")];

    let _ = super::handle(
        &mut app,
        crate::app::message::project::ProjectMessage::ProjectEditOpened(path.clone()),
    );
    assert_eq!(app.project_edit_name, "Meta Name");

    app.recent_projects_meta.clear();
    let _ = super::handle(
        &mut app,
        crate::app::message::project::ProjectMessage::ProjectEditOpened(
            "/tmp/fallback-project".to_string(),
        ),
    );
    assert_eq!(app.project_edit_name, "fallback-project");
}

#[test]
fn edit_simple_field_messages_update_state() {
    let mut app = app();

    let _ = super::handle(
        &mut app,
        crate::app::message::project::ProjectMessage::ProjectEditTabSelected(
            crate::app::state::ProjectEditTab::Scheduling,
        ),
    );
    let _ = super::handle(
        &mut app,
        crate::app::message::project::ProjectMessage::ProjectEditNameChanged(
            "Name".to_string(),
        ),
    );
    let _ = super::handle(
        &mut app,
        crate::app::message::project::ProjectMessage::ProjectEditIconChanged("icon".to_string()),
    );
    let _ = super::handle(
        &mut app,
        crate::app::message::project::ProjectMessage::ProjectEditIconHovered(true),
    );
    let _ = super::handle(
        &mut app,
        crate::app::message::project::ProjectMessage::ProjectEditIconColorChanged(
            "#112233".to_string(),
        ),
    );
    let _ = super::handle(
        &mut app,
        crate::app::message::project::ProjectMessage::ProjectEditIconColorPickerToggled,
    );
    let _ = super::handle(
        &mut app,
        crate::app::message::project::ProjectMessage::ProjectEditStartScriptChanged(
            "cargo run".to_string(),
        ),
    );
    let _ = super::handle(
        &mut app,
        crate::app::message::project::ProjectMessage::ProjectEditWorktreeToggled(true),
    );

    assert_eq!(app.project_edit_tab, crate::app::state::ProjectEditTab::Scheduling);
    assert_eq!(app.project_edit_name, "Name");
    assert_eq!(app.project_edit_icon, "icon");
    assert!(app.project_edit_icon_hovered);
    assert_eq!(app.project_edit_icon_color, "#112233");
    assert!(app.project_edit_icon_color_picker_open);
    assert_eq!(app.project_edit_start_script, "cargo run");
    assert!(app.project_edit_worktree_enabled);

    let _ = super::handle(
        &mut app,
        crate::app::message::project::ProjectMessage::ProjectEditIconColorPickerClosed,
    );
    assert!(!app.project_edit_icon_color_picker_open);
}

#[test]
fn edit_color_format_preset_editor_action_and_recycle_toggle_update_state() {
    let mut app = app();
    app.project_edit_start_script = "first".to_string();
    app.project_edit_start_script_editor = iced::widget::text_editor::Content::with_text("first");

    let _ = super::handle(
        &mut app,
        crate::app::message::project::ProjectMessage::ProjectEditIconColorPresetSelected(
            "#abcdef".to_string(),
        ),
    );
    let _ = super::handle(
        &mut app,
        crate::app::message::project::ProjectMessage::ProjectEditIconColorFormatChanged(
            crate::app::views::design::models::ColorFormat::Rgba,
        ),
    );
    let _ = super::handle(
        &mut app,
            crate::app::message::project::ProjectMessage::ProjectEditStartScriptEditorAction(
                iced::widget::text_editor::Action::Edit(iced::widget::text_editor::Edit::Paste(
                    std::sync::Arc::new(" line".to_string()),
            )),
        ),
    );
    let _ = super::handle(
        &mut app,
        crate::app::message::project::ProjectMessage::ProjectEditRecycleWorktreeOnTaskFinishToggled(
            true,
        ),
    );

    assert_eq!(app.project_edit_icon_color, "#abcdef");
    assert_eq!(
        app.project_edit_icon_color_format,
        crate::app::views::design::models::ColorFormat::Rgba
    );
    assert!(app.project_edit_start_script.contains("line"));
    assert!(app.project_edit_task_board_settings.recycle_worktree_on_task_finish);
}

#[test]
fn edit_icon_file_picked_ignores_empty_and_accepts_trimmed_path() {
    let mut app = app();
    app.project_edit_icon = "old".to_string();

    let _ = super::handle(
        &mut app,
        crate::app::message::project::ProjectMessage::ProjectEditIconFilePicked(None),
    );
    assert_eq!(app.project_edit_icon, "old");

    let _ = super::handle(
        &mut app,
        crate::app::message::project::ProjectMessage::ProjectEditIconFilePicked(Some(
            "   ".to_string(),
        )),
    );
    assert_eq!(app.project_edit_icon, "old");

    app.project_edit_icon_hovered = true;
    let _ = super::handle(
        &mut app,
        crate::app::message::project::ProjectMessage::ProjectEditIconFilePicked(Some(
            " /tmp/icon.png ".to_string(),
        )),
    );
    assert_eq!(app.project_edit_icon, "/tmp/icon.png");
    assert!(!app.project_edit_icon_hovered);
}

#[test]
fn edit_toggles_and_numeric_changes_clamp_settings() {
    let mut app = app();

    let _ = super::handle(
        &mut app,
        crate::app::message::project::ProjectMessage::ProjectEditAutoPromotePoolTasksToggled(true),
    );
    assert!(app.project_edit_task_board_settings.auto_promote_pool_tasks);
    assert!(app.project_edit_task_board_settings.auto_execute);

    let _ = super::handle(
        &mut app,
        crate::app::message::project::ProjectMessage::ProjectEditTaskBoardAutoRefreshToggled(false),
    );
    let _ = super::handle(
        &mut app,
        crate::app::message::project::ProjectMessage::ProjectEditSessionAutoRefreshToggled(false),
    );
    let _ = super::handle(
        &mut app,
        crate::app::message::project::ProjectMessage::ProjectEditCodeReviewToggled(true),
    );

    assert!(!app.project_edit_task_board_auto_refresh);
    assert!(!app.project_edit_task_board_settings.auto_refresh);
    assert!(!app.project_edit_session_auto_refresh);
    assert!(app.project_edit_task_board_settings.code_review_enabled);

    let messages = [
        crate::app::message::project::ProjectMessage::ProjectEditMaxConcurrentChanged(99),
        crate::app::message::project::ProjectMessage::ProjectEditSessionRefreshIntervalSecondsChanged(0),
        crate::app::message::project::ProjectMessage::ProjectEditTaskBoardRefreshIntervalSecondsChanged(9999),
        crate::app::message::project::ProjectMessage::ProjectEditTaskBoardSchedulerTickIntervalSecondsChanged(99),
        crate::app::message::project::ProjectMessage::ProjectEditTaskBoardAutoPromoteTickIntervalSecondsChanged(0),
        crate::app::message::project::ProjectMessage::ProjectEditFailedRetryMinutesChanged(9999),
        crate::app::message::project::ProjectMessage::ProjectEditRunningTimeoutMinutesChanged(0),
        crate::app::message::project::ProjectMessage::ProjectEditPrSubmittedStallTimeoutSecondsChanged(1),
    ];
    for message in messages {
        let _ = super::handle(&mut app, message);
    }

    assert_eq!(app.project_edit_task_board_settings.max_concurrent, 10);
    assert_eq!(app.project_edit_session_refresh_interval_seconds_input, "1");
    assert_eq!(app.project_edit_task_board_settings.refresh_interval_seconds, 3600);
    assert_eq!(app.project_edit_task_board_settings.scheduler_tick_interval_seconds, 60);
    assert_eq!(app.project_edit_task_board_settings.auto_promote_tick_interval_seconds, 1);
    assert_eq!(app.project_edit_task_board_settings.failed_retry_minutes, 1440);
    assert_eq!(app.project_edit_task_board_settings.running_timeout_minutes, 1);
    assert_eq!(app.project_edit_task_board_settings.pr_submitted_stall_timeout_seconds, 5);
}

#[test]
fn edit_input_messages_preserve_raw_text() {
    let mut app = app();

    let inputs = [
        crate::app::message::project::ProjectMessage::ProjectEditMaxConcurrentInputChanged("x".to_string()),
        crate::app::message::project::ProjectMessage::ProjectEditSessionRefreshIntervalSecondsInputChanged("2".to_string()),
        crate::app::message::project::ProjectMessage::ProjectEditTaskBoardRefreshIntervalSecondsInputChanged("3".to_string()),
        crate::app::message::project::ProjectMessage::ProjectEditTaskBoardSchedulerTickIntervalSecondsInputChanged("4".to_string()),
        crate::app::message::project::ProjectMessage::ProjectEditTaskBoardAutoPromoteTickIntervalSecondsInputChanged("5".to_string()),
        crate::app::message::project::ProjectMessage::ProjectEditFailedRetryMinutesInputChanged("6".to_string()),
        crate::app::message::project::ProjectMessage::ProjectEditRunningTimeoutMinutesInputChanged("7".to_string()),
        crate::app::message::project::ProjectMessage::ProjectEditPrSubmittedStallTimeoutSecondsInputChanged("8".to_string()),
    ];
    for message in inputs {
        let _ = super::handle(&mut app, message);
    }

    assert_eq!(app.project_edit_max_concurrent_input, "x");
    assert_eq!(app.project_edit_session_refresh_interval_seconds_input, "2");
    assert_eq!(app.project_edit_task_board_refresh_interval_seconds_input, "3");
    assert_eq!(app.project_edit_task_board_scheduler_tick_interval_seconds_input, "4");
    assert_eq!(app.project_edit_task_board_auto_promote_tick_interval_seconds_input, "5");
    assert_eq!(app.project_edit_failed_retry_minutes_input, "6");
    assert_eq!(app.project_edit_running_timeout_minutes_input, "7");
    assert_eq!(app.project_edit_pr_submitted_stall_timeout_seconds_input, "8");
}

#[test]
fn edit_save_without_path_is_noop_and_runtime_error_sets_error_message() {
    let mut app = app();

    let task = super::handle(
        &mut app,
        crate::app::message::project::ProjectMessage::ProjectEditSaved,
    );
    assert!(task.is_some());
    assert!(app.project_edit_path.is_none());

    let _ = super::handle(
        &mut app,
        crate::app::message::project::ProjectMessage::ProjectEditRuntimeSaved(Err(
            "gateway".to_string(),
        )),
    );
    assert_eq!(app.error_message.as_deref(), Some("保存项目扩展配置失败: gateway"));
}

#[test]
fn edit_save_updates_meta_worktree_and_resets_editor_state() {
    let mut app = app();
    let path = "/tmp/project".to_string();
    app.project_path = Some(path.clone());
    app.recent_projects = vec![path.clone()];
    app.recent_projects_edits = vec!["Old".to_string()];
    app.project_edit_path = Some(path.clone());
    app.project_edit_name = "  ".to_string();
    app.project_edit_icon = " icon ".to_string();
    app.project_edit_icon_color = " #123456 ".to_string();
    app.project_edit_start_script = " pnpm dev ".to_string();
    app.project_edit_worktree_enabled = true;
    app.project_edit_max_concurrent_input = "99".to_string();
    app.project_edit_task_board_auto_refresh = false;
    app.project_edit_session_auto_refresh = false;
    app.project_edit_session_refresh_interval_seconds_input = "9999".to_string();
    app.project_edit_task_board_refresh_interval_seconds_input = "bad".to_string();
    app.project_edit_task_board_scheduler_tick_interval_seconds_input = "0".to_string();
    app.project_edit_task_board_auto_promote_tick_interval_seconds_input = "9999".to_string();
    app.project_edit_failed_retry_minutes_input = "0".to_string();
    app.project_edit_running_timeout_minutes_input = "9999".to_string();
    app.project_edit_pr_submitted_stall_timeout_seconds_input = "1".to_string();

    let task = super::handle(
        &mut app,
        crate::app::message::project::ProjectMessage::ProjectEditSaved,
    );

    assert!(task.is_some());
    assert_eq!(app.recent_projects_edits, vec!["project".to_string()]);
    let saved = app.recent_projects_meta.iter().find(|item| item.path == path).unwrap();
    assert_eq!(saved.name, "project");
    assert_eq!(saved.icon.as_deref(), Some("icon"));
    assert_eq!(saved.icon_color.as_deref(), Some("#123456"));
    assert_eq!(saved.worktree_start_command.as_deref(), Some("pnpm dev"));
    assert!(!saved.session_auto_refresh);
    assert_eq!(saved.session_refresh_interval_seconds, 3600);
    assert!(app.project_worktree_enabled.contains_key(&path));
    assert!(app.project_edit_path.is_none());
    assert!(app.project_edit_name.is_empty());
    assert!(app.project_edit_icon.is_empty());
    assert_eq!(app.project_edit_tab, crate::app::state::ProjectEditTab::General);
    assert_eq!(app.task_board_settings.max_concurrent, 10);
    assert!(!app.task_board_settings.auto_refresh);
}

#[test]
fn edit_cancel_resets_transient_fields() {
    let mut app = app();
    app.project_edit_path = Some("/tmp/project".to_string());
    app.project_edit_name = "Name".to_string();
    app.project_edit_icon = "icon".to_string();
    app.project_edit_icon_hovered = true;
    app.project_edit_icon_color = "#fff".to_string();
    app.project_edit_icon_color_picker_open = true;
    app.project_edit_start_script = "run".to_string();
    app.project_edit_worktree_enabled = true;
    app.project_edit_max_concurrent_input = "10".to_string();
    app.project_edit_task_board_auto_refresh = false;

    let _ = super::handle(
        &mut app,
        crate::app::message::project::ProjectMessage::ProjectEditCanceled,
    );

    assert!(app.project_edit_path.is_none());
    assert!(app.project_edit_name.is_empty());
    assert!(app.project_edit_icon.is_empty());
    assert!(!app.project_edit_icon_hovered);
    assert!(app.project_edit_icon_color.is_empty());
    assert!(!app.project_edit_icon_color_picker_open);
    assert!(app.project_edit_start_script.is_empty());
    assert!(!app.project_edit_worktree_enabled);
    assert!(app.project_edit_max_concurrent_input.is_empty());
    assert!(app.project_edit_task_board_auto_refresh);
}

#[test]
fn recent_remove_updates_recent_lists_and_clears_related_state() {
    let mut app = app();
    let removed = "/tmp/remove".to_string();
    let kept = "/tmp/keep".to_string();
    app.recent_projects = vec![removed.clone(), kept.clone()];
    app.recent_projects_meta = vec![meta(&kept, "Keep")];
    app.hovered_recent_project = Some(removed.clone());
    app.project_tools_menu_path = Some(removed.clone());
    app.project_edit_path = Some(removed.clone());
    app.project_edit_name = "Remove".to_string();

    let _ = super::handle(
        &mut app,
        crate::app::message::project::ProjectMessage::RecentRemovePressed(removed),
    );

    assert_eq!(app.recent_projects, vec![kept]);
    assert_eq!(app.recent_projects_edits, vec!["Keep".to_string()]);
    assert!(app.hovered_recent_project.is_none());
    assert!(app.project_tools_menu_path.is_none());
    assert!(app.project_edit_path.is_none());
    assert!(app.project_edit_name.is_empty());
}
