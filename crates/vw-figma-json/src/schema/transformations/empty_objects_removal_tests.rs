use super::*;
use serde_json::json;

#[test]
fn test_remove_empty_object_field() {
    let mut tree = json!({
        "name": "Shape",
        "data": {},
        "opacity": 1.0
    });

    remove_empty_objects(&mut tree).unwrap();

    assert!(tree.get("data").is_none());
    assert_eq!(tree.get("name").unwrap().as_str(), Some("Shape"));
    assert_eq!(tree.get("opacity").unwrap().as_f64(), Some(1.0));
}

#[test]
fn test_remove_empty_objects_from_array() {
    let mut tree = json!({
        "items": [1, {}, 2, {}, 3]
    });

    remove_empty_objects(&mut tree).unwrap();

    let items = tree.get("items").unwrap().as_array().unwrap();
    assert_eq!(items.len(), 3);
    assert_eq!(items[0].as_i64(), Some(1));
    assert_eq!(items[1].as_i64(), Some(2));
    assert_eq!(items[2].as_i64(), Some(3));
}

#[test]
fn test_preserve_non_empty_objects() {
    let mut tree = json!({
        "name": "Shape",
        "data": {"key": "value"},
        "empty": {}
    });

    remove_empty_objects(&mut tree).unwrap();

    assert!(tree.get("empty").is_none());
    assert!(tree.get("data").is_some());
    assert_eq!(
        tree.get("data").unwrap().as_object().unwrap().get("key").unwrap().as_str(),
        Some("value")
    );
}

#[test]
fn test_nested_empty_objects() {
    let mut tree = json!({
        "children": [
            {
                "name": "Child1",
                "metadata": {}
            },
            {
                "name": "Child2",
                "data": {"value": 42}
            }
        ]
    });

    remove_empty_objects(&mut tree).unwrap();

    assert!(tree["children"][0].get("metadata").is_none());
    assert_eq!(tree["children"][0]["name"].as_str(), Some("Child1"));

    assert!(tree["children"][1].get("data").is_some());
    assert_eq!(tree["children"][1]["name"].as_str(), Some("Child2"));
}

#[test]
fn test_array_of_empty_objects() {
    let mut tree = json!({
        "items": [{}, {}, {}]
    });

    remove_empty_objects(&mut tree).unwrap();

    let items = tree.get("items").unwrap().as_array().unwrap();
    assert_eq!(items.len(), 0);
}

#[test]
fn test_mixed_array() {
    let mut tree = json!({
        "items": [
            {},
            {"name": "A"},
            {},
            {"name": "B"},
            {}
        ]
    });

    remove_empty_objects(&mut tree).unwrap();

    let items = tree.get("items").unwrap().as_array().unwrap();
    assert_eq!(items.len(), 2);
    assert_eq!(items[0]["name"].as_str(), Some("A"));
    assert_eq!(items[1]["name"].as_str(), Some("B"));
}

#[test]
fn test_deeply_nested() {
    let mut tree = json!({
        "level1": {
            "level2": {
                "level3": {
                    "empty": {},
                    "data": "value"
                },
                "alsoEmpty": {}
            }
        }
    });

    remove_empty_objects(&mut tree).unwrap();

    assert!(tree["level1"]["level2"]["level3"].get("empty").is_none());
    assert_eq!(tree["level1"]["level2"]["level3"]["data"].as_str(), Some("value"));
    assert!(tree["level1"]["level2"].get("alsoEmpty").is_none());
}

#[test]
fn test_no_empty_objects() {
    let mut tree = json!({
        "name": "Rectangle",
        "width": 100,
        "children": [
            {"name": "Child1"},
            {"name": "Child2"}
        ]
    });

    let original = tree.clone();
    remove_empty_objects(&mut tree).unwrap();

    // 树应该保持不变
    assert_eq!(tree, original);
}

#[test]
fn test_derived_symbol_data_case() {
    // 来自 Roles-members.json 的真实示例
    let mut tree = json!({
        "derivedSymbolData": [
            {},
            {},
            {},
            {
                "size": {
                    "x": 42.0,
                    "y": 22.0
                }
            },
            {}
        ]
    });

    remove_empty_objects(&mut tree).unwrap();

    let data = tree.get("derivedSymbolData").unwrap().as_array().unwrap();
    assert_eq!(data.len(), 1);
    assert!(data[0].get("size").is_some());
    assert_eq!(data[0]["size"]["x"].as_f64(), Some(42.0));
}

#[test]
fn test_symbol_overrides_case() {
    // 另一个现实世界的模式
    let mut tree = json!({
        "symbolOverrides": [
            {
                "opacity": 0.0
            },
            {
                "textData": {
                    "characters": "Hello"
                }
            },
            {}
        ]
    });

    remove_empty_objects(&mut tree).unwrap();

    let overrides = tree.get("symbolOverrides").unwrap().as_array().unwrap();
    assert_eq!(overrides.len(), 2);
    assert!(overrides[0].get("opacity").is_some());
    assert!(overrides[1].get("textData").is_some());
}

#[test]
fn test_empty_array_preserved() {
    let mut tree = json!({
        "name": "Test",
        "items": []
    });

    remove_empty_objects(&mut tree).unwrap();

    // 应保留空数组，仅删除空对象
    assert!(tree.get("items").is_some());
    let items = tree.get("items").unwrap().as_array().unwrap();
    assert_eq!(items.len(), 0);
}

#[test]
fn test_object_becomes_empty_after_removal() {
    let mut tree = json!({
        "parent": {
            "child1": {},
            "child2": {}
        }
    });

    remove_empty_objects(&mut tree).unwrap();

    // 删除空的子级后，父级变为空并且也被删除
    // 这是正确的行为 - 递归删除所有空对象
    assert!(tree.get("parent").is_none());
}
