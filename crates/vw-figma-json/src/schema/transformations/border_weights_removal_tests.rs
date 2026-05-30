use super::*;
use serde_json::json;

#[test]
fn test_remove_all_border_weights() {
    let mut tree = json!({
        "name": "Rectangle",
        "borderTopWeight": 1.0,
        "borderBottomWeight": 2.0,
        "borderLeftWeight": 1.5,
        "borderRightWeight": 2.5,
        "visible": true
    });

    remove_border_weights(&mut tree).unwrap();

    assert!(tree.get("borderTopWeight").is_none());
    assert!(tree.get("borderBottomWeight").is_none());
    assert!(tree.get("borderLeftWeight").is_none());
    assert!(tree.get("borderRightWeight").is_none());
    assert_eq!(tree.get("name").unwrap().as_str(), Some("Rectangle"));
    assert_eq!(tree.get("visible").unwrap().as_bool(), Some(true));
}

#[test]
fn test_remove_partial_border_weights() {
    let mut tree = json!({
        "name": "Shape",
        "borderTopWeight": 1.0,
        "borderLeftWeight": 1.0,
        "width": 100
    });

    remove_border_weights(&mut tree).unwrap();

    assert!(tree.get("borderTopWeight").is_none());
    assert!(tree.get("borderLeftWeight").is_none());
    assert_eq!(tree.get("width").unwrap().as_i64(), Some(100));
}

#[test]
fn test_no_border_weights() {
    let mut tree = json!({
        "name": "Circle",
        "radius": 50,
        "visible": true
    });

    remove_border_weights(&mut tree).unwrap();

    // 没有边界权重的树应该保持不变
    assert_eq!(tree.get("name").unwrap().as_str(), Some("Circle"));
    assert_eq!(tree.get("radius").unwrap().as_i64(), Some(50));
    assert_eq!(tree.get("visible").unwrap().as_bool(), Some(true));
}

#[test]
fn test_nested_objects() {
    let mut tree = json!({
        "children": [
            {
                "name": "Child1",
                "borderTopWeight": 1.0,
                "borderBottomWeight": 1.0
            },
            {
                "name": "Child2",
                "borderLeftWeight": 2.0,
                "borderRightWeight": 2.0
            }
        ]
    });

    remove_border_weights(&mut tree).unwrap();

    // 所有嵌套边框权重应被删除
    assert!(tree["children"][0].get("borderTopWeight").is_none());
    assert!(tree["children"][0].get("borderBottomWeight").is_none());
    assert_eq!(tree["children"][0]["name"].as_str(), Some("Child1"));

    assert!(tree["children"][1].get("borderLeftWeight").is_none());
    assert!(tree["children"][1].get("borderRightWeight").is_none());
    assert_eq!(tree["children"][1]["name"].as_str(), Some("Child2"));
}

#[test]
fn test_deeply_nested() {
    let mut tree = json!({
        "document": {
            "children": [
                {
                    "type": "FRAME",
                    "borderTopWeight": 1.0,
                    "borderBottomWeight": 1.0,
                    "borderLeftWeight": 1.0,
                    "borderRightWeight": 1.0
                }
            ]
        }
    });

    remove_border_weights(&mut tree).unwrap();

    let frame = &tree["document"]["children"][0];
    assert!(frame.get("borderTopWeight").is_none());
    assert!(frame.get("borderBottomWeight").is_none());
    assert!(frame.get("borderLeftWeight").is_none());
    assert!(frame.get("borderRightWeight").is_none());
    assert_eq!(frame["type"].as_str(), Some("FRAME"));
}

#[test]
fn test_preserve_other_border_properties() {
    let mut tree = json!({
        "name": "Rectangle",
        "borderTopWeight": 1.0,
        "borderRadius": 5.0,
        "borderColor": "#ff0000"
    });

    remove_border_weights(&mut tree).unwrap();

    // 应保留其他边框属性
    assert!(tree.get("borderTopWeight").is_none());
    assert_eq!(tree.get("borderRadius").unwrap().as_f64(), Some(5.0));
    assert_eq!(tree.get("borderColor").unwrap().as_str(), Some("#ff0000"));
}

#[test]
fn test_multiple_shapes_in_array() {
    let mut tree = json!({
        "shapes": [
            {
                "type": "rectangle",
                "borderTopWeight": 1.0,
                "borderBottomWeight": 1.0
            },
            {
                "type": "ellipse",
                "borderLeftWeight": 2.0,
                "borderRightWeight": 2.0
            },
            {
                "type": "line",
                "borderTopWeight": 0.5
            }
        ]
    });

    remove_border_weights(&mut tree).unwrap();

    // 所有数组元素中的所有边框权重应被删除
    assert!(tree["shapes"][0].get("borderTopWeight").is_none());
    assert!(tree["shapes"][0].get("borderBottomWeight").is_none());
    assert_eq!(tree["shapes"][0]["type"].as_str(), Some("rectangle"));

    assert!(tree["shapes"][1].get("borderLeftWeight").is_none());
    assert!(tree["shapes"][1].get("borderRightWeight").is_none());
    assert_eq!(tree["shapes"][1]["type"].as_str(), Some("ellipse"));

    assert!(tree["shapes"][2].get("borderTopWeight").is_none());
    assert_eq!(tree["shapes"][2]["type"].as_str(), Some("line"));
}

#[test]
fn test_zero_border_weights() {
    let mut tree = json!({
        "name": "Shape",
        "borderTopWeight": 0.0,
        "borderBottomWeight": 0.0,
        "borderLeftWeight": 0.0,
        "borderRightWeight": 0.0
    });

    remove_border_weights(&mut tree).unwrap();

    // 即使是零值边界权重也应该被删除
    assert!(tree.get("borderTopWeight").is_none());
    assert!(tree.get("borderBottomWeight").is_none());
    assert!(tree.get("borderLeftWeight").is_none());
    assert!(tree.get("borderRightWeight").is_none());
}

#[test]
fn test_remove_border_stroke_weights_independent() {
    let mut tree = json!({
        "name": "Rectangle",
        "borderStrokeWeightsIndependent": true,
        "borderTopWeight": 1.0,
        "borderBottomWeight": 2.0,
        "visible": true
    });

    remove_border_weights(&mut tree).unwrap();

    assert!(tree.get("borderStrokeWeightsIndependent").is_none());
    assert!(tree.get("borderTopWeight").is_none());
    assert!(tree.get("borderBottomWeight").is_none());
    assert_eq!(tree.get("name").unwrap().as_str(), Some("Rectangle"));
    assert_eq!(tree.get("visible").unwrap().as_bool(), Some(true));
}

#[test]
fn test_remove_border_stroke_weights_independent_nested() {
    let mut tree = json!({
        "children": [
            {
                "name": "Child1",
                "borderStrokeWeightsIndependent": true,
                "borderTopWeight": 1.0
            },
            {
                "name": "Child2",
                "borderStrokeWeightsIndependent": false
            }
        ]
    });

    remove_border_weights(&mut tree).unwrap();

    assert!(tree["children"][0].get("borderStrokeWeightsIndependent").is_none());
    assert!(tree["children"][0].get("borderTopWeight").is_none());
    assert_eq!(tree["children"][0]["name"].as_str(), Some("Child1"));

    assert!(tree["children"][1].get("borderStrokeWeightsIndependent").is_none());
    assert_eq!(tree["children"][1]["name"].as_str(), Some("Child2"));
}
