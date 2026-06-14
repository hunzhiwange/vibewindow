use iced::Color;

use crate::app::components::git_panel::diff_view::DiffRenderCtx;
use crate::app::components::git_panel::utils::Lang;
use crate::app::state::GitDiffSelectedLine;
use crate::app::{App, DiffTheme};

fn palette() -> (Color, Color, Color, Color, Color) {
    (
        Color::from_rgba8(0, 255, 0, 0.10),
        Color::from_rgba8(0, 255, 0, 0.25),
        Color::from_rgba8(255, 0, 0, 0.10),
        Color::from_rgba8(255, 0, 0, 0.25),
        Color::from_rgba8(255, 210, 0, 0.22),
    )
}

fn render_count(
    app: &App,
    old_index: usize,
    old_len: usize,
    new_index: usize,
    new_len: usize,
    old_lines: &[&str],
    new_lines: &[&str],
) -> usize {
    let ctx = DiffRenderCtx::new(app);
    let (add_line_bg, add_word_bg, del_line_bg, del_word_bg, hover) = palette();

    super::replaces::render_replace_ops(
        app,
        &ctx,
        "src/lib.rs",
        old_index,
        old_len,
        new_index,
        new_len,
        old_lines,
        new_lines,
        Lang::Rust,
        DiffTheme::GitHub,
        add_line_bg,
        add_word_bg,
        del_line_bg,
        del_word_bg,
        hover,
        0.22,
        hover,
        false,
        true,
    )
    .len()
}

#[test]
fn should_compute_word_diff_requires_highlight_equal_lengths_changes_and_short_lines() {
    let mut app = App::new().0;
    app.show_git_diff_highlight = true;

    assert!(super::replaces::should_compute_word_diff(&app, 1, 1, "let a = 1;", "let a = 2;"));
    assert!(super::replaces::should_compute_word_diff(&app, 1, 1, "a", "b"));
    assert!(!super::replaces::should_compute_word_diff(&app, 1, 2, "a", "b"));
    assert!(!super::replaces::should_compute_word_diff(&app, 1, 1, "same", "same"));
    assert!(!super::replaces::should_compute_word_diff(&app, 1, 1, &"a".repeat(513), "b"));

    app.show_git_diff_highlight = false;
    assert!(super::replaces::should_compute_word_diff(&app, 1, 1, "a", "b"));
}

#[test]
fn render_replace_ops_merge_view_emits_old_then_new_rows() {
    let mut app = App::new().0;
    app.merge_view = true;

    assert_eq!(
        render_count(&app, 0, 2, 0, 3, &["old a", "old b"], &["new a", "new b", "new c"]),
        5
    );
}

#[test]
fn render_replace_ops_split_view_uses_max_old_new_count() {
    let mut app = App::new().0;
    app.merge_view = false;

    assert_eq!(
        render_count(&app, 0, 2, 0, 3, &["old a", "old b"], &["new a", "new b", "new c"]),
        3
    );
    assert_eq!(render_count(&app, 0, 3, 0, 1, &["old a", "old b", "old c"], &["new a"]), 3);
}

#[test]
fn render_replace_ops_handles_equal_length_word_diff_and_missing_lines() {
    let mut app = App::new().0;
    app.show_git_diff_highlight = true;
    app.merge_view = false;

    assert_eq!(render_count(&app, 0, 2, 0, 2, &["let a = 1;", "let b = 1;"], &["let a = 2;"]), 2);
}

#[test]
fn render_replace_ops_covers_selected_staged_hovered_and_file_staged_branches() {
    let mut app = App::new().0;
    app.staged_old_lines_selected = vec![("src/lib.rs".to_string(), 0)];
    app.staged_lines_selected = vec![("src/lib.rs".to_string(), 0)];
    app.git_diff_selected_lines = vec![
        GitDiffSelectedLine {
            file: "src/lib.rs".to_string(),
            line: 0,
            is_old: true,
            text: "old".to_string(),
        },
        GitDiffSelectedLine {
            file: "src/lib.rs".to_string(),
            line: 0,
            is_old: false,
            text: "new".to_string(),
        },
    ];
    app.git_diff_hovered_line = Some(("src/lib.rs".to_string(), 0, true));

    assert_eq!(render_count(&app, 0, 1, 0, 1, &["old"], &["new"]), 2);

    app.merge_view = true;
    app.staged_files_selected = vec!["src/lib.rs".to_string()];
    app.git_diff_hovered_line = Some(("src/lib.rs".to_string(), 0, false));
    assert_eq!(render_count(&app, 0, 1, 0, 1, &["old"], &["new"]), 2);
}
