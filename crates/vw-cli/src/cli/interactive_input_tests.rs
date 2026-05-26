use super::interactive_input::{InputEditResult, apply_key_to_input, insert_newline};
use crossterm::event::KeyCode;

#[test]
fn apply_key_to_input_handles_unicode_editing() {
    let mut input = String::from("你b");
    let mut cursor = 1;

    assert_eq!(
        apply_key_to_input(&mut input, &mut cursor, KeyCode::Char('好')),
        InputEditResult::Updated
    );
    assert_eq!(input, "你好b");
    assert_eq!(cursor, 2);

    assert_eq!(
        apply_key_to_input(&mut input, &mut cursor, KeyCode::Backspace),
        InputEditResult::Updated
    );
    assert_eq!(input, "你b");
    assert_eq!(cursor, 1);

    insert_newline(&mut input, &mut cursor);
    assert_eq!(input, "你\nb");
}
