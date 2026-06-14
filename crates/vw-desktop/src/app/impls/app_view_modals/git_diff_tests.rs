use super::*;

fn test_app() -> App {
    App::new().0
}

fn root() -> iced::Element<'static, Message> {
    iced::widget::container(iced::widget::text("root")).into()
}

#[test]
fn with_git_diff_comment_returns_root_when_comment_modal_is_absent() {
    let app = test_app();

    let _element: iced::Element<'_, Message> = with_git_diff_comment(&app, root());
}

#[test]
fn with_git_diff_overlays_builds_empty_overlay_layer_when_modals_are_absent() {
    let app = test_app();

    let _element: iced::Element<'_, Message> = with_git_diff_overlays(&app, root());
}
