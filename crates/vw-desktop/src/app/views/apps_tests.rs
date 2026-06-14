#[test]
fn apps_test_module_is_linked() {
    let name = "apps";
    assert_eq!(name.len(), 4);
}

#[test]
fn view_builds_without_active_modal() {
    let app = crate::app::App::new().0;
    let _ = super::view(&app);
}

#[test]
fn view_builds_with_add_bookmark_modal_overlay() {
    let mut app = crate::app::App::new().0;
    app.show_web_links_menu = true;
    let _ = super::view(&app);
}

#[test]
fn view_builds_with_edit_bookmark_modal_overlay() {
    let mut app = crate::app::App::new().0;
    app.editing_web_bookmark = Some(0);
    let _ = super::view(&app);
}
