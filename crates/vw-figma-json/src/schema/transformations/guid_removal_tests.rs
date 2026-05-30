use super::*;
use serde_json::json;

#[test]
fn test_remove_guid_simple() {
    let mut tree = json!({
        "name": "Rectangle",
        "guid": {
            "localID": 3,
            "sessionID": 1
        },
        "visible": true
    });

    remove_guid_fields(&mut tree).unwrap();

    assert!(tree.get("guid").is_none());
    assert_eq!(tree.get("name").unwrap().as_str(), Some("Rectangle"));
    assert_eq!(tree.get("visible").unwrap().as_bool(), Some(true));
}

#[test]
fn test_remove_guid_nested() {
    let mut tree = json!({
        "name": "Root",
        "guid": {
            "localID": 0,
            "sessionID": 0
        },
        "children": [
            {
                "name": "Child1",
                "guid": {
                    "localID": 1,
                    "sessionID": 0
                }
            },
            {
                "name": "Child2",
                "guid": {
                    "localID": 2,
                    "sessionID": 1
                }
            }
        ]
    });

    remove_guid_fields(&mut tree).unwrap();

    // 应该删除根 guid
    assert!(tree.get("guid").is_none());
    assert_eq!(tree.get("name").unwrap().as_str(), Some("Root"));

    // 儿童指南应拆除
    assert!(tree["children"][0].get("guid").is_none());
    assert_eq!(tree["children"][0].get("name").unwrap().as_str(), Some("Child1"));

    assert!(tree["children"][1].get("guid").is_none());
    assert_eq!(tree["children"][1].get("name").unwrap().as_str(), Some("Child2"));
}

#[test]
fn test_remove_guid_deeply_nested() {
    let mut tree = json!({
        "document": {
            "guid": {
                "localID": 0,
                "sessionID": 0
            },
            "children": [
                {
                    "guid": {
                        "localID": 1,
                        "sessionID": 0
                    },
                    "children": [
                        {
                            "guid": {
                                "localID": 2,
                                "sessionID": 0
                            },
                            "name": "DeepChild"
                        }
                    ]
                }
            ]
        }
    });

    remove_guid_fields(&mut tree).unwrap();

    // 所有级别的所有指南都应删除
    assert!(tree["document"].get("guid").is_none());
    assert!(tree["document"]["children"][0].get("guid").is_none());
    assert!(tree["document"]["children"][0]["children"][0].get("guid").is_none());

    // 其他字段应保留
    assert_eq!(
        tree["document"]["children"][0]["children"][0].get("name").unwrap().as_str(),
        Some("DeepChild")
    );
}

#[test]
fn test_remove_guid_missing() {
    let mut tree = json!({
        "name": "Rectangle",
        "visible": true,
        "x": 10,
        "y": 20
    });

    remove_guid_fields(&mut tree).unwrap();

    // 没有 guid 的树应该保持不变
    assert!(tree.get("guid").is_none());
    assert_eq!(tree.get("name").unwrap().as_str(), Some("Rectangle"));
    assert_eq!(tree.get("visible").unwrap().as_bool(), Some(true));
    assert_eq!(tree.get("x").unwrap().as_i64(), Some(10));
    assert_eq!(tree.get("y").unwrap().as_i64(), Some(20));
}

#[test]
fn test_remove_guid_preserves_other_fields() {
    let mut tree = json!({
        "name": "Frame",
        "guid": {
            "localID": 5,
            "sessionID": 2
        },
        "type": "FRAME",
        "opacity": 1.0,
        "visible": true,
        "x": 100,
        "y": 200
    });

    remove_guid_fields(&mut tree).unwrap();

    // 仅应删除 guid
    assert!(tree.get("guid").is_none());

    // 保留所有其他字段
    assert_eq!(tree.get("name").unwrap().as_str(), Some("Frame"));
    assert_eq!(tree.get("type").unwrap().as_str(), Some("FRAME"));
    assert_eq!(tree.get("opacity").unwrap().as_f64(), Some(1.0));
    assert_eq!(tree.get("visible").unwrap().as_bool(), Some(true));
    assert_eq!(tree.get("x").unwrap().as_i64(), Some(100));
    assert_eq!(tree.get("y").unwrap().as_i64(), Some(200));
}

#[test]
fn test_remove_guid_in_arrays() {
    let mut tree = json!({
        "items": [
            {
                "guid": {
                    "localID": 1,
                    "sessionID": 0
                },
                "name": "Item1"
            },
            {
                "guid": {
                    "localID": 2,
                    "sessionID": 0
                },
                "name": "Item2"
            }
        ]
    });

    remove_guid_fields(&mut tree).unwrap();

    // 数组中的所有 guid 都应该被删除
    assert!(tree["items"][0].get("guid").is_none());
    assert_eq!(tree["items"][0].get("name").unwrap().as_str(), Some("Item1"));

    assert!(tree["items"][1].get("guid").is_none());
    assert_eq!(tree["items"][1].get("name").unwrap().as_str(), Some("Item2"));
}

#[test]
fn test_remove_guid_mixed_objects() {
    let mut tree = json!({
        "name": "Root",
        "guid": {
            "localID": 0,
            "sessionID": 0
        },
        "properties": {
            "width": 100,
            "height": 200
        },
        "children": [
            {
                "guid": {
                    "localID": 1,
                    "sessionID": 0
                },
                "name": "Child"
            }
        ]
    });

    remove_guid_fields(&mut tree).unwrap();

    // 根 guid已删除
    assert!(tree.get("guid").is_none());

    // 属性对象未更改(无 guid)
    assert_eq!(tree["properties"]["width"].as_i64(), Some(100));
    assert_eq!(tree["properties"]["height"].as_i64(), Some(200));

    // 儿童指南已删除
    assert!(tree["children"][0].get("guid").is_none());
    assert_eq!(tree["children"][0].get("name").unwrap().as_str(), Some("Child"));
}

#[test]
fn test_remove_guid_empty_object() {
    let mut tree = json!({});

    remove_guid_fields(&mut tree).unwrap();

    // 空对象应保持为空
    assert_eq!(tree.as_object().unwrap().len(), 0);
}

#[test]
fn test_remove_guid_primitives() {
    let mut tree = json!("string value");

    remove_guid_fields(&mut tree).unwrap();

    // 原始值应保持不变
    assert_eq!(tree.as_str(), Some("string value"));
}
