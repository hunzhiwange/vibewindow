use super::*;
use serde_json::json;

#[test]
fn test_remove_edit_info_simple() {
    let mut tree = json!({
        "name": "Rectangle",
        "editInfo": {
            "createdAt": 1761413476,
            "lastEditedAt": 1761413532,
            "userId": "1106160570506540696"
        },
        "visible": true
    });

    remove_edit_info_fields(&mut tree).unwrap();

    assert!(tree.get("editInfo").is_none());
    assert_eq!(tree.get("name").unwrap().as_str(), Some("Rectangle"));
    assert_eq!(tree.get("visible").unwrap().as_bool(), Some(true));
}

#[test]
fn test_remove_edit_info_nested() {
    let mut tree = json!({
        "name": "Root",
        "editInfo": {
            "createdAt": 0,
            "lastEditedAt": 1761414263,
            "userId": "1106160570506540696"
        },
        "children": [
            {
                "name": "Child1",
                "editInfo": {
                    "createdAt": 1761413389,
                    "lastEditedAt": 1761414263,
                    "userId": "1106160570506540696"
                }
            },
            {
                "name": "Child2",
                "editInfo": {
                    "createdAt": 1761414252,
                    "lastEditedAt": 1761414252,
                    "userId": "1106160570506540696"
                }
            }
        ]
    });

    remove_edit_info_fields(&mut tree).unwrap();

    // 根 editInfo 应该被删除
    assert!(tree.get("editInfo").is_none());
    assert_eq!(tree.get("name").unwrap().as_str(), Some("Root"));

    // 子项 editInfo 应该被删除
    assert!(tree["children"][0].get("editInfo").is_none());
    assert_eq!(tree["children"][0].get("name").unwrap().as_str(), Some("Child1"));

    assert!(tree["children"][1].get("editInfo").is_none());
    assert_eq!(tree["children"][1].get("name").unwrap().as_str(), Some("Child2"));
}

#[test]
fn test_remove_edit_info_deeply_nested() {
    let mut tree = json!({
        "document": {
            "editInfo": {
                "createdAt": 0,
                "lastEditedAt": 1000,
                "userId": "user1"
            },
            "children": [
                {
                    "editInfo": {
                        "createdAt": 500,
                        "lastEditedAt": 800,
                        "userId": "user2"
                    },
                    "children": [
                        {
                            "editInfo": {
                                "createdAt": 600,
                                "lastEditedAt": 700,
                                "userId": "user3"
                            },
                            "name": "DeepChild"
                        }
                    ]
                }
            ]
        }
    });

    remove_edit_info_fields(&mut tree).unwrap();

    // 所有级别的 editInfo 都应该被删除
    assert!(tree["document"].get("editInfo").is_none());
    assert!(tree["document"]["children"][0].get("editInfo").is_none());
    assert!(tree["document"]["children"][0]["children"][0].get("editInfo").is_none());

    // 其他字段应保留
    assert_eq!(
        tree["document"]["children"][0]["children"][0].get("name").unwrap().as_str(),
        Some("DeepChild")
    );
}

#[test]
fn test_remove_edit_info_missing() {
    let mut tree = json!({
        "name": "Rectangle",
        "visible": true,
        "x": 10,
        "y": 20
    });

    remove_edit_info_fields(&mut tree).unwrap();

    // 没有 editInfo 的树应该保持不变
    assert!(tree.get("editInfo").is_none());
    assert_eq!(tree.get("name").unwrap().as_str(), Some("Rectangle"));
    assert_eq!(tree.get("visible").unwrap().as_bool(), Some(true));
    assert_eq!(tree.get("x").unwrap().as_i64(), Some(10));
    assert_eq!(tree.get("y").unwrap().as_i64(), Some(20));
}

#[test]
fn test_remove_edit_info_preserves_other_fields() {
    let mut tree = json!({
        "name": "Frame",
        "editInfo": {
            "createdAt": 1761413389,
            "lastEditedAt": 1761414263,
            "userId": "1106160570506540696"
        },
        "type": "FRAME",
        "opacity": 1.0,
        "visible": true,
        "x": 100,
        "y": 200
    });

    remove_edit_info_fields(&mut tree).unwrap();

    // 仅应删除 editInfo
    assert!(tree.get("editInfo").is_none());

    // 保留所有其他字段
    assert_eq!(tree.get("name").unwrap().as_str(), Some("Frame"));
    assert_eq!(tree.get("type").unwrap().as_str(), Some("FRAME"));
    assert_eq!(tree.get("opacity").unwrap().as_f64(), Some(1.0));
    assert_eq!(tree.get("visible").unwrap().as_bool(), Some(true));
    assert_eq!(tree.get("x").unwrap().as_i64(), Some(100));
    assert_eq!(tree.get("y").unwrap().as_i64(), Some(200));
}

#[test]
fn test_remove_edit_info_in_arrays() {
    let mut tree = json!({
        "items": [
            {
                "editInfo": {
                    "createdAt": 1000,
                    "lastEditedAt": 2000,
                    "userId": "user1"
                },
                "name": "Item1"
            },
            {
                "editInfo": {
                    "createdAt": 3000,
                    "lastEditedAt": 4000,
                    "userId": "user2"
                },
                "name": "Item2"
            }
        ]
    });

    remove_edit_info_fields(&mut tree).unwrap();

    // 数组中的所有 editInfo 应被删除
    assert!(tree["items"][0].get("editInfo").is_none());
    assert_eq!(tree["items"][0].get("name").unwrap().as_str(), Some("Item1"));

    assert!(tree["items"][1].get("editInfo").is_none());
    assert_eq!(tree["items"][1].get("name").unwrap().as_str(), Some("Item2"));
}

#[test]
fn test_remove_edit_info_mixed_objects() {
    let mut tree = json!({
        "name": "Root",
        "editInfo": {
            "createdAt": 0,
            "lastEditedAt": 1000,
            "userId": "root_user"
        },
        "properties": {
            "width": 100,
            "height": 200
        },
        "children": [
            {
                "editInfo": {
                    "createdAt": 500,
                    "lastEditedAt": 800,
                    "userId": "child_user"
                },
                "name": "Child"
            }
        ]
    });

    remove_edit_info_fields(&mut tree).unwrap();

    // 根 editInfo 已删除
    assert!(tree.get("editInfo").is_none());

    // 属性对象未更改(无 editInfo)
    assert_eq!(tree["properties"]["width"].as_i64(), Some(100));
    assert_eq!(tree["properties"]["height"].as_i64(), Some(200));

    // 子 editInfo 已删除
    assert!(tree["children"][0].get("editInfo").is_none());
    assert_eq!(tree["children"][0].get("name").unwrap().as_str(), Some("Child"));
}

#[test]
fn test_remove_edit_info_empty_object() {
    let mut tree = json!({});

    remove_edit_info_fields(&mut tree).unwrap();

    // 空对象应保持为空
    assert_eq!(tree.as_object().unwrap().len(), 0);
}

#[test]
fn test_remove_edit_info_primitives() {
    let mut tree = json!(42);

    remove_edit_info_fields(&mut tree).unwrap();

    // 原始值应保持不变
    assert_eq!(tree.as_i64(), Some(42));
}
