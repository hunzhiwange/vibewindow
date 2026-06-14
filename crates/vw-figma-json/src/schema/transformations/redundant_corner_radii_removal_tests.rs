use super::*;
use serde_json::json;

#[test]
fn test_removes_individual_radii_when_corner_radius_exists() {
    let mut tree = json!({
        "name": "Rectangle",
        "cornerRadius": 12.0,
        "rectangleTopLeftCornerRadius": 12.0,
        "rectangleTopRightCornerRadius": 12.0,
        "rectangleBottomLeftCornerRadius": 12.0,
        "rectangleBottomRightCornerRadius": 12.0
    });

    remove_redundant_corner_radii(&mut tree).unwrap();

    assert!(tree.get("cornerRadius").is_some());
    assert!(tree.get("rectangleTopLeftCornerRadius").is_none());
    assert!(tree.get("rectangleTopRightCornerRadius").is_none());
    assert!(tree.get("rectangleBottomLeftCornerRadius").is_none());
    assert!(tree.get("rectangleBottomRightCornerRadius").is_none());
    assert_eq!(tree.get("name").unwrap().as_str(), Some("Rectangle"));
}

#[test]
fn test_preserves_individual_radii_when_no_corner_radius() {
    let mut tree = json!({
        "name": "Rectangle",
        "rectangleTopLeftCornerRadius": 12.0,
        "rectangleTopRightCornerRadius": 8.0,
        "rectangleBottomLeftCornerRadius": 4.0,
        "rectangleBottomRightCornerRadius": 0.0
    });

    remove_redundant_corner_radii(&mut tree).unwrap();

    // 应保留各个半径
    assert!(tree.get("rectangleTopLeftCornerRadius").is_some());
    assert!(tree.get("rectangleTopRightCornerRadius").is_some());
    assert!(tree.get("rectangleBottomLeftCornerRadius").is_some());
    assert!(tree.get("rectangleBottomRightCornerRadius").is_some());
    assert_eq!(tree.get("rectangleTopLeftCornerRadius").unwrap().as_f64(), Some(12.0));
}

#[test]
fn test_removes_only_some_individual_radii() {
    let mut tree = json!({
        "name": "Rectangle",
        "cornerRadius": 16.0,
        "rectangleTopLeftCornerRadius": 16.0,
        "rectangleBottomRightCornerRadius": 16.0
    });

    remove_redundant_corner_radii(&mut tree).unwrap();

    assert!(tree.get("cornerRadius").is_some());
    assert!(tree.get("rectangleTopLeftCornerRadius").is_none());
    assert!(tree.get("rectangleBottomRightCornerRadius").is_none());
}

#[test]
fn test_handles_corner_radius_as_integer() {
    let mut tree = json!({
        "name": "Rectangle",
        "cornerRadius": 10,
        "rectangleTopLeftCornerRadius": 10.0,
        "rectangleTopRightCornerRadius": 10.0,
        "rectangleBottomLeftCornerRadius": 10.0,
        "rectangleBottomRightCornerRadius": 10.0
    });

    remove_redundant_corner_radii(&mut tree).unwrap();

    assert!(tree.get("cornerRadius").is_some());
    assert!(tree.get("rectangleTopLeftCornerRadius").is_none());
    assert!(tree.get("rectangleTopRightCornerRadius").is_none());
}

#[test]
fn test_handles_nested_objects() {
    let mut tree = json!({
        "name": "Parent",
        "children": [
            {
                "name": "Child1",
                "cornerRadius": 12.0,
                "rectangleTopLeftCornerRadius": 12.0,
                "rectangleBottomRightCornerRadius": 12.0
            },
            {
                "name": "Child2",
                "rectangleTopLeftCornerRadius": 8.0,
                "rectangleBottomRightCornerRadius": 8.0
            }
        ]
    });

    remove_redundant_corner_radii(&mut tree).unwrap();

    let children = tree.get("children").unwrap().as_array().unwrap();
    // Child1 有cornerRadius，因此应删除各个半径
    assert!(children[0].get("rectangleTopLeftCornerRadius").is_none());
    assert!(children[0].get("rectangleBottomRightCornerRadius").is_none());
    // Child2 没有cornerRadius，因此应保留各个半径
    assert!(children[1].get("rectangleTopLeftCornerRadius").is_some());
    assert!(children[1].get("rectangleBottomRightCornerRadius").is_some());
}

#[test]
fn test_handles_deeply_nested_structures() {
    let mut tree = json!({
        "name": "Root",
        "cornerRadius": 16.0,
        "rectangleTopLeftCornerRadius": 16.0,
        "children": [
            {
                "name": "Level1",
                "children": [
                    {
                        "name": "Level2",
                        "cornerRadius": 12.0,
                        "rectangleTopLeftCornerRadius": 12.0,
                        "rectangleTopRightCornerRadius": 12.0,
                        "rectangleBottomLeftCornerRadius": 12.0,
                        "rectangleBottomRightCornerRadius": 12.0
                    }
                ]
            }
        ]
    });

    remove_redundant_corner_radii(&mut tree).unwrap();

    assert!(tree.get("rectangleTopLeftCornerRadius").is_none());
    let level1 = &tree.get("children").unwrap().as_array().unwrap()[0];
    let level2 = &level1.get("children").unwrap().as_array().unwrap()[0];
    assert!(level2.get("rectangleTopLeftCornerRadius").is_none());
    assert!(level2.get("rectangleTopRightCornerRadius").is_none());
    assert!(level2.get("rectangleBottomLeftCornerRadius").is_none());
    assert!(level2.get("rectangleBottomRightCornerRadius").is_none());
    assert!(level2.get("cornerRadius").is_some());
}

#[test]
fn test_handles_missing_all_radius_properties() {
    let mut tree = json!({
        "name": "Rectangle",
        "type": "ROUNDED_RECTANGLE",
        "size": {"x": 100.0, "y": 100.0}
    });

    remove_redundant_corner_radii(&mut tree).unwrap();

    assert_eq!(tree.get("name").unwrap().as_str(), Some("Rectangle"));
    assert!(tree.get("type").is_some());
}

#[test]
fn test_handles_empty_object() {
    let mut tree = json!({});

    remove_redundant_corner_radii(&mut tree).unwrap();

    assert_eq!(tree.as_object().unwrap().len(), 0);
}

#[test]
fn test_handles_primitive_value() {
    let mut tree = json!("rectangle");

    remove_redundant_corner_radii(&mut tree).unwrap();

    assert_eq!(tree.as_str(), Some("rectangle"));
}

#[test]
fn test_preserves_other_fields() {
    let mut tree = json!({
        "name": "Button",
        "type": "INSTANCE",
        "cornerRadius": 12.0,
        "cornerSmoothing": 0.6000000238418579,
        "rectangleTopLeftCornerRadius": 12.0,
        "rectangleTopRightCornerRadius": 12.0,
        "rectangleBottomLeftCornerRadius": 12.0,
        "rectangleBottomRightCornerRadius": 12.0,
        "fillPaints": [{"color": "#343439", "type": "SOLID"}],
        "size": {"x": 327.0, "y": 48.0}
    });

    remove_redundant_corner_radii(&mut tree).unwrap();

    assert!(tree.get("rectangleTopLeftCornerRadius").is_none());
    assert!(tree.get("rectangleTopRightCornerRadius").is_none());
    assert!(tree.get("rectangleBottomLeftCornerRadius").is_none());
    assert!(tree.get("rectangleBottomRightCornerRadius").is_none());
    assert_eq!(tree.get("name").unwrap().as_str(), Some("Button"));
    assert_eq!(tree.get("type").unwrap().as_str(), Some("INSTANCE"));
    assert!(tree.get("cornerRadius").is_some());
    assert!(tree.get("cornerSmoothing").is_some());
    assert!(tree.get("fillPaints").is_some());
    assert!(tree.get("size").is_some());
}
