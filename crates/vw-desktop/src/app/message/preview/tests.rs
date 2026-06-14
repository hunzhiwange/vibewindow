#[test]
fn test_module_is_wired() {
    let module = module_path!();

    assert!(module.ends_with("tests"));
}

#[test]
fn dismiss_preview_popup_menus_clears_all_preview_menu_state() {
    let mut app = crate::app::App::new().0;
    app.show_preview_context_menu = true;
    app.preview_context_menu_pos = Some((1.0, 2.0));
    app.preview_nav_popup = Some((
        "/tmp".to_string(),
        3.0,
        4.0,
        vec![("src".to_string(), true), ("main.rs".to_string(), false)],
    ));
    app.preview_tab_menu_path = Some("/tmp/main.rs".to_string());
    app.preview_tab_menu_pos = Some(iced::Point::new(5.0, 6.0));

    super::dismiss_preview_popup_menus(&mut app);

    assert!(!app.show_preview_context_menu);
    assert!(app.preview_context_menu_pos.is_none());
    assert!(app.preview_nav_popup.is_none());
    assert!(app.preview_tab_menu_path.is_none());
    assert!(app.preview_tab_menu_pos.is_none());
}

#[test]
fn update_routes_search_message_without_side_effects() {
    let mut app = crate::app::App::new().0;
    app.active_preview_path = Some("/tmp/a.txt".to_string());

    let _ = super::update(&mut app, super::PreviewMessage::SearchNext);

    assert_eq!(app.active_preview_path.as_deref(), Some("/tmp/a.txt"));
}
