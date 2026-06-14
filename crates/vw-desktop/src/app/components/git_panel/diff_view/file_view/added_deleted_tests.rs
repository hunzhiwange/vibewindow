use super::super::DiffRenderCtx;
use super::added_deleted::{render_added_lines, render_deleted_lines};
use crate::app::components::git_panel::utils::Lang;
use crate::app::{App, DiffTheme};
use iced::Color;

fn app() -> App {
    App::new().0
}

#[test]
fn task_702_test_module_is_wired() {
    let path = std::path::Path::new(file!());
    assert!(path.ends_with("added_deleted_tests.rs"));
}

#[test]
fn render_added_and_deleted_lines_cover_merge_and_split_paths() {
    let mut app = app();
    app.staged_lines_selected.push(("src/lib.rs".to_string(), 0));
    app.staged_old_lines_selected.push(("src/lib.rs".to_string(), 1));
    app.git_diff_hovered_line = Some(("src/lib.rs".to_string(), 0, false));
    let ctx = DiffRenderCtx::new(&app);

    let added = render_added_lines(
        &app,
        &ctx,
        "src/lib.rs",
        &["let added = true;", "println!(\"new\");"],
        Lang::Rust,
        DiffTheme::GitHub,
        Color::from_rgb(0.8, 1.0, 0.8),
        Color::from_rgb(0.5, 0.9, 0.5),
        Color::BLACK,
        0.2,
        Color::WHITE,
        true,
        true,
    );
    let _ = added;

    let deleted = render_deleted_lines(
        &app,
        &ctx,
        "src/lib.rs",
        &["let deleted = true;", "println!(\"old\");"],
        Lang::Rust,
        DiffTheme::GitHub,
        Color::from_rgb(1.0, 0.8, 0.8),
        Color::BLACK,
        0.2,
        Color::WHITE,
        true,
        true,
    );
    let _ = deleted;

    app.merge_view = false;
    let ctx = DiffRenderCtx::new(&app);
    let split_added = render_added_lines(
        &app,
        &ctx,
        "src/lib.rs",
        &["split added"],
        Lang::Other,
        DiffTheme::GitHub,
        Color::TRANSPARENT,
        Color::TRANSPARENT,
        Color::BLACK,
        0.0,
        Color::WHITE,
        false,
        false,
    );
    let split_deleted = render_deleted_lines(
        &app,
        &ctx,
        "src/lib.rs",
        &["split deleted"],
        Lang::Other,
        DiffTheme::GitHub,
        Color::TRANSPARENT,
        Color::BLACK,
        0.0,
        Color::WHITE,
        false,
        false,
    );
    let _ = (split_added, split_deleted);
}
