use super::*;
use serde_json::json;

#[test]
fn test_remove_default_letter_spacing() {
    let mut tree = json!({
        "name": "Text",
        "letterSpacing": {"units": "PERCENT", "value": 0.0},
        "fontSize": 16.0
    });

    remove_default_text_properties(&mut tree).unwrap();

    assert!(tree.get("letterSpacing").is_none());
    assert_eq!(tree.get("fontSize").unwrap().as_f64(), Some(16.0));
}

#[test]
fn test_remove_default_line_height() {
    let mut tree = json!({
        "name": "Text",
        "lineHeight": {"units": "PERCENT", "value": 100.0},
        "fontSize": 16.0
    });

    remove_default_text_properties(&mut tree).unwrap();

    assert!(tree.get("lineHeight").is_none());
    assert_eq!(tree.get("fontSize").unwrap().as_f64(), Some(16.0));
}

#[test]
fn test_remove_both_defaults() {
    let mut tree = json!({
        "name": "Text",
        "letterSpacing": {"units": "PERCENT", "value": 0.0},
        "lineHeight": {"units": "PERCENT", "value": 100.0},
        "fontSize": 14.0
    });

    remove_default_text_properties(&mut tree).unwrap();

    assert!(tree.get("letterSpacing").is_none());
    assert!(tree.get("lineHeight").is_none());
    assert_eq!(tree.get("fontSize").unwrap().as_f64(), Some(14.0));
}

#[test]
fn test_preserve_non_default_letter_spacing() {
    let mut tree = json!({
        "name": "Text",
        "letterSpacing": {"units": "PERCENT", "value": 5.0},
        "fontSize": 16.0
    });

    remove_default_text_properties(&mut tree).unwrap();

    // 应保留非默认字母间距
    assert!(tree.get("letterSpacing").is_some());
    assert_eq!(tree["letterSpacing"]["value"].as_f64(), Some(5.0));
}

#[test]
fn test_preserve_non_default_line_height() {
    let mut tree = json!({
        "name": "Text",
        "lineHeight": {"units": "PERCENT", "value": 120.0},
        "fontSize": 16.0
    });

    remove_default_text_properties(&mut tree).unwrap();

    // 应保留非默认 lineHeight
    assert!(tree.get("lineHeight").is_some());
    assert_eq!(tree["lineHeight"]["value"].as_f64(), Some(120.0));
}

#[test]
fn test_preserve_pixels_units() {
    let mut tree = json!({
        "name": "Text",
        "letterSpacing": {"units": "PIXELS", "value": 0.0},
        "lineHeight": {"units": "PIXELS", "value": 100.0},
        "fontSize": 16.0
    });

    remove_default_text_properties(&mut tree).unwrap();

    // 即使使用默认值，也应保留非 PERCENT 单位
    assert!(tree.get("letterSpacing").is_some());
    assert!(tree.get("lineHeight").is_some());
}

#[test]
fn test_nested_objects() {
    let mut tree = json!({
        "children": [
            {
                "name": "Text1",
                "letterSpacing": {"units": "PERCENT", "value": 0.0}
            },
            {
                "name": "Text2",
                "lineHeight": {"units": "PERCENT", "value": 100.0}
            }
        ]
    });

    remove_default_text_properties(&mut tree).unwrap();

    // 两个嵌套默认值都应该被删除
    assert!(tree["children"][0].get("letterSpacing").is_none());
    assert!(tree["children"][1].get("lineHeight").is_none());
}

#[test]
fn test_no_text_properties() {
    let mut tree = json!({
        "name": "Rectangle",
        "width": 100,
        "height": 200
    });

    remove_default_text_properties(&mut tree).unwrap();

    // 没有文本属性的树应该保持不变
    assert_eq!(tree.get("name").unwrap().as_str(), Some("Rectangle"));
    assert_eq!(tree.get("width").unwrap().as_i64(), Some(100));
}

#[test]
fn test_deeply_nested() {
    let mut tree = json!({
        "document": {
            "children": [
                {
                    "type": "TEXT",
                    "letterSpacing": {"units": "PERCENT", "value": 0.0},
                    "lineHeight": {"units": "PERCENT", "value": 100.0}
                }
            ]
        }
    });

    remove_default_text_properties(&mut tree).unwrap();

    let text_node = &tree["document"]["children"][0];
    assert!(text_node.get("letterSpacing").is_none());
    assert!(text_node.get("lineHeight").is_none());
    assert_eq!(text_node["type"].as_str(), Some("TEXT"));
}

#[test]
fn test_root_array_is_transformed() {
    let mut tree = json!([
        {
            "name": "Text1",
            "letterSpacing": {"units": "PERCENT", "value": 0.0}
        },
        {
            "name": "Text2",
            "lineHeight": {"units": "PERCENT", "value": 100.0}
        },
        {
            "name": "Text3",
            "letterSpacing": {"units": "PERCENT", "value": 2.0},
            "lineHeight": {"units": "PERCENT", "value": 101.0}
        }
    ]);

    remove_default_text_properties(&mut tree).unwrap();

    assert!(tree[0].get("letterSpacing").is_none());
    assert!(tree[1].get("lineHeight").is_none());
    assert_eq!(tree[2]["letterSpacing"]["value"].as_f64(), Some(2.0));
    assert_eq!(tree[2]["lineHeight"]["value"].as_f64(), Some(101.0));
}

#[test]
fn test_primitive_roots_are_accepted_without_changes() {
    let mut string_tree = json!("letterSpacing");
    let mut number_tree = json!(100.0);
    let mut bool_tree = json!(true);
    let mut null_tree = json!(null);

    remove_default_text_properties(&mut string_tree).unwrap();
    remove_default_text_properties(&mut number_tree).unwrap();
    remove_default_text_properties(&mut bool_tree).unwrap();
    remove_default_text_properties(&mut null_tree).unwrap();

    assert_eq!(string_tree, json!("letterSpacing"));
    assert_eq!(number_tree, json!(100.0));
    assert_eq!(bool_tree, json!(true));
    assert_eq!(null_tree, json!(null));
}

#[test]
fn test_preserve_malformed_text_property_values() {
    let mut tree = json!({
        "letterSpacing": {"units": "PERCENT"},
        "lineHeight": {"value": 100.0},
        "children": [
            {"letterSpacing": {"units": "PERCENT", "value": "0"}},
            {"lineHeight": {"units": "PERCENT", "value": "100"}},
            {"letterSpacing": "0 PERCENT"},
            {"lineHeight": 100.0}
        ]
    });

    let original = tree.clone();

    remove_default_text_properties(&mut tree).unwrap();

    assert_eq!(tree, original);
}

#[test]
fn test_default_detection_tolerates_tiny_float_drift() {
    let mut tree = json!({
        "letterSpacing": {"units": "PERCENT", "value": 0.000000000009},
        "lineHeight": {"units": "PERCENT", "value": 100.000000000009},
        "children": [
            {
                "letterSpacing": {"units": "PERCENT", "value": 0.0000000002},
                "lineHeight": {"units": "PERCENT", "value": 100.0000000002}
            }
        ]
    });

    remove_default_text_properties(&mut tree).unwrap();

    assert!(tree.get("letterSpacing").is_none());
    assert!(tree.get("lineHeight").is_none());
    assert_eq!(tree["children"][0]["letterSpacing"]["value"].as_f64(), Some(0.0000000002));
    assert_eq!(tree["children"][0]["lineHeight"]["value"].as_f64(), Some(100.0000000002));
}

#[test]
fn test_is_default_letter_spacing() {
    assert!(is_default_letter_spacing(&json!({
        "units": "PERCENT",
        "value": 0.0
    })));

    assert!(!is_default_letter_spacing(&json!({
        "units": "PERCENT",
        "value": 5.0
    })));

    assert!(!is_default_letter_spacing(&json!({
        "units": "PIXELS",
        "value": 0.0
    })));

    assert!(!is_default_letter_spacing(&json!({
        "units": "PERCENT"
    })));

    assert!(!is_default_letter_spacing(&json!({
        "value": 0.0
    })));

    assert!(!is_default_letter_spacing(&json!({
        "units": "PERCENT",
        "value": "0"
    })));

    assert!(!is_default_letter_spacing(&json!(0.0)));
}

#[test]
fn test_is_default_line_height() {
    assert!(is_default_line_height(&json!({
        "units": "PERCENT",
        "value": 100.0
    })));

    assert!(!is_default_line_height(&json!({
        "units": "PERCENT",
        "value": 120.0
    })));

    assert!(!is_default_line_height(&json!({
        "units": "PIXELS",
        "value": 100.0
    })));

    assert!(!is_default_line_height(&json!({
        "units": "PERCENT"
    })));

    assert!(!is_default_line_height(&json!({
        "value": 100.0
    })));

    assert!(!is_default_line_height(&json!({
        "units": "PERCENT",
        "value": "100"
    })));

    assert!(!is_default_line_height(&json!(100.0)));
}
