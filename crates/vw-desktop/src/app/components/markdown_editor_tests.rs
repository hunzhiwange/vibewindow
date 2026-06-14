use super::markdown_editor::{MarkdownViewMode, mode_switch, view};
use crate::app::{App, Message};
use iced::Theme;
use iced::widget::{markdown, text_editor};

fn test_app() -> App {
    App::new().0
}

fn keep(element: iced::Element<'_, Message>) {
    std::hint::black_box(element);
}

fn on_mode(mode: MarkdownViewMode) -> Message {
    Message::MarkdownTool(crate::app::message::markdown_tool::MarkdownToolMessage::SetViewMode(
        mode,
    ))
}

fn on_action(action: text_editor::Action) -> Message {
    Message::MarkdownTool(crate::app::message::markdown_tool::MarkdownToolMessage::EditorAction(
        action,
    ))
}

#[test]
fn task_744_test_module_is_wired() {
    let path = std::path::Path::new(file!());
    assert!(path.ends_with("markdown_editor_tests.rs"));
}

#[test]
fn markdown_view_mode_is_copy_eq_and_debuggable() {
    assert_eq!(MarkdownViewMode::Edit, MarkdownViewMode::Edit);
    assert_ne!(MarkdownViewMode::Edit, MarkdownViewMode::Preview);
    assert_eq!(format!("{:?}", MarkdownViewMode::Split), "Split");
}

#[test]
fn mode_switch_builds_for_each_selected_mode() {
    keep(mode_switch(MarkdownViewMode::Edit, on_mode));
    keep(mode_switch(MarkdownViewMode::Preview, on_mode));
    keep(mode_switch(MarkdownViewMode::Split, on_mode));
}

#[test]
fn view_builds_edit_preview_and_split_modes_in_light_theme() {
    let app = test_app();
    let editor = text_editor::Content::with_text("# Title\n\n- item");
    let preview = markdown::Content::parse(&editor.text());

    keep(view(&editor, &preview, &Theme::Light, &app, MarkdownViewMode::Edit, on_action));
    keep(view(&editor, &preview, &Theme::Light, &app, MarkdownViewMode::Preview, on_action));
    keep(view(&editor, &preview, &Theme::Light, &app, MarkdownViewMode::Split, on_action));
}

#[test]
fn view_builds_dark_theme_highlighter_path() {
    let app = test_app();
    let editor = text_editor::Content::with_text("```rust\nfn main() {}\n```");
    let preview = markdown::Content::parse(&editor.text());

    keep(view(&editor, &preview, &Theme::Dark, &app, MarkdownViewMode::Split, on_action));
}
