use super::styles::{
    estimate_preview_tabs_viewport_width, estimate_tab_title_width_px, estimate_tab_width_px,
    file_icon_for, menu_button_style, should_show_preview_tabs_scrollbar, truncate_title,
};
use crate::app::assets::Icon;
use crate::app::components::editor::Editor;
use crate::app::{App, PreviewTab};
use iced::widget::button;
use iced::widget::svg;
use iced::{Background, Color, Length, Theme};

fn test_app() -> App {
    App::new().0
}

fn assert_close(actual: f32, expected: f32) {
    assert!((actual - expected).abs() < 0.001, "actual={actual}, expected={expected}");
}

fn preview_tab(title: &str) -> PreviewTab {
    PreviewTab {
        path: format!("/tmp/{title}"),
        title: title.to_string(),
        content: String::new(),
        is_dirty: false,
        truncated: false,
        auto_save_revision: 0,
        editor: Editor::new("", "txt"),
        scroll_id: iced::widget::Id::unique(),
        #[cfg(not(target_arch = "wasm32"))]
        lsp_server_key: None,
        #[cfg(not(target_arch = "wasm32"))]
        lsp_uri: None,
        #[cfg(not(target_arch = "wasm32"))]
        lsp_language_id: None,
    }
}

#[test]
fn file_icon_for_known_extensions() {
    assert_eq!(file_icon_for("main.rs"), Icon::Rust);
    assert_eq!(file_icon_for("component.TSX"), Icon::Typescript);
    assert_eq!(file_icon_for("index.jsx"), Icon::Javascript);
    assert_eq!(file_icon_for("package.json"), Icon::Json);
    assert_eq!(file_icon_for("Cargo.toml"), Icon::Toml);
    assert_eq!(file_icon_for("workflow.yml"), Icon::Yaml);
    assert_eq!(file_icon_for("README.md"), Icon::Markdown);
    assert_eq!(file_icon_for("index.html"), Icon::Html);
    assert_eq!(file_icon_for("styles.css"), Icon::Css);
    assert_eq!(file_icon_for("script.py"), Icon::Python);
    assert_eq!(file_icon_for("main.go"), Icon::Go);
    assert_eq!(file_icon_for("run.sh"), Icon::Console);
    assert_eq!(file_icon_for("photo.webp"), Icon::Image);
    assert_eq!(file_icon_for("vector.svg"), Icon::Image);
    assert_eq!(file_icon_for("unknown.bin"), Icon::Document);
}

#[test]
fn truncate_title_keeps_short_titles_and_shortens_long_titles() {
    assert_eq!(truncate_title("short", 8), "short");
    assert_eq!(truncate_title("preview-panel", 7), "preview…");
    assert_eq!(truncate_title("中文标题", 2), "中文…");
    assert_eq!(truncate_title("abc", 0), "…");
}

#[test]
fn estimate_title_width_respects_unicode_width_and_minimum() {
    assert_eq!(estimate_tab_title_width_px(""), 32.0);
    assert_eq!(estimate_tab_title_width_px("abc"), 32.0);
    assert!(estimate_tab_title_width_px("中文标题") > estimate_tab_title_width_px("title"));
}

#[test]
fn estimate_tab_width_adds_fixed_chrome() {
    let title_width = estimate_tab_title_width_px("main.rs");

    assert_eq!(estimate_tab_width_px("main.rs"), title_width + 62.0);
}

#[test]
fn viewport_width_accounts_for_file_manager_and_diff_split() {
    let mut app = test_app();
    app.window_size = (1000.0, 700.0);
    app.show_file_manager = false;
    app.show_diff = false;

    assert_eq!(estimate_preview_tabs_viewport_width(&app), 982.0);

    app.show_file_manager = true;
    app.file_manager_width = 240.0;
    assert_eq!(estimate_preview_tabs_viewport_width(&app), 734.0);

    app.show_diff = true;
    app.split_ratio = 0.75;
    assert_close(estimate_preview_tabs_viewport_width(&app), 168.0);

    app.split_ratio = 0.01;
    assert_close(estimate_preview_tabs_viewport_width(&app), 577.2);
}

#[test]
fn scrollbar_visibility_tracks_estimated_tab_widths() {
    let mut app = test_app();
    app.window_size = (1200.0, 700.0);
    app.preview_tabs = vec![];
    assert!(!should_show_preview_tabs_scrollbar(&app));

    app.preview_tabs = vec![preview_tab("main.rs")];
    assert!(!should_show_preview_tabs_scrollbar(&app));

    app.window_size = (160.0, 700.0);
    app.preview_tabs =
        vec![preview_tab("very-long-file-name-that-will-overflow-the-small-preview-tabs.rs")];
    assert!(should_show_preview_tabs_scrollbar(&app));
}

#[test]
fn menu_button_style_only_adds_background_on_hover() {
    let theme = Theme::Light;

    let active = menu_button_style(&theme, button::Status::Active);
    let hovered = menu_button_style(&theme, button::Status::Hovered);

    assert!(active.background.is_none());
    assert_eq!(active.text_color, theme.palette().text);
    assert_eq!(active.border.width, 0.0);
    assert!(matches!(hovered.background, Some(Background::Color(_))));
}

#[test]
fn icon_svg_helpers_use_fixed_small_size_and_theme_color() {
    let small = super::styles::small_icon_svg(Icon::X);
    let file = super::styles::file_tab_icon_svg(Icon::Rust);

    std::hint::black_box((small, file));

    let dark_style = svg::Style { color: Some(Theme::Dark.palette().text) };
    assert!(matches!(dark_style.color, Some(Color { .. })));
    assert_eq!(Length::Fixed(14.0), Length::Fixed(14.0));
}
