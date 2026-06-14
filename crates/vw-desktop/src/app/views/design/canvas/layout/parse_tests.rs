use std::collections::HashMap;

use crate::app::views::design::models::{ThemeCondition, VariableDef, VariableValue};

fn variables() -> HashMap<String, VariableDef> {
    HashMap::from([
        (
            "-space".to_string(),
            VariableDef {
                kind: "number".to_string(),
                collection: None,
                value: vec![VariableValue { value: "12".to_string(), theme: None }],
            },
        ),
        (
            "-theme-space".to_string(),
            VariableDef {
                kind: "number".to_string(),
                collection: None,
                value: vec![
                    VariableValue { value: "8".to_string(), theme: None },
                    VariableValue {
                        value: "24".to_string(),
                        theme: Some(ThemeCondition { mode: "Dark".to_string() }),
                    },
                ],
            },
        ),
    ])
}

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
    let padding =
        super::parse_padding(&Some(serde_json::json!([1, 2, 3, 4])), &HashMap::new(), None);
    assert_eq!((padding.top, padding.right, padding.bottom, padding.left), (1.0, 2.0, 3.0, 4.0));
}

#[test]
fn parse_padding_accepts_number_string_array_and_variables() {
    let vars = variables();

    let one = super::parse_padding(&Some(serde_json::json!(6)), &vars, None);
    assert_eq!((one.top, one.right, one.bottom, one.left), (6.0, 6.0, 6.0, 6.0));

    let two = super::parse_padding(&Some(serde_json::json!("1 2")), &vars, None);
    assert_eq!((two.top, two.right, two.bottom, two.left), (1.0, 2.0, 1.0, 2.0));

    let three = super::parse_padding(&Some(serde_json::json!("1 2 3")), &vars, None);
    assert_eq!((three.top, three.right, three.bottom, three.left), (1.0, 2.0, 3.0, 2.0));

    let from_array = super::parse_padding(
        &Some(serde_json::json!(["$-space", "3px", "$-theme-space"])),
        &vars,
        Some("Dark"),
    );
    assert_eq!(
        (from_array.top, from_array.right, from_array.bottom, from_array.left),
        (12.0, 3.0, 24.0, 3.0)
    );
}

#[test]
fn parse_padding_returns_zero_for_invalid_shapes() {
    let invalid = super::parse_padding(&Some(serde_json::json!({ "x": 1 })), &HashMap::new(), None);
    assert_eq!((invalid.top, invalid.right, invalid.bottom, invalid.left), (0.0, 0.0, 0.0, 0.0));

    let too_many =
        super::parse_padding(&Some(serde_json::json!([1, 2, 3, 4, 5])), &HashMap::new(), None);
    assert_eq!(
        (too_many.top, too_many.right, too_many.bottom, too_many.left),
        (0.0, 0.0, 0.0, 0.0)
    );
}

#[test]
fn parse_gap_returns_zero_for_invalid_values() {
    assert_eq!(super::parse_gap(&Some(serde_json::json!("wide")), &HashMap::new(), None), 0.0);
}

#[test]
fn parse_gap_accepts_numbers_and_variables() {
    let vars = variables();
    assert_eq!(super::parse_gap(&Some(serde_json::json!(5)), &vars, None), 5.0);
    assert_eq!(
        super::parse_gap(&Some(serde_json::json!("$-theme-space")), &vars, Some("Dark")),
        24.0
    );
    assert_eq!(super::parse_gap(&None, &vars, None), 0.0);
}

#[test]
fn parse_align_mode_accepts_css_aliases() {
    use crate::app::views::design::canvas::types::AlignMode;

    assert!(matches!(
        super::parse_align_mode(&Some("flex-start".to_string())),
        Some(AlignMode::Start)
    ));
    assert!(matches!(super::parse_align_mode(&Some("flex_end".to_string())), Some(AlignMode::End)));
    assert!(matches!(
        super::parse_align_mode(&Some("center".to_string())),
        Some(AlignMode::Center)
    ));
    assert!(matches!(
        super::parse_align_mode(&Some("space-between".to_string())),
        Some(AlignMode::SpaceBetween)
    ));
    assert!(matches!(
        super::parse_align_mode(&Some("space_around".to_string())),
        Some(AlignMode::SpaceAround)
    ));
    assert!(matches!(
        super::parse_align_mode(&Some("space-evenly".to_string())),
        Some(AlignMode::SpaceEvenly)
    ));
    assert!(matches!(
        super::parse_align_mode(&Some("stretch".to_string())),
        Some(AlignMode::Stretch)
    ));
    assert!(super::parse_align_mode(&Some("unknown".to_string())).is_none());
}
