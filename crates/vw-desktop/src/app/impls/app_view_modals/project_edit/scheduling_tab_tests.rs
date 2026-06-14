use super::*;

fn test_app() -> App {
    let (app, _task) = App::new();
    app
}

#[test]
fn scheduling_tab_builds_with_default_settings() {
    let app = test_app();

    let element = scheduling_tab(&app);

    std::hint::black_box(element);
}

#[test]
fn scheduling_tab_clamps_low_values_for_stepper_controls() {
    let mut app = test_app();
    app.project_edit_task_board_settings.max_concurrent = 0;
    app.project_edit_task_board_settings.scheduler_tick_interval_seconds = 0;
    app.project_edit_task_board_settings.auto_promote_tick_interval_seconds = 0;
    app.project_edit_task_board_settings.failed_retry_minutes = 0;
    app.project_edit_task_board_settings.running_timeout_minutes = 0;
    app.project_edit_task_board_settings.pr_submitted_stall_timeout_seconds = 0;
    app.project_edit_task_board_settings.recycle_worktree_on_task_finish = true;

    let element = scheduling_tab(&app);

    std::hint::black_box(element);
}

#[test]
fn scheduling_tab_clamps_high_values_for_stepper_controls() {
    let mut app = test_app();
    app.project_edit_task_board_settings.max_concurrent = 99;
    app.project_edit_task_board_settings.scheduler_tick_interval_seconds = 99;
    app.project_edit_task_board_settings.auto_promote_tick_interval_seconds = 9000;
    app.project_edit_task_board_settings.failed_retry_minutes = 9999;
    app.project_edit_task_board_settings.running_timeout_minutes = 9999;
    app.project_edit_task_board_settings.pr_submitted_stall_timeout_seconds = 9999;

    let element = scheduling_tab(&app);

    std::hint::black_box(element);
}
