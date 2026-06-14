use super::context_menu::{diff_selection_menu, wrap_diff_row_with_context_menu};
use crate::app::App;
use crate::app::state::GitDiffContextMenuState;
use iced::widget::text;

fn app() -> App {
    App::new().0
}

#[test]
fn task_701_test_module_is_wired() {
    let path = std::path::Path::new(file!());
    assert!(path.ends_with("context_menu_tests.rs"));
}

#[test]
fn selection_menu_and_context_wrapper_render_for_closed_and_open_states() {
    let mut app = app();

    let _menu = diff_selection_menu();
    let closed = wrap_diff_row_with_context_menu(
        &app,
        "src/lib.rs",
        2,
        false,
        "let x = 1;".to_string(),
        text("row").into(),
    );
    let _ = closed;

    app.git_diff_context_menu = Some(GitDiffContextMenuState {
        file: "src/lib.rs".to_string(),
        line: 2,
        is_old: false,
        x: 12.0,
        y: 24.0,
    });
    let open = wrap_diff_row_with_context_menu(
        &app,
        "src/lib.rs",
        2,
        false,
        "let x = 1;".to_string(),
        text("row").into(),
    );
    let _ = open;
}
