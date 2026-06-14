use super::super::DiffRenderCtx;
use super::gaps::render_gap;
use super::start_end::GapRange;
use crate::app::components::git_panel::utils::Lang;
use crate::app::{App, DiffTheme};
use iced::Color;

fn app() -> App {
    App::new().0
}

#[test]
fn task_703_test_module_is_wired() {
    let path = std::path::Path::new(file!());
    assert!(path.ends_with("gaps_tests.rs"));
}

#[test]
fn render_gap_returns_empty_for_zero_length_range() {
    let app = app();
    let ctx = DiffRenderCtx::new(&app);
    let elements = render_gap(
        &app,
        &ctx,
        "src/lib.rs",
        &["a", "b"],
        GapRange { start_old: 1, end_old: 1, start_new: 1, gap_id: 0 },
        Lang::Other,
        DiffTheme::GitHub,
        Color::TRANSPARENT,
        Color::WHITE,
        false,
    );

    assert!(elements.is_empty());
}

#[test]
fn render_gap_covers_full_partial_merge_and_split_paths() {
    let mut app = app();
    let old_lines = ["0", "1", "2", "3", "4", "5", "6", "7", "8", "9"];
    app.context_expansions.insert(("src/lib.rs".to_string(), 7), (4, 4));
    app.git_diff_hovered_line = Some(("src/lib.rs".to_string(), 2, true));
    let ctx = DiffRenderCtx::new(&app);

    let full = render_gap(
        &app,
        &ctx,
        "src/lib.rs",
        &old_lines,
        GapRange { start_old: 1, end_old: 5, start_new: 1, gap_id: 7 },
        Lang::Other,
        DiffTheme::GitHub,
        Color::from_rgb(0.95, 0.95, 0.95),
        Color::WHITE,
        true,
    );
    assert_eq!(full.len(), 4);

    app.context_expansions.clear();
    let ctx = DiffRenderCtx::new(&app);
    let partial = render_gap(
        &app,
        &ctx,
        "src/lib.rs",
        &old_lines,
        GapRange { start_old: 0, end_old: 10, start_new: 0, gap_id: 8 },
        Lang::Other,
        DiffTheme::GitHub,
        Color::TRANSPARENT,
        Color::WHITE,
        false,
    );
    assert_eq!(partial.len(), 7);

    app.merge_view = false;
    let ctx = DiffRenderCtx::new(&app);
    let split = render_gap(
        &app,
        &ctx,
        "src/lib.rs",
        &old_lines,
        GapRange { start_old: 0, end_old: 10, start_new: 0, gap_id: 9 },
        Lang::Other,
        DiffTheme::GitHub,
        Color::TRANSPARENT,
        Color::WHITE,
        false,
    );
    assert_eq!(split.len(), 7);
}
