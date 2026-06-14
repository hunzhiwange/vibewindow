use super::*;
use serde_json::json;

#[test]
fn test_remove_frame_mask_disabled() {
    let mut tree = json!({
        "name": "Frame",
        "frameMaskDisabled": false,
        "visible": true
    });

    remove_frame_properties(&mut tree).unwrap();

    assert!(tree.get("frameMaskDisabled").is_none());
    assert_eq!(tree.get("name").unwrap().as_str(), Some("Frame"));
    assert_eq!(tree.get("visible").unwrap().as_bool(), Some(true));
}

#[test]
fn test_remove_target_aspect_ratio() {
    let mut tree = json!({
        "name": "Image",
        "targetAspectRatio": {
            "value": {
                "x": 300.0,
                "y": 300.0
            }
        },
        "opacity": 1.0
    });

    remove_frame_properties(&mut tree).unwrap();

    assert!(tree.get("targetAspectRatio").is_none());
    assert_eq!(tree.get("name").unwrap().as_str(), Some("Image"));
    assert_eq!(tree.get("opacity").unwrap().as_f64(), Some(1.0));
}

#[test]
fn test_remove_both_frame_properties() {
    let mut tree = json!({
        "name": "Frame",
        "frameMaskDisabled": false,
        "targetAspectRatio": {
            "value": {
                "x": 16.0,
                "y": 9.0
            }
        },
        "type": "FRAME"
    });

    remove_frame_properties(&mut tree).unwrap();

    assert!(tree.get("frameMaskDisabled").is_none());
    assert!(tree.get("targetAspectRatio").is_none());
    assert_eq!(tree.get("name").unwrap().as_str(), Some("Frame"));
    assert_eq!(tree.get("type").unwrap().as_str(), Some("FRAME"));
}

#[test]
fn test_nested_frame_properties() {
    let mut tree = json!({
        "name": "Root",
        "children": [
            {
                "name": "Child1",
                "frameMaskDisabled": false
            },
            {
                "name": "Child2",
                "children": [
                    {
                        "name": "DeepChild",
                        "targetAspectRatio": {
                            "value": {
                                "x": 100.0,
                                "y": 100.0
                            }
                        }
                    }
                ]
            }
        ]
    });

    remove_frame_properties(&mut tree).unwrap();

    // 检查第一个嵌套元素
    assert!(tree["children"][0].get("frameMaskDisabled").is_none());
    assert_eq!(tree["children"][0].get("name").unwrap().as_str(), Some("Child1"));

    // 检查深层嵌套元素
    let deep_child = &tree["children"][1]["children"][0];
    assert!(deep_child.get("targetAspectRatio").is_none());
    assert_eq!(deep_child.get("name").unwrap().as_str(), Some("DeepChild"));
}

#[test]
fn test_no_frame_properties() {
    let mut tree = json!({
        "name": "Rectangle",
        "width": 100,
        "height": 200,
        "visible": true
    });

    remove_frame_properties(&mut tree).unwrap();

    // 没有框架属性的树应该保持不变
    assert_eq!(tree.get("name").unwrap().as_str(), Some("Rectangle"));
    assert_eq!(tree.get("width").unwrap().as_i64(), Some(100));
    assert_eq!(tree.get("height").unwrap().as_i64(), Some(200));
    assert_eq!(tree.get("visible").unwrap().as_bool(), Some(true));
}

#[test]
fn test_preserves_other_frame_fields() {
    let mut tree = json!({
        "name": "Ecran 1",
        "frameMaskDisabled": false,
        "type": "FRAME",
        "size": {
            "x": 1170.0,
            "y": 2532.0
        },
        "fillPaints": [
            {
                "color": "#ffffff",
                "opacity": 1.0,
                "visible": true
            }
        ]
    });

    remove_frame_properties(&mut tree).unwrap();

    // 框架属性已删除
    assert!(tree.get("frameMaskDisabled").is_none());

    // 保留其他字段
    assert_eq!(tree.get("name").unwrap().as_str(), Some("Ecran 1"));
    assert_eq!(tree.get("type").unwrap().as_str(), Some("FRAME"));
    assert!(tree.get("size").is_some());
    assert!(tree.get("fillPaints").is_some());
}

#[test]
fn test_multiple_frames() {
    let mut tree = json!({
        "items": [
            {
                "name": "Frame1",
                "frameMaskDisabled": false
            },
            {
                "name": "Frame2",
                "frameMaskDisabled": true
            },
            {
                "name": "Image1",
                "targetAspectRatio": {
                    "value": {
                        "x": 4.0,
                        "y": 3.0
                    }
                }
            }
        ]
    });

    remove_frame_properties(&mut tree).unwrap();

    // 数组中的所有框架属性应被删除
    assert!(tree["items"][0].get("frameMaskDisabled").is_none());
    assert_eq!(tree["items"][0].get("name").unwrap().as_str(), Some("Frame1"));

    assert!(tree["items"][1].get("frameMaskDisabled").is_none());
    assert_eq!(tree["items"][1].get("name").unwrap().as_str(), Some("Frame2"));

    assert!(tree["items"][2].get("targetAspectRatio").is_none());
    assert_eq!(tree["items"][2].get("name").unwrap().as_str(), Some("Image1"));
}

#[test]
fn test_frame_mask_disabled_different_values() {
    let mut tree = json!({
        "frame1": {
            "frameMaskDisabled": false,
            "name": "Frame1"
        },
        "frame2": {
            "frameMaskDisabled": true,
            "name": "Frame2"
        }
    });

    remove_frame_properties(&mut tree).unwrap();

    // true 和 false 值都应该被删除
    assert!(tree["frame1"].get("frameMaskDisabled").is_none());
    assert_eq!(tree["frame1"].get("name").unwrap().as_str(), Some("Frame1"));

    assert!(tree["frame2"].get("frameMaskDisabled").is_none());
    assert_eq!(tree["frame2"].get("name").unwrap().as_str(), Some("Frame2"));
}

#[test]
fn test_empty_object() {
    let mut tree = json!({});

    remove_frame_properties(&mut tree).unwrap();

    // 空对象应保持为空
    assert_eq!(tree.as_object().unwrap().len(), 0);
}

#[test]
fn test_frame_properties_primitive_value() {
    let mut tree = json!(null);

    remove_frame_properties(&mut tree).unwrap();

    assert!(tree.is_null());
}

#[test]
fn test_frame_properties_mixed_array_primitives() {
    let mut tree = json!([
        {"frameMaskDisabled": true, "name": "Frame"},
        "unchanged",
        12
    ]);

    remove_frame_properties(&mut tree).unwrap();

    assert!(tree[0].get("frameMaskDisabled").is_none());
    assert_eq!(tree[0]["name"].as_str(), Some("Frame"));
    assert_eq!(tree[1].as_str(), Some("unchanged"));
    assert_eq!(tree[2].as_i64(), Some(12));
}
