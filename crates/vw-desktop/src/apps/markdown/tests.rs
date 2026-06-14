use crate::app::App;
use crate::app::message::MarkdownToolMessage;

#[test]
fn markdown_view_delegates_to_markdown_tool_view() {
    let (app, _task) = App::new();

    let _element = super::view(&app);
}

#[test]
fn markdown_update_delegates_to_markdown_tool_update() {
    let (mut app, _task) = App::new();

    let _task = super::update(&mut app, MarkdownToolMessage::Clear);
}
