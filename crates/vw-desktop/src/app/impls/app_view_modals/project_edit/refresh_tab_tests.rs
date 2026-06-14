use super::*;

fn test_app() -> App {
    let (app, _task) = App::new();
    app
}

#[test]
fn refresh_tab_builds_with_default_refresh_settings() {
    let app = test_app();

    let element = refresh_tab(&app);

    std::hint::black_box(element);
}

#[test]
fn refresh_tab_clamps_invalid_and_low_intervals_for_controls() {
    let mut app = test_app();
    app.project_edit_session_refresh_interval_seconds_input = "not-a-number".to_string();
    app.project_edit_task_board_settings.refresh_interval_seconds = 0;
    app.project_edit_task_board_auto_refresh = false;
    app.project_edit_session_auto_refresh = false;
    app.project_edit_task_board_settings.auto_promote_pool_tasks = false;
    app.project_edit_task_board_settings.code_review_enabled = true;

    let element = refresh_tab(&app);

    std::hint::black_box(element);
}

#[test]
fn refresh_tab_clamps_high_intervals_for_controls() {
    let mut app = test_app();
    app.project_edit_session_refresh_interval_seconds_input = "9000".to_string();
    app.project_edit_task_board_settings.refresh_interval_seconds = 9000;

    let element = refresh_tab(&app);

    std::hint::black_box(element);
}
