use super::common::{clone_page_elements, parse_fills, upsert_variable_value};
use crate::app::views::design::models::{DesignElement, VariableValue};
use crate::app::views::design::properties::fill::types::{FillItem, FillObject};

fn element(id: &str, reference: Option<&str>) -> DesignElement {
    DesignElement {
        id: id.to_string(),
        kind: "rect".to_string(),
        reference: reference.map(ToString::to_string),
        group_id: 1,
        ..serde_json::from_value(serde_json::json!({})).unwrap()
    }
}

#[test]
fn test_module_is_wired() {
    let module = module_path!();

    assert!(module.ends_with("common_tests"));
}

#[test]
fn upsert_variable_value_adds_updates_and_removes_empty_values() {
    let mut values = Vec::<VariableValue>::new();

    upsert_variable_value(&mut values, None, "#fff".to_string());
    upsert_variable_value(&mut values, Some(" Dark "), "#000".to_string());
    assert_eq!(values.len(), 2);
    assert_eq!(values[0].value, "#fff");
    assert_eq!(values[1].theme.as_ref().unwrap().mode, "Dark");

    upsert_variable_value(&mut values, Some("dark"), "#111".to_string());
    assert_eq!(values[1].value, "#111");

    upsert_variable_value(&mut values, None, String::new());
    assert_eq!(values.len(), 1);
    assert_eq!(values[0].value, "#111");

    upsert_variable_value(&mut values, Some("unused"), String::new());
    assert_eq!(values.len(), 1);
}

#[test]
fn clone_page_elements_rewrites_ids_groups_and_internal_refs() {
    let mut root = element("root", None);
    root.children.push(element("child", Some("root")));
    let cloned = clone_page_elements(&[root], 7);

    assert_eq!(cloned.len(), 1);
    assert_eq!(cloned[0].group_id, 7);
    assert_ne!(cloned[0].id, "root");
    assert_eq!(cloned[0].children[0].group_id, 7);
    assert_ne!(cloned[0].children[0].id, "child");
    assert_eq!(cloned[0].children[0].reference, Some(cloned[0].id.clone()));
}

#[test]
fn parse_fills_accepts_lists_single_items_strings_and_legacy_solid() {
    let list = parse_fills(&serde_json::json!(["#fff", {"type": "solid", "color": "#000"}]));
    assert_eq!(list.len(), 2);

    let single = parse_fills(&serde_json::json!({"type": "solid", "color": "#123"}));
    assert!(matches!(single.as_slice(), [FillItem::Object(FillObject::Solid { .. })]));

    let string_fill = parse_fills(&serde_json::json!("#abc"));
    assert!(matches!(string_fill.as_slice(), [FillItem::Color(color)] if color == "#abc"));

    let legacy = parse_fills(&serde_json::json!({"color": "#def", "enabled": false}));
    assert!(
        matches!(legacy.as_slice(), [FillItem::Object(FillObject::Solid { color, .. })] if color == "#def")
    );

    assert!(parse_fills(&serde_json::json!(false)).is_empty());
}
