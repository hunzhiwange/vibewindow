use super::*;

fn test_app() -> App {
    let (app, _task) = App::new();
    app
}

#[test]
fn launch_tab_builds_when_worktree_is_disabled() {
    let mut app = test_app();
    app.project_edit_start_script = "bun install\nbun dev".to_string();
    app.project_edit_start_script_editor =
        iced::widget::text_editor::Content::with_text(&app.project_edit_start_script);
    app.project_edit_worktree_enabled = false;

    let element = launch_tab(&app);

    std::hint::black_box(element);
}

#[test]
fn launch_tab_builds_when_worktree_is_enabled() {
    let mut app = test_app();
    app.project_edit_start_script = String::new();
    app.project_edit_start_script_editor = iced::widget::text_editor::Content::new();
    app.project_edit_worktree_enabled = true;

    let element = launch_tab(&app);

    std::hint::black_box(element);
}
