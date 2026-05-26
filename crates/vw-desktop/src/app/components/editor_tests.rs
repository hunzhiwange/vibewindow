use super::editor::Editor;

#[test]
fn editor_starts_with_initial_content() {
    let editor = Editor::new("fn main() {}", "rust");

    assert_eq!(editor.content(), "fn main() {}");
    assert!(!editor.can_undo());
    assert!(!editor.can_redo());
}

#[test]
fn editor_setters_are_callable() {
    let mut editor = Editor::new("", "rust");

    editor.set_font_size(14.0);
    editor.set_line_height(1.4);
    editor.set_ui_language(iced_code_editor::i18n::Language::ChineseSimplified);
}
