use iced::Color;

use crate::app::components::git_panel::diff_view::DiffRenderCtx;
use crate::app::components::git_panel::utils::Lang;
use crate::app::state::GitDiffSelectedLine;
use crate::app::{App, DiffTheme};

#[test]
fn render_equal_line_builds_for_plain_selected_and_hovered_rows() {
    let mut app = App::new().0;
    let ctx = DiffRenderCtx::new(&app);
    let _plain = super::standard::render_equal_line(
        &app,
        &ctx,
        "src/lib.rs",
        0,
        0,
        "let value = 1;",
        Lang::Rust,
        DiffTheme::GitHub,
        Color::WHITE,
        Color::from_rgba8(255, 210, 0, 0.22),
        false,
    );

    app.git_diff_selected_lines = vec![GitDiffSelectedLine {
        file: "src/lib.rs".to_string(),
        line: 1,
        is_old: false,
        text: "same".to_string(),
    }];
    app.git_diff_hovered_line = Some(("src/lib.rs".to_string(), 1, true));
    let ctx = DiffRenderCtx::new(&app);
    let _active = super::standard::render_equal_line(
        &app,
        &ctx,
        "src/lib.rs",
        1,
        1,
        "same",
        Lang::Other,
        DiffTheme::Monokai,
        Color::BLACK,
        Color::from_rgba8(255, 210, 0, 0.22),
        true,
    );
}
