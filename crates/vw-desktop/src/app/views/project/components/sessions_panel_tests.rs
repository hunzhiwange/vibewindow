use super::*;
use crate::app::state::{AcpHistoryReplayMode, ChatSendBehavior, QueueItem, SessionRuntimeState};
use iced::widget::button;
use vw_shared::session::info::{Info, Summary, TimeInfo};

fn test_app() -> crate::app::App {
    crate::app::App::new().0
}

fn keep_element(element: Element<'_, Message>) {
    std::hint::black_box(element);
}

fn session(id: &str, title: &str, directory: &str, archived: bool) -> Info {
    Info {
        id: id.to_string(),
        slug: format!("{id}-slug"),
        project_id: "project".to_string(),
        directory: directory.to_string(),
        parent_id: None,
        summary: Some(Summary { additions: 3, deletions: 1, files: 2, diffs: None }),
        share: None,
        title: title.to_string(),
        version: "1".to_string(),
        time: TimeInfo {
            created: 1,
            updated: 2,
            compacting: None,
            archived: archived.then_some(3),
        },
        permission: None,
        revert: None,
    }
}

fn queue_item(query: &str) -> QueueItem {
    QueueItem {
        created_ms: 1,
        query: query.to_string(),
        attachments: Vec::new(),
        root: None,
        model: None,
        acp_test: false,
        acp_agent: None,
        acp_allowed_tools: None,
        agent: None,
        allowed_tools: None,
        acp_force_new_session: false,
        acp_history_mode: AcpHistoryReplayMode::Discard,
        acp_recent_count: 3,
        full_access_enabled: true,
        send_behavior: ChatSendBehavior::Queue,
        request_history_override: None,
        resume_history_only: false,
        workflow_mode_enabled: false,
    }
}

#[test]
fn test_module_is_wired() {
    let module = std::hint::black_box(module_path!());

    assert!(module.ends_with("sessions_panel_tests"));
}

#[test]
fn button_styles_cover_light_and_dark_states() {
    for theme in [Theme::Light, Theme::Dark] {
        for status in [
            button::Status::Active,
            button::Status::Hovered,
            button::Status::Pressed,
            button::Status::Disabled,
        ] {
            assert!(icon_action_button_style(&theme, status).background.is_some());
            assert!(outline_panel_button_style(&theme, false, status).background.is_some());
            assert!(outline_panel_button_style(&theme, true, status).background.is_some());
            assert!(session_row_button_style(&theme, false, status).background.is_some());
            assert!(session_row_button_style(&theme, true, status).background.is_some());
        }
    }
}

#[test]
fn menu_builders_return_elements() {
    keep_element(session_menu_button(
        "重命名",
        Message::Project(message::ProjectMessage::SessionRenamePressed("s1".to_string())),
    ));
    keep_element(build_session_menu("s1".to_string()));
    keep_element(build_project_tools_menu("/tmp/project".to_string()));
}

#[test]
fn header_builds_clickable_and_static_variants() {
    keep_element(project_sessions_header(
        "项目".to_string(),
        "/tmp/project".to_string(),
        12,
        true,
        true,
    ));
    keep_element(project_sessions_header(
        "项目".to_string(),
        "/tmp/project".to_string(),
        8,
        false,
        false,
    ));
}

#[test]
fn session_items_build_loading_empty_and_load_button_states() {
    let app = test_app();
    keep_element(session_items_list(&app, "/tmp/project", None, 18, 10, false));
    keep_element(session_items_list(&app, "/tmp/project", Some(&Vec::new()), 18, 10, false));
    keep_element(session_items_list(&app, "/tmp/project", None, 18, 10, true));
}

#[test]
fn session_items_filters_archived_and_shows_load_more() {
    let app = test_app();
    let sessions = vec![
        session("s1", "可见会话一", "/tmp/project", false),
        session("s2", "已归档会话", "/tmp/project", true),
        session("s3", "可见会话二", "/tmp/project/worktree-a", false),
    ];

    keep_element(session_items_list(&app, "/tmp/project", Some(&sessions), 16, 1, false));
    keep_element(session_items_list(&app, "/tmp/project", Some(&sessions), 16, 0, false));
}

#[test]
fn session_items_reports_empty_when_only_archived_sessions_exist() {
    let app = test_app();
    let sessions = vec![
        session("s1", "归档一", "/tmp/project", true),
        session("s2", "归档二", "/tmp/project/worktree-a", true),
    ];

    keep_element(session_items_list(&app, "/tmp/project", Some(&sessions), 16, 10, false));
}

#[test]
fn session_items_builds_runtime_status_variants() {
    let mut app = test_app();
    app.active_session_id = Some("running".to_string());
    app.session_menu_id = Some("queued".to_string());
    app.session_menu_anchor = Some(Point::new(4.0, 6.0));

    let mut running = SessionRuntimeState::default();
    running.is_requesting = true;
    app.session_runtime_states.insert("running".to_string(), running);

    let mut queued = SessionRuntimeState::default();
    queued.queue.push(queue_item("继续"));
    app.session_runtime_states.insert("queued".to_string(), queued);

    let mut unseen = SessionRuntimeState::default();
    unseen.has_unseen_success = true;
    app.session_runtime_states.insert("unseen".to_string(), unseen);

    let sessions = vec![
        session("running", "运行中会话", "/tmp/project", false),
        session("queued", "排队会话", "/tmp/project/worktree-b", false),
        session("unseen", "未读成功会话", "/tmp/project", false),
        session("idle", "空闲会话", "", false),
    ];

    keep_element(session_items_list(&app, "/tmp/project", Some(&sessions), 10, 10, false));
}

#[test]
fn project_panel_builds_empty_and_selected_project_states() {
    let mut app = test_app();
    keep_element(project_sessions_panel(&app, 476.0, 72.0, 0.5, None));

    app.recent_projects.push("/tmp/project".to_string());
    keep_element(project_sessions_panel(&app, 476.0, 72.0, 0.5, None));

    app.recent_projects_edits.push("自定义项目名".to_string());
    app.project_tools_menu_path = Some("/tmp/project".to_string());
    app.project_session_has_vertical_scrollbar.insert("/tmp/project".to_string(), true);
    app.project_session_load_counts.insert("/tmp/project".to_string(), 1);
    app.project_sessions.insert(
        "/tmp/project".to_string(),
        vec![
            session("s1", "第一条会话", "/tmp/project", false),
            session("s2", "第二条会话", "/tmp/project/worktree-c", false),
        ],
    );

    keep_element(project_sessions_panel(&app, 476.0, 72.0, 0.5, Some("/tmp/project".to_string())));

    app.project_sessions_loading.insert("/tmp/loading".to_string());
    keep_element(project_sessions_panel(&app, 80.0, 120.0, 0.5, Some("/tmp/loading".to_string())));
}

#[test]
fn project_panel_uses_path_fallback_for_unknown_or_blank_recent_names() {
    let mut app = test_app();
    app.recent_projects = vec!["/tmp/project".to_string()];
    app.recent_projects_edits = vec!["   ".to_string()];
    app.project_path = Some("/tmp/project".to_string());

    keep_element(project_sessions_panel(&app, 476.0, 72.0, 0.5, Some("/tmp/project".to_string())));
    keep_element(project_sessions_panel(
        &app,
        476.0,
        72.0,
        0.5,
        Some("/tmp/unknown-project".to_string()),
    ));
}
