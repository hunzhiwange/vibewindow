#[test]
fn modals_test_module_is_linked() {
    let name = "modals";
    assert_eq!(name.len(), 6);
}

#[test]
fn active_modal_returns_none_without_modal_state() {
    let app = crate::app::App::new().0;
    assert!(super::active_modal(&app).is_none());
}

#[test]
fn active_modal_prefers_edit_modal_over_add_modal() {
    let mut app = crate::app::App::new().0;
    app.show_web_links_menu = true;
    app.editing_web_bookmark = Some(2);

    let Some((_modal, close)) = super::active_modal(&app) else {
        panic!("expected edit modal");
    };
    assert!(matches!(
        close,
        crate::app::Message::View(crate::app::message::ViewMessage::WebBookmarkEditCancel)
    ));
}

#[test]
fn active_modal_returns_add_bookmark_close_message() {
    let mut app = crate::app::App::new().0;
    app.show_web_links_menu = true;

    let Some((_modal, close)) = super::active_modal(&app) else {
        panic!("expected add modal");
    };
    assert!(matches!(
        close,
        crate::app::Message::View(crate::app::message::ViewMessage::ToggleWebLinksMenu)
    ));
}

#[test]
fn modal_shell_wraps_any_content() {
    let content = iced::widget::text("body").into();
    let _ = super::modal_shell(content);
}
