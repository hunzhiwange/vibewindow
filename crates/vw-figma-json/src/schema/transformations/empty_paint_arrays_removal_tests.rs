use super::*;
use serde_json::json;

#[test]
fn test_removes_empty_fill_paints() {
    let mut tree = json!({
        "name": "Rectangle",
        "fillPaints": [],
        "size": {"x": 100.0, "y": 100.0}
    });

    remove_empty_paint_arrays(&mut tree).unwrap();

    assert!(tree.get("fillPaints").is_none());
    assert_eq!(tree.get("name").unwrap().as_str(), Some("Rectangle"));
    assert!(tree.get("size").is_some());
}

#[test]
fn test_removes_empty_stroke_paints() {
    let mut tree = json!({
        "name": "Rectangle",
        "strokePaints": [],
        "size": {"x": 100.0, "y": 100.0}
    });

    remove_empty_paint_arrays(&mut tree).unwrap();

    assert!(tree.get("strokePaints").is_none());
    assert_eq!(tree.get("name").unwrap().as_str(), Some("Rectangle"));
    assert!(tree.get("size").is_some());
}

#[test]
fn test_removes_both_empty_paint_arrays() {
    let mut tree = json!({
        "name": "Rectangle",
        "fillPaints": [],
        "strokePaints": []
    });

    remove_empty_paint_arrays(&mut tree).unwrap();

    assert!(tree.get("fillPaints").is_none());
    assert!(tree.get("strokePaints").is_none());
    assert_eq!(tree.get("name").unwrap().as_str(), Some("Rectangle"));
}

#[test]
fn test_preserves_non_empty_fill_paints() {
    let mut tree = json!({
        "name": "Rectangle",
        "fillPaints": [
            {
                "color": "#ffffff",
                "type": "SOLID"
            }
        ]
    });

    remove_empty_paint_arrays(&mut tree).unwrap();

    assert!(tree.get("fillPaints").is_some());
    let fills = tree.get("fillPaints").unwrap().as_array().unwrap();
    assert_eq!(fills.len(), 1);
    assert_eq!(fills[0].get("color").unwrap().as_str(), Some("#ffffff"));
}

#[test]
fn test_preserves_non_empty_stroke_paints() {
    let mut tree = json!({
        "name": "Rectangle",
        "strokePaints": [
            {
                "color": "#000000",
                "type": "SOLID"
            }
        ]
    });

    remove_empty_paint_arrays(&mut tree).unwrap();

    assert!(tree.get("strokePaints").is_some());
    let strokes = tree.get("strokePaints").unwrap().as_array().unwrap();
    assert_eq!(strokes.len(), 1);
    assert_eq!(strokes[0].get("color").unwrap().as_str(), Some("#000000"));
}

#[test]
fn test_handles_missing_paint_arrays() {
    let mut tree = json!({
        "name": "Rectangle",
        "size": {"x": 100.0, "y": 100.0}
    });

    remove_empty_paint_arrays(&mut tree).unwrap();

    assert!(tree.get("fillPaints").is_none());
    assert!(tree.get("strokePaints").is_none());
    assert_eq!(tree.get("name").unwrap().as_str(), Some("Rectangle"));
}

#[test]
fn test_handles_nested_objects() {
    let mut tree = json!({
        "name": "Parent",
        "children": [
            {
                "name": "Child1",
                "fillPaints": []
            },
            {
                "name": "Child2",
                "strokePaints": []
            }
        ]
    });

    remove_empty_paint_arrays(&mut tree).unwrap();

    let children = tree.get("children").unwrap().as_array().unwrap();
    assert!(children[0].get("fillPaints").is_none());
    assert!(children[1].get("strokePaints").is_none());
    assert_eq!(children[0].get("name").unwrap().as_str(), Some("Child1"));
    assert_eq!(children[1].get("name").unwrap().as_str(), Some("Child2"));
}

#[test]
fn test_handles_deeply_nested_structures() {
    let mut tree = json!({
        "name": "Root",
        "fillPaints": [],
        "children": [
            {
                "name": "Level1",
                "strokePaints": [],
                "children": [
                    {
                        "name": "Level2",
                        "fillPaints": [],
                        "strokePaints": []
                    }
                ]
            }
        ]
    });

    remove_empty_paint_arrays(&mut tree).unwrap();

    assert!(tree.get("fillPaints").is_none());
    let level1 = &tree.get("children").unwrap().as_array().unwrap()[0];
    assert!(level1.get("strokePaints").is_none());
    let level2 = &level1.get("children").unwrap().as_array().unwrap()[0];
    assert!(level2.get("fillPaints").is_none());
    assert!(level2.get("strokePaints").is_none());
    assert_eq!(level2.get("name").unwrap().as_str(), Some("Level2"));
}

#[test]
fn test_handles_empty_object() {
    let mut tree = json!({});

    remove_empty_paint_arrays(&mut tree).unwrap();

    assert_eq!(tree.as_object().unwrap().len(), 0);
}

#[test]
fn test_preserves_other_fields() {
    let mut tree = json!({
        "name": "icon/ai",
        "type": "INSTANCE",
        "fillPaints": [],
        "strokePaints": [],
        "size": {"x": 20.0, "y": 20.0},
        "transform": {"x": 0.0, "y": 9.0}
    });

    remove_empty_paint_arrays(&mut tree).unwrap();

    assert!(tree.get("fillPaints").is_none());
    assert!(tree.get("strokePaints").is_none());
    assert_eq!(tree.get("name").unwrap().as_str(), Some("icon/ai"));
    assert_eq!(tree.get("type").unwrap().as_str(), Some("INSTANCE"));
    assert!(tree.get("size").is_some());
    assert!(tree.get("transform").is_some());
}

#[test]
fn test_mixed_empty_and_non_empty() {
    let mut tree = json!({
        "children": [
            {
                "name": "EmptyFills",
                "fillPaints": [],
                "strokePaints": [{"color": "#000", "type": "SOLID"}]
            },
            {
                "name": "EmptyStrokes",
                "fillPaints": [{"color": "#fff", "type": "SOLID"}],
                "strokePaints": []
            },
            {
                "name": "BothEmpty",
                "fillPaints": [],
                "strokePaints": []
            },
            {
                "name": "NeitherEmpty",
                "fillPaints": [{"color": "#f00", "type": "SOLID"}],
                "strokePaints": [{"color": "#0f0", "type": "SOLID"}]
            }
        ]
    });

    remove_empty_paint_arrays(&mut tree).unwrap();

    let children = tree.get("children").unwrap().as_array().unwrap();

    // 子级 0：删除空填充，保留笔划
    assert!(children[0].get("fillPaints").is_none());
    assert!(children[0].get("strokePaints").is_some());

    // 子 1：保留填充，删除空笔划
    assert!(children[1].get("fillPaints").is_some());
    assert!(children[1].get("strokePaints").is_none());

    // 子 2：两个空数组均已删除
    assert!(children[2].get("fillPaints").is_none());
    assert!(children[2].get("strokePaints").is_none());

    // 子 3：均保留
    assert!(children[3].get("fillPaints").is_some());
    assert!(children[3].get("strokePaints").is_some());
}

#[test]
fn test_preserves_non_array_paint_fields() {
    let mut tree = json!({
        "fillPaints": "none",
        "strokePaints": null,
        "children": [true, 7]
    });

    remove_empty_paint_arrays(&mut tree).unwrap();

    assert_eq!(tree["fillPaints"].as_str(), Some("none"));
    assert!(tree["strokePaints"].is_null());
    assert_eq!(tree["children"][0].as_bool(), Some(true));
    assert_eq!(tree["children"][1].as_i64(), Some(7));
}

#[test]
fn test_root_array_and_primitive_values() {
    let mut tree = json!([
        {"fillPaints": []},
        {"strokePaints": []},
        "paint"
    ]);

    remove_empty_paint_arrays(&mut tree).unwrap();

    assert!(tree[0].get("fillPaints").is_none());
    assert!(tree[1].get("strokePaints").is_none());
    assert_eq!(tree[2].as_str(), Some("paint"));

    let mut primitive = json!(false);
    remove_empty_paint_arrays(&mut primitive).unwrap();
    assert_eq!(primitive.as_bool(), Some(false));
}
