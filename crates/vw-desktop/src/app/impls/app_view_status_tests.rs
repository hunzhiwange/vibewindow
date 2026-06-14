#[test]
fn app_view_status_tests_module_is_wired() {
    assert!(module_path!().ends_with("app_view_status_tests"));
}

#[cfg(not(target_arch = "wasm32"))]
use crate::app::preview::{LspProgress, PreviewTab};

#[cfg(not(target_arch = "wasm32"))]
fn preview_tab(path: &str, server_key: Option<&'static str>) -> PreviewTab {
    PreviewTab {
        path: path.to_string(),
        title: "main.rs".to_string(),
        content: "fn main() {}\n".to_string(),
        is_dirty: false,
        truncated: false,
        auto_save_revision: 0,
        editor: crate::app::components::editor::Editor::new("fn main() {}", "rust"),
        scroll_id: iced::widget::Id::unique(),
        lsp_server_key: server_key,
        lsp_uri: Some("file:///workspace/src/main.rs".to_string()),
        lsp_language_id: Some("rust".to_string()),
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn is_dark_theme_detects_dark_and_light_palettes() {
    assert!(super::is_dark_theme(&iced::Theme::Dark));
    assert!(!super::is_dark_theme(&iced::Theme::Light));
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn status_bar_builds_ready_and_status_variants() {
    let (mut app, _) = crate::app::App::new();
    std::hint::black_box(super::status_bar(&app));

    app.lsp_status = Some("LSP: starting".to_string());
    std::hint::black_box(super::status_bar(&app));
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn status_bar_builds_active_tab_lsp_variants() {
    let (mut app, _) = crate::app::App::new();
    app.active_preview_path = Some("/workspace/src/main.rs".to_string());
    app.preview_tabs.push(preview_tab("/workspace/src/main.rs", Some("rust-analyzer")));

    std::hint::black_box(super::status_bar(&app));

    app.lsp_progress.insert(
        "rust-analyzer".to_string(),
        std::collections::HashMap::from([(
            "token".to_string(),
            LspProgress {
                title: "index".to_string(),
                message: Some("crates".to_string()),
                percentage: Some(42),
            },
        )]),
    );
    app.spinner_frame = 99;
    std::hint::black_box(super::status_bar(&app));

    app.lsp_progress.get_mut("rust-analyzer").unwrap().get_mut("token").unwrap().percentage =
        Some(100);
    std::hint::black_box(super::status_bar(&app));
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn status_bar_builds_notification_variant() {
    let (mut app, _) = crate::app::App::new();
    app.notifications.push(crate::app::state::Notification {
        id: 1,
        message: "warning".to_string(),
        created_at: web_time::SystemTime::UNIX_EPOCH,
    });

    std::hint::black_box(super::status_bar(&app));
}
