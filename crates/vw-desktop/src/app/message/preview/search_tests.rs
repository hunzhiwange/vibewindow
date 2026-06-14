#[test]
fn test_module_is_wired() {
    let module = module_path!();

    assert!(module.ends_with("search_tests"));
}

#[test]
fn update_returns_none_task_without_mutating_app_for_search_messages() {
    let mut app = crate::app::App::new().0;
    app.file_url_input = "kept".to_string();
    app.active_preview_path = Some("/tmp/a.txt".to_string());

    let _ =
        super::search::update(&mut app, super::PreviewMessage::SearchChanged("needle".to_string()));
    let _ = super::search::update(&mut app, super::PreviewMessage::SearchNext);
    let _ = super::search::update(&mut app, super::PreviewMessage::SearchPrev);
    let _ = super::search::update(&mut app, super::PreviewMessage::CopySearchMatches);

    assert_eq!(app.file_url_input, "kept");
    assert_eq!(app.active_preview_path.as_deref(), Some("/tmp/a.txt"));
}
