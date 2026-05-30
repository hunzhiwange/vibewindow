use super::todo_views::todo_tool_card_padding;

#[test]
fn todo_views_test_module_is_linked() {
    assert_eq!("todo_views", "todo_views");
}

#[test]
fn todo_tool_card_padding_keeps_content_away_from_border() {
    let padding = todo_tool_card_padding();

    assert_eq!(padding.top, 10.0);
    assert_eq!(padding.right, 12.0);
    assert_eq!(padding.bottom, 10.0);
    assert_eq!(padding.left, 12.0);
}
