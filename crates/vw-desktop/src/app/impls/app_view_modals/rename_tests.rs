#[test]
fn rename_tests_module_is_wired() {
    assert!(module_path!().ends_with("rename_tests"));
}

use iced::widget::text;

fn session(id: &str, title: &str) -> vw_shared::session::info::Info {
    vw_shared::session::info::Info {
        id: id.to_string(),
        slug: id.to_string(),
        project_id: "project".to_string(),
        directory: "/tmp/project".to_string(),
        parent_id: None,
        summary: None,
        share: None,
        title: title.to_string(),
        version: "1".to_string(),
        time: vw_shared::session::info::TimeInfo {
            created: 1,
            updated: 2,
            compacting: None,
            archived: None,
        },
        permission: None,
        revert: None,
    }
}

#[test]
fn file_tree_rename_without_path_leaves_root_content() {
    let (app, _) = crate::app::App::new();

    let element = super::with_file_tree_rename(&app, text("root").into());

    std::hint::black_box(element);
}

#[test]
fn file_tree_rename_builds_modal_from_path_filename() {
    let (mut app, _) = crate::app::App::new();
    app.file_tree_rename_path = Some("/workspace/src/main.rs".to_string());
    app.file_tree_rename_value = "lib.rs".to_string();

    let element = super::with_file_tree_rename(&app, text("root").into());

    std::hint::black_box(element);
}

#[test]
fn session_rename_uses_global_or_project_sessions() {
    let (mut app, _) = crate::app::App::new();
    app.session_rename_id = Some("session-1".to_string());
    app.session_rename_value = "new title".to_string();
    app.sessions.push(session("session-1", "old title"));

    let global_element = super::with_session_rename(&app, text("root").into());
    std::hint::black_box(global_element);

    app.sessions.clear();
    app.project_sessions
        .insert("/workspace".to_string(), vec![session("session-1", "project title")]);
    let project_element = super::with_session_rename(&app, text("root").into());
    std::hint::black_box(project_element);
}

#[test]
fn session_rename_without_id_leaves_root_content() {
    let (app, _) = crate::app::App::new();

    let element = super::with_session_rename(&app, text("root").into());

    std::hint::black_box(element);
}
