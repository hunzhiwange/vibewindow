use super::types::{MindMapTheme, MindMapThemeView};

#[test]
fn theme_palette_cycles_branch_fills() {
    let theme = MindMapTheme {
        id: "local",
        name: "Local",
        background_color: 0,
        root_fill: 0,
        root_text: 0,
        branch_fills: &[1, 2, 3],
        branch_text: 0,
        leaf_fill: 0,
        leaf_text: 0,
        line_color: None,
        is_dark: false,
    };

    assert_eq!(theme.palette(4), 2);
}

#[test]
fn theme_view_palette_returns_default_for_empty_fills() {
    let view = MindMapThemeView {
        background_color: 0,
        root_fill: 0,
        root_text: 0,
        branch_fills: &[],
        branch_text: 0,
        leaf_fill: 0,
        leaf_text: 0,
        line_color: None,
        is_dark: false,
    };

    assert_eq!(view.palette(0), 0xFF0000FF);
}
