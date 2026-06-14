use super::file_search::file_search_overlay;
use iced::Length;

#[test]
fn file_search_overlay_renders_empty_state_with_trimmed_query() {
    let mut app = crate::app::App::new().0;
    app.file_search_query = "  src\\main  ".to_string();

    let overlay = file_search_overlay(&app);

    assert_eq!(overlay.as_widget().size().width, Length::Fixed(320.0));
}

#[test]
fn file_search_overlay_renders_ranked_file_entries_and_project_relative_paths() {
    let mut app = crate::app::App::new().0;
    app.project_path = Some("/repo".to_string());
    app.file_search_query = "main".to_string();
    app.file_search_selected_index = 1;
    app.set_file_index(
        "/repo",
        vec![
            "/repo/src/main.rs".to_string(),
            "/repo/src/bin/tool.rs".to_string(),
            "/other/main.txt".to_string(),
        ],
    );

    let overlay = file_search_overlay(&app);

    assert_eq!(overlay.as_widget().size().width, Length::Fixed(320.0));
}

#[test]
fn file_search_overlay_limits_large_result_sets() {
    let mut app = crate::app::App::new().0;
    app.project_path = Some("/repo".to_string());
    app.file_search_query = "file".to_string();
    app.set_file_index("/repo", (0..30).map(|i| format!("src/file_{i}.rs")).collect());

    let overlay = file_search_overlay(&app);

    assert_eq!(overlay.as_widget().size().width, Length::Fixed(320.0));
}
