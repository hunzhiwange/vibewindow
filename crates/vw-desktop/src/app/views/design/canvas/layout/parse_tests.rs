use std::collections::HashMap;

#[test]
fn parse_layout_accepts_row_and_column_aliases() {
    assert!(matches!(
        super::parse_layout(&Some("row".to_string())),
        Some(crate::app::views::design::canvas::types::LayoutDirection::Horizontal)
    ));
    assert!(matches!(
        super::parse_layout(&Some("column".to_string())),
        Some(crate::app::views::design::canvas::types::LayoutDirection::Vertical)
    ));
    assert!(super::parse_layout(&Some("grid".to_string())).is_none());
}

#[test]
fn parse_padding_follows_css_shorthand_order() {
    let padding = super::parse_padding(&Some(serde_json::json!([1, 2, 3, 4])), &HashMap::new(), None);
    assert_eq!((padding.top, padding.right, padding.bottom, padding.left), (1.0, 2.0, 3.0, 4.0));
}

#[test]
fn parse_gap_returns_zero_for_invalid_values() {
    assert_eq!(super::parse_gap(&Some(serde_json::json!("wide")), &HashMap::new(), None), 0.0);
}
