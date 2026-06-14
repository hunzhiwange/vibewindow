use super::*;
use iced::Element;
use iced::widget::text;

#[test]
fn field_row_accepts_query_rule_control() {
    let element: Element<'_, Message> = field_row("pattern", "匹配关键字或模式。", text("bug"));
    drop(element);
}

#[test]
fn source_keeps_empty_state_and_rule_editor_paths() {
    let source = include_str!("system_settings_query_classification.rs");

    assert!(source.contains("暂无分类规则"));
    assert!(source.contains("QueryClassificationMessage::AddRule"));
    assert!(source.contains("QueryClassificationMessage::RemoveRule(idx)"));
    assert!(source.contains("QueryClassificationMessage::PatternChanged(idx, value)"));
    assert!(source.contains("QueryClassificationMessage::CategoryChanged(idx, value)"));
    assert!(source.contains("QueryClassificationMessage::PriorityChanged(idx, value)"));
}
