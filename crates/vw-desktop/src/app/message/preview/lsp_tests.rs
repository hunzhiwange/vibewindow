#[test]
fn test_module_is_wired() {
    let module = module_path!();

    assert!(module.ends_with("lsp_tests"));
}

fn app() -> crate::app::App {
    crate::app::App::new().0
}

#[test]
fn clear_lsp_hover_clears_hover_state_and_path_when_completion_hidden() {
    let mut app = app();
    app.lsp_overlay.show_hover("hover".to_string());
    app.lsp_overlay_path = Some("/tmp/main.rs".to_string());
    app.lsp_hover_anchor =
        Some(("/tmp/main.rs".to_string(), iced_code_editor::LspPosition { line: 1, character: 2 }));
    app.lsp_hover_pending = Some(crate::app::LspHoverPending {
        path: "/tmp/main.rs".to_string(),
        position: iced_code_editor::LspPosition { line: 1, character: 2 },
        point: iced::Point::new(10.0, 20.0),
        ready_at: std::time::Instant::now(),
    });
    app.lsp_hover_hide_deadline = Some(std::time::Instant::now());

    super::lsp::clear_lsp_hover(&mut app);

    assert!(!app.lsp_overlay.hover_visible);
    assert!(app.lsp_hover_anchor.is_none());
    assert!(app.lsp_hover_pending.is_none());
    assert!(app.lsp_hover_hide_deadline.is_none());
    assert!(app.lsp_overlay_path.is_none());
}

#[test]
fn clear_lsp_completion_clears_items_and_preserves_hover_path() {
    let mut app = app();
    app.lsp_overlay.all_completions = vec!["Alpha".to_string(), "Beta".to_string()];
    app.lsp_overlay.completion_items = vec!["Alpha".to_string()];
    app.lsp_overlay.completion_filter = "a".to_string();
    app.lsp_overlay.completion_visible = true;
    app.lsp_overlay.completion_selected = 9;
    app.lsp_overlay.hover_visible = true;
    app.lsp_overlay_path = Some("/tmp/main.rs".to_string());

    super::lsp::clear_lsp_completion(&mut app, true);

    assert!(app.lsp_overlay.all_completions.is_empty());
    assert!(app.lsp_overlay.completion_items.is_empty());
    assert!(app.lsp_overlay.completion_filter.is_empty());
    assert!(!app.lsp_overlay.completion_visible);
    assert!(app.lsp_overlay.completion_suppressed);
    assert_eq!(app.lsp_overlay.completion_selected, 0);
    assert_eq!(app.lsp_overlay_path.as_deref(), Some("/tmp/main.rs"));
}

#[test]
fn filter_completions_matches_case_insensitively_and_clamps_selection() {
    let mut app = app();
    app.lsp_overlay.all_completions =
        vec!["ReadFile".to_string(), "write_file".to_string(), "Close".to_string()];
    app.lsp_overlay.completion_filter = "FILE".to_string();
    app.lsp_overlay.completion_selected = 8;

    super::lsp::filter_completions(&mut app);

    assert_eq!(app.lsp_overlay.completion_filter, "file");
    assert_eq!(
        app.lsp_overlay.completion_items,
        vec!["ReadFile".to_string(), "write_file".to_string()]
    );
    assert!(app.lsp_overlay.completion_visible);
    assert_eq!(app.lsp_overlay.completion_selected, 1);
}

#[test]
fn filter_completions_hides_empty_result() {
    let mut app = app();
    app.lsp_overlay.all_completions = vec!["Alpha".to_string()];
    app.lsp_overlay.completion_filter = "zzz".to_string();
    app.lsp_overlay.completion_selected = 4;

    super::lsp::filter_completions(&mut app);

    assert!(app.lsp_overlay.completion_items.is_empty());
    assert!(!app.lsp_overlay.completion_visible);
    assert_eq!(app.lsp_overlay.completion_selected, 0);
}

#[test]
fn disable_lsp_detaches_state_and_marks_status() {
    let mut app = app();
    app.lsp_disabled = false;
    app.lsp_overlay.show_hover("hover".to_string());
    app.lsp_overlay.all_completions = vec!["item".to_string()];
    app.lsp_overlay.completion_items = vec!["item".to_string()];
    app.lsp_overlay.completion_visible = true;
    app.lsp_overlay_path = Some("/tmp/main.rs".to_string());
    app.lsp_progress.entry("rust-analyzer".to_string()).or_default().insert(
        "token".to_string(),
        crate::app::LspProgress { title: "index".to_string(), message: None, percentage: Some(10) },
    );

    super::lsp::disable_lsp(&mut app);

    assert!(app.lsp_disabled);
    assert!(!app.lsp_overlay.hover_visible);
    assert!(!app.lsp_overlay.completion_visible);
    assert!(app.lsp_progress.is_empty());
    assert_eq!(app.lsp_status.as_deref(), Some("LSP 已禁用"));
}

#[test]
fn tick_when_disabled_only_advances_spinner() {
    let mut app = app();
    app.lsp_disabled = true;
    let initial = app.spinner_frame;

    let _ = super::lsp::tick(&mut app);

    assert_eq!(app.spinner_frame, initial.wrapping_add(1));
}
