#[test]
fn test_module_is_wired() {
    let module = module_path!();

    assert!(module.ends_with("open_tests"));
}

fn app() -> crate::app::App {
    crate::app::App::new().0
}

fn session_info(id: &str, additions: i64, deletions: i64) -> vw_shared::session::info::Info {
    vw_shared::session::info::Info {
        id: id.to_string(),
        slug: id.to_string(),
        project_id: "project".to_string(),
        directory: "/tmp/project".to_string(),
        parent_id: None,
        summary: Some(vw_shared::session::info::Summary {
            additions,
            deletions,
            files: 1,
            diffs: None,
        }),
        share: None,
        title: id.to_string(),
        version: "1".to_string(),
        time: vw_shared::session::info::TimeInfo {
            created: 1,
            updated: 1,
            compacting: None,
            archived: None,
        },
        permission: None,
        revert: None,
    }
}

#[test]
fn merge_project_sessions_returns_incoming_when_no_existing_list() {
    let incoming = vec![session_info("incoming", 1, 2)];

    let merged = super::merge_project_sessions(None, incoming.clone());

    assert_eq!(merged.len(), 1);
    assert_eq!(merged[0].id, incoming[0].id);
}

#[test]
fn merge_project_sessions_replaces_existing_summary() {
    let existing = vec![session_info("task-board-1", 0, 0), session_info("local-only", 2, 1)];
    let incoming = vec![session_info("task-board-1", 7, 3)];

    let merged = super::merge_project_sessions(Some(existing), incoming);

    assert_eq!(merged.len(), 2);
    let summary = merged[0].summary.as_ref().expect("incoming summary should be kept");
    assert_eq!(merged[0].id, "task-board-1");
    assert_eq!(summary.additions, 7);
    assert_eq!(summary.deletions, 3);
    assert_eq!(merged[1].id, "local-only");
}

#[test]
fn sessions_loaded_updates_list_or_pushes_notification() {
    let mut app = app();
    let sessions = vec![session_info("s1", 1, 0)];

    let task = super::handle(
        &mut app,
        crate::app::message::project::ProjectMessage::SessionsLoaded(Ok(sessions.clone())),
    );

    assert!(task.is_some());
    assert_eq!(app.sessions.len(), 1);
    assert_eq!(app.sessions[0].id, "s1");

    let _ = super::handle(
        &mut app,
        crate::app::message::project::ProjectMessage::SessionsLoaded(Err("boom".to_string())),
    );
    assert!(app.notifications.iter().any(|item| item.message.contains("boom")));
}

#[test]
fn session_bootstrap_loaded_sets_previews_archived_and_sessions() {
    let mut app = app();
    let sessions = vec![session_info("s1", 1, 1)];
    let mut previews = std::collections::HashMap::new();
    previews.insert("s1".to_string(), "preview text".to_string());
    let archived = std::collections::HashSet::from(["old".to_string()]);

    let _ = super::handle(
        &mut app,
        crate::app::message::project::ProjectMessage::SessionBootstrapLoaded {
            result: Ok(sessions),
            previews: previews.clone(),
            archived_session_ids: archived.clone(),
        },
    );

    assert_eq!(app.session_previews, previews);
    assert_eq!(app.archived_session_ids, archived);
    assert_eq!(app.sessions[0].id, "s1");
}

#[test]
fn project_sessions_loaded_merges_and_clears_loading_state() {
    let mut app = app();
    let path = "/tmp/project".to_string();
    app.project_sessions_loading.insert(path.clone());
    app.project_sessions.insert(path.clone(), vec![session_info("local", 0, 0)]);

    let _ = super::handle(
        &mut app,
        crate::app::message::project::ProjectMessage::ProjectSessionsLoaded(
            path.clone(),
            Ok(vec![session_info("remote", 3, 1)]),
        ),
    );

    assert!(!app.project_sessions_loading.contains(&path));
    let sessions = app.project_sessions.get(&path).expect("sessions should be stored");
    assert_eq!(sessions.iter().map(|s| s.id.as_str()).collect::<Vec<_>>(), vec!["remote", "local"]);
    assert_eq!(app.project_session_load_counts.get(&path), Some(&10));
}

#[test]
fn project_sessions_loaded_records_error_message() {
    let mut app = app();
    let path = "/tmp/project".to_string();
    app.project_sessions_loading.insert(path.clone());

    let _ = super::handle(
        &mut app,
        crate::app::message::project::ProjectMessage::ProjectSessionsLoaded(
            path.clone(),
            Err("offline".to_string()),
        ),
    );

    assert!(!app.project_sessions_loading.contains(&path));
    assert_eq!(app.error_message.as_deref(), Some("Failed to load project sessions: offline"));
}

#[test]
fn project_session_scroll_and_load_more_update_per_project_state() {
    let mut app = app();
    let path = "/tmp/project".to_string();

    let _ = super::handle(
        &mut app,
        crate::app::message::project::ProjectMessage::ProjectSessionListScrollChanged {
            project_path: path.clone(),
            has_vertical_scrollbar: true,
        },
    );
    assert_eq!(app.project_session_has_vertical_scrollbar.get(&path), Some(&true));

    let _ = super::handle(
        &mut app,
        crate::app::message::project::ProjectMessage::ProjectLoadMoreSessions(path.clone()),
    );
    assert_eq!(app.project_session_load_counts.get(&path), Some(&15));

    let _ = super::handle(
        &mut app,
        crate::app::message::project::ProjectMessage::ProjectLoadMoreSessions(path.clone()),
    );
    assert_eq!(app.project_session_load_counts.get(&path), Some(&20));
}

#[test]
fn recent_overlay_closed_preserves_open_picker_but_otherwise_resets_overlay_state() {
    let mut app = app();
    app.new_session_picker_project = Some("/tmp/project".to_string());
    app.hovered_recent_project = Some("/tmp/project".to_string());

    let _ =
        super::handle(&mut app, crate::app::message::project::ProjectMessage::RecentOverlayClosed);
    assert!(app.new_session_picker_project.is_some());
    assert!(app.hovered_recent_project.is_some());

    app.new_session_picker_project = None;
    app.session_menu_id = Some("s1".to_string());
    app.project_tools_menu_path = Some("/tmp/project".to_string());

    let _ =
        super::handle(&mut app, crate::app::message::project::ProjectMessage::RecentOverlayClosed);

    assert!(app.hovered_recent_project.is_none());
    assert!(app.session_menu_id.is_none());
    assert!(app.project_tools_menu_path.is_none());
}

#[test]
fn project_create_session_opens_picker_when_worktree_enabled() {
    let mut app = app();
    let path = "/tmp/project".to_string();
    app.project_worktree_enabled.insert(path.clone(), true);
    app.new_session_picker_options.push(("old".to_string(), "旧".to_string()));
    app.new_session_worktree_name = "old-name".to_string();
    app.new_session_delete_error = Some("old error".to_string());

    let task = super::handle(
        &mut app,
        crate::app::message::project::ProjectMessage::ProjectCreateSession(path.clone()),
    );

    assert!(task.is_some());
    assert_eq!(app.new_session_picker_project, Some(path));
    assert!(app.new_session_picker_options.is_empty());
    assert!(app.new_session_worktree_name.is_empty());
    assert!(app.new_session_delete_error.is_none());
}

#[test]
fn project_branches_loaded_ignores_stale_project_and_applies_current_project() {
    let mut app = app();
    app.project_path = Some("/tmp/current".to_string());
    app.selected_branch = Some("main".to_string());

    let _ = super::handle(
        &mut app,
        crate::app::message::project::ProjectMessage::ProjectBranchesLoaded {
            project_path: "/tmp/other".to_string(),
            selected_branch: Some("feature".to_string()),
            branches: vec!["feature".to_string()],
        },
    );
    assert_eq!(app.selected_branch.as_deref(), Some("main"));

    let _ = super::handle(
        &mut app,
        crate::app::message::project::ProjectMessage::ProjectBranchesLoaded {
            project_path: "/tmp/current".to_string(),
            selected_branch: Some("develop".to_string()),
            branches: vec!["main".to_string(), "develop".to_string()],
        },
    );

    assert_eq!(app.selected_branch.as_deref(), Some("develop"));
    assert_eq!(app.branches, vec!["main".to_string(), "develop".to_string()]);
}

#[test]
fn project_load_sessions_skips_duplicate_loads_and_marks_new_load() {
    let mut app = app();
    let path = "/tmp/project".to_string();
    app.project_sessions_loading.insert(path.clone());

    let _ = super::handle(
        &mut app,
        crate::app::message::project::ProjectMessage::ProjectLoadSessions(path.clone()),
    );
    assert_eq!(app.project_sessions_loading.len(), 1);

    app.project_sessions_loading.clear();
    let task = super::handle(
        &mut app,
        crate::app::message::project::ProjectMessage::ProjectLoadSessions(path.clone()),
    );

    assert!(task.is_some());
    assert!(app.project_sessions_loading.contains(&path));
    assert!(app.project_sessions_last_refresh_at.contains_key(&path));
}
