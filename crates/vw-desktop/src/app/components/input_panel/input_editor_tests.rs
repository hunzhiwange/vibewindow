use super::input_editor::{
    INPUT_LINE_HEIGHT, INPUT_MAX_LINES, INPUT_MIN_LINES, INPUT_VERTICAL_PADDING,
    binding_from_key_press, build_input_editor, input_context_menu,
};
use crate::app::{Message, message};
use iced::keyboard;
use iced::widget::text_editor::{Binding, Content, KeyPress, Status};
use iced::{Length, Theme};

fn key_press(key: keyboard::Key, modifiers: keyboard::Modifiers) -> KeyPress {
    KeyPress {
        key: key.clone(),
        modified_key: key,
        physical_key: keyboard::key::Physical::Code(keyboard::key::Code::KeyA),
        modifiers,
        text: None,
        status: Status::Focused { is_hovered: false },
    }
}

fn named_key(named: keyboard::key::Named) -> keyboard::Key {
    keyboard::Key::Named(named)
}

fn assert_custom_chat(
    binding: Option<Binding<Message>>,
    expected: fn(&message::ChatMessage) -> bool,
) {
    match binding {
        Some(Binding::Custom(Message::Chat(chat))) => assert!(expected(&chat)),
        _ => panic!("expected custom chat binding"),
    }
}

#[test]
fn editor_height_clamps_between_minimum_and_maximum_lines() {
    let app = crate::app::App::new().0;

    let (_, empty_height) = build_input_editor(&app, &Content::new(), false, false);
    assert_eq!(
        empty_height,
        INPUT_MIN_LINES as f32 * INPUT_LINE_HEIGHT + INPUT_VERTICAL_PADDING * 2.0
    );

    let many_lines = Content::with_text("1\n2\n3\n4\n5\n6\n7\n8\n9\n10");
    let (_, max_height) = build_input_editor(&app, &many_lines, true, true);
    assert_eq!(
        max_height,
        INPUT_MAX_LINES as f32 * INPUT_LINE_HEIGHT + INPUT_VERTICAL_PADDING * 2.0
    );
}

#[test]
fn build_input_editor_wraps_context_menu_when_open() {
    let mut app = crate::app::App::new().0;
    app.input_context_menu_pos = Some((10.0, 20.0));
    app.input_context_menu_open = true;
    let content = Content::with_text("hello\nworld");

    let (element, height) = build_input_editor(&app, &content, false, false);

    assert_eq!(element.as_widget().size().width, Length::Fill);
    assert_eq!(height, 3.0 * INPUT_LINE_HEIGHT + INPUT_VERTICAL_PADDING * 2.0);
}

#[test]
fn input_context_menu_contains_four_actions() {
    let menu = input_context_menu();

    assert_eq!(menu.as_widget().size().width, Length::Shrink);
}

#[test]
fn file_search_key_bindings_capture_navigation_and_selection() {
    let mut app = crate::app::App::new().0;
    app.show_file_search = true;

    assert_custom_chat(
        binding_from_key_press(
            &app,
            key_press(named_key(keyboard::key::Named::ArrowUp), keyboard::Modifiers::NONE),
        ),
        |message| matches!(message, message::ChatMessage::FileSearchNavigateUp),
    );
    assert_custom_chat(
        binding_from_key_press(
            &app,
            key_press(named_key(keyboard::key::Named::ArrowDown), keyboard::Modifiers::NONE),
        ),
        |message| matches!(message, message::ChatMessage::FileSearchNavigateDown),
    );
    assert_custom_chat(
        binding_from_key_press(
            &app,
            key_press(named_key(keyboard::key::Named::Tab), keyboard::Modifiers::NONE),
        ),
        |message| matches!(message, message::ChatMessage::FileSearchSelectCurrent),
    );
    assert_custom_chat(
        binding_from_key_press(
            &app,
            key_press(named_key(keyboard::key::Named::Escape), keyboard::Modifiers::NONE),
        ),
        |message| matches!(message, message::ChatMessage::FileSearchInputChanged(value) if value.is_empty()),
    );
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn native_enter_selects_file_search_only_without_modifiers() {
    let mut app = crate::app::App::new().0;
    app.show_file_search = true;

    assert_custom_chat(
        binding_from_key_press(
            &app,
            key_press(named_key(keyboard::key::Named::Enter), keyboard::Modifiers::NONE),
        ),
        |message| matches!(message, message::ChatMessage::FileSearchSelectCurrent),
    );

    let shifted = binding_from_key_press(
        &app,
        key_press(named_key(keyboard::key::Named::Enter), keyboard::Modifiers::SHIFT),
    );
    assert!(matches!(shifted, Some(Binding::Enter)));
}

#[test]
fn regular_key_bindings_capture_paste_and_send_shortcuts() {
    let app = crate::app::App::new().0;

    assert_custom_chat(
        binding_from_key_press(
            &app,
            key_press(keyboard::Key::Character("v".into()), keyboard::Modifiers::COMMAND),
        ),
        |message| matches!(message, message::ChatMessage::PasteIntoInput),
    );

    #[cfg(not(target_arch = "wasm32"))]
    assert_custom_chat(
        binding_from_key_press(
            &app,
            key_press(named_key(keyboard::key::Named::Enter), keyboard::Modifiers::NONE),
        ),
        |message| matches!(message, message::ChatMessage::SendPressed),
    );

    let alt_paste = binding_from_key_press(
        &app,
        key_press(
            keyboard::Key::Character("v".into()),
            keyboard::Modifiers::COMMAND | keyboard::Modifiers::ALT,
        ),
    );
    assert!(!matches!(alt_paste, Some(Binding::Custom(_))));
}

#[test]
fn editor_style_callbacks_can_be_invoked_through_construction_inputs() {
    let app = crate::app::App::new().0;
    let content = Content::with_text("one\ntwo\nthree\nfour");
    let (_, height) = build_input_editor(&app, &content, true, false);

    assert_eq!(height, 4.0 * INPUT_LINE_HEIGHT + INPUT_VERTICAL_PADDING * 2.0);
    assert!(Theme::Dark.palette().background.r < Theme::Light.palette().background.r);
}
