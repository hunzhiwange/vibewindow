use super::tool_text_support::{
    ToolTextTarget, chat_text_font, chat_text_font_name, chat_text_line_height,
    chat_text_selection_color, is_safe_for_text_editor, read_only_text_style,
    selected_chat_text_for_target, tool_inline_text_editor, tool_text_editor, tool_text_key,
    tool_text_style, tool_text_style_with_danger,
};
use crate::app::App;
use iced::widget::text_editor;
use iced::{Color, Theme};

#[test]
fn tool_text_key_packs_indices_without_collisions_for_adjacent_values() {
    assert_ne!(tool_text_key(1, 2, 3), tool_text_key(1, 2, 4));
    assert_ne!(tool_text_key(1, 2, 3), tool_text_key(1, 3, 3));
    assert_eq!(tool_text_key(0, 0, 0), 0);
}

#[test]
fn text_editor_safety_rejects_oversized_lines() {
    assert!(is_safe_for_text_editor("short\ntext"));
    assert!(!is_safe_for_text_editor(&"x".repeat(2_001)));
    assert!(!is_safe_for_text_editor(&"x".repeat(20_001)));
    assert!(is_safe_for_text_editor(""));
}

#[test]
fn chat_text_font_name_is_configured() {
    assert!(!chat_text_font_name().is_empty());
    let _ = chat_text_font();
    match chat_text_line_height() {
        iced::widget::text::LineHeight::Relative(value) => assert_eq!(value, 1.6),
        _ => panic!("chat text line height should be relative"),
    }
}

#[test]
fn text_styles_use_theme_selection_and_danger_color() {
    let theme = Theme::Dark;
    let selection = chat_text_selection_color(&theme);
    assert!(selection.a > 0.0);

    let readonly = read_only_text_style(&theme, Color::from_rgb8(1, 2, 3));
    assert_eq!(readonly.value, Color::from_rgb8(1, 2, 3));
    assert_eq!(readonly.border.width, 0.0);

    let normal = tool_text_style(&theme, text_editor::Status::Active);
    let danger = tool_text_style_with_danger(&theme, text_editor::Status::Active, true);
    assert_ne!(normal.value, danger.value);
}

#[test]
fn text_editor_lookup_returns_none_for_missing_targets_and_some_for_present_targets() {
    let mut app = App::new().0;

    assert!(
        tool_text_editor(
            &app,
            ToolTextTarget::ToolCardText { msg_idx: 1, tool_idx: 2, text_idx: 3 },
            "JetBrains Mono",
            12.0,
            false,
            false,
        )
        .is_none()
    );
    app.chat_tool_text_editors
        .insert(tool_text_key(1, 2, 3), text_editor::Content::with_text("hello"));
    app.chat_special_text_editors
        .insert((7_u64 << 32) | 8, text_editor::Content::with_text("special"));

    assert!(
        tool_text_editor(
            &app,
            ToolTextTarget::ToolCardText { msg_idx: 1, tool_idx: 2, text_idx: 3 },
            "JetBrains Mono",
            12.0,
            true,
            false,
        )
        .is_some()
    );
    assert!(
        tool_inline_text_editor(
            &app,
            ToolTextTarget::SpecialMessageText { msg_idx: 7, text_idx: 8 },
            chat_text_font_name(),
            12.0,
            |theme| theme.palette().text,
        )
        .is_some()
    );
    assert_eq!(selected_chat_text_for_target(&app, 1_u64 << 32), None);
}

#[test]
fn oversized_text_targets_fall_back_to_preview_elements() {
    let mut app = App::new().0;
    app.chat_tool_text_editors
        .insert(tool_text_key(0, 0, 0), text_editor::Content::with_text(&"x".repeat(20_001)));

    assert!(
        tool_text_editor(
            &app,
            ToolTextTarget::ToolCardText { msg_idx: 0, tool_idx: 0, text_idx: 0 },
            "JetBrains Mono",
            12.0,
            false,
            true,
        )
        .is_some()
    );
    assert!(
        tool_inline_text_editor(
            &app,
            ToolTextTarget::ToolCardText { msg_idx: 0, tool_idx: 0, text_idx: 0 },
            "JetBrains Mono",
            12.0,
            |theme| theme.palette().text,
        )
        .is_some()
    );
}
