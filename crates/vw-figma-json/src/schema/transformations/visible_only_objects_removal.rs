use crate::error::Result;
use serde_json::Value as JsonValue;

/// 从 JSON 树中删除仅包含visible 属性的对象
///
/// 递归遍历JSON树并移除只有一个key的对象
/// 名为 "visible"。这些对象通常出现在Figma的symbolOverrides数组中
/// 并用于隐藏/显示元素而不提供其他有意义的数据。
///
/// 仅具有 `visible` 的对象将从以下位置删除：
/// - 数组(仅可见的对象元素被过滤掉)
/// - 对象值(如果值只有 `visible`，则删除键值对)
///
/// # 参数
/// * `tree` - 要修改的 JSON 树(通常是文档根)
///
/// # 返回值
/// * `Ok(())` - 成功删除所有仅可见对象
///
/// # 示例
/// ```no_run
/// use fig2json::schema::remove_visible_only_objects;
/// use serde_json::json;
///
/// let mut tree = json!({
///     "symbolOverrides": [
///         {"visible": false},
///         {"textData": {"characters": "Hello"}},
///         {"visible": true}
///     ]
/// });
/// remove_visible_only_objects(&mut tree).unwrap();
/// // symbolOverrides 现在只有 textData 对象
/// ```
pub fn remove_visible_only_objects(tree: &mut JsonValue) -> Result<()> {
    transform_recursive(tree);
    Ok(())
}

/// 从 JSON 值中递归删除仅可见对象
fn transform_recursive(value: &mut JsonValue) {
    match value {
        JsonValue::Object(map) => {
            // 首先，递归到所有值
            let keys: Vec<String> = map.keys().cloned().collect();
            for key in &keys {
                if let Some(val) = map.get_mut(key) {
                    transform_recursive(val);
                }
            }

            // 然后删除其值为仅可见对象的所有键
            map.retain(|_, v| !is_visible_only_object(v));
        }
        JsonValue::Array(arr) => {
            // 首先，递归到数组元素
            for val in arr.iter_mut() {
                transform_recursive(val);
            }

            // 然后从数组中过滤掉仅可见的对象
            arr.retain(|v| !is_visible_only_object(v));
        }
        _ => {
            // 原始值，无需处理
        }
    }
}

/// 检查 JSON 值是否是仅具有 "visible" 键的对象
fn is_visible_only_object(value: &JsonValue) -> bool {
    match value {
        JsonValue::Object(map) => map.len() == 1 && map.contains_key("visible"),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_remove_visible_only_from_array() {
        let mut tree = json!({
            "symbolOverrides": [
                {"visible": false},
                {"textData": {"characters": "Hello"}},
                {"visible": true}
            ]
        });

        remove_visible_only_objects(&mut tree).unwrap();

        let overrides = tree.get("symbolOverrides").unwrap().as_array().unwrap();
        assert_eq!(overrides.len(), 1);
        assert!(overrides[0].get("textData").is_some());
    }

    #[test]
    fn test_remove_visible_only_object_field() {
        let mut tree = json!({
            "name": "Shape",
            "metadata": {"visible": false},
            "opacity": 1.0
        });

        remove_visible_only_objects(&mut tree).unwrap();

        assert!(tree.get("metadata").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Shape"));
        assert_eq!(tree.get("opacity").unwrap().as_f64(), Some(1.0));
    }

    #[test]
    fn test_preserve_objects_with_visible_and_other_fields() {
        let mut tree = json!({
            "symbolOverrides": [
                {"visible": false, "opacity": 0.5},
                {"visible": true, "textData": {"characters": "Test"}},
                {"visible": false}
            ]
        });

        remove_visible_only_objects(&mut tree).unwrap();

        let overrides = tree.get("symbolOverrides").unwrap().as_array().unwrap();
        assert_eq!(overrides.len(), 2);
        assert!(overrides[0].get("visible").is_some());
        assert!(overrides[0].get("opacity").is_some());
        assert!(overrides[1].get("visible").is_some());
        assert!(overrides[1].get("textData").is_some());
    }

    #[test]
    fn test_nested_visible_only_objects() {
        let mut tree = json!({
            "children": [
                {
                    "name": "Child1",
                    "data": {"visible": true}
                },
                {
                    "name": "Child2",
                    "overrides": [
                        {"visible": false},
                        {"opacity": 0.5}
                    ]
                }
            ]
        });

        remove_visible_only_objects(&mut tree).unwrap();

        assert!(tree["children"][0].get("data").is_none());
        assert_eq!(tree["children"][0]["name"].as_str(), Some("Child1"));

        let overrides = tree["children"][1]["overrides"].as_array().unwrap();
        assert_eq!(overrides.len(), 1);
        assert!(overrides[0].get("opacity").is_some());
    }

    #[test]
    fn test_array_of_visible_only_objects() {
        let mut tree = json!({
            "items": [
                {"visible": false},
                {"visible": true},
                {"visible": false}
            ]
        });

        remove_visible_only_objects(&mut tree).unwrap();

        let items = tree.get("items").unwrap().as_array().unwrap();
        assert_eq!(items.len(), 0);
    }

    #[test]
    fn test_mixed_array() {
        let mut tree = json!({
            "items": [
                {"visible": false},
                {"name": "A"},
                {"visible": true},
                {"name": "B", "visible": false},
                {"visible": false}
            ]
        });

        remove_visible_only_objects(&mut tree).unwrap();

        let items = tree.get("items").unwrap().as_array().unwrap();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0]["name"].as_str(), Some("A"));
        assert_eq!(items[1]["name"].as_str(), Some("B"));
        // 第二项应该仍然可见，因为它也有其他字段
        assert!(items[1].get("visible").is_some());
    }

    #[test]
    fn test_no_visible_only_objects() {
        let mut tree = json!({
            "name": "Rectangle",
            "visible": true,
            "children": [
                {"name": "Child1", "visible": false},
                {"name": "Child2"}
            ]
        });

        let original = tree.clone();
        remove_visible_only_objects(&mut tree).unwrap();

        // 树应该保持不变，因为所有可见字段也有其他字段
        assert_eq!(tree, original);
    }

    #[test]
    fn test_real_world_roles_members_case() {
        // 来自 archives/roles-members.json 第 203 行
        let mut tree = json!({
            "symbolData": {
                "symbolOverrides": [
                    {
                        "textData": {
                            "characters": "Roles"
                        }
                    },
                    {
                        "textData": {
                            "characters": "Members"
                        }
                    },
                    {
                        "textData": {
                            "characters": "Audit"
                        }
                    },
                    {
                        "visible": false
                    },
                    {
                        "overrideLevel": 1,
                        "textData": {
                            "characters": "Commands"
                        }
                    }
                ],
                "uniformScaleFactor": 1.0
            }
        });

        remove_visible_only_objects(&mut tree).unwrap();

        let overrides = tree["symbolData"]["symbolOverrides"].as_array().unwrap();
        assert_eq!(overrides.len(), 4);
        // 验证仅可见对象是否已删除
        assert!(
            overrides
                .iter()
                .all(|o| o.get("textData").is_some() || o.get("overrideLevel").is_some())
        );
    }

    #[test]
    fn test_deeply_nested() {
        let mut tree = json!({
            "level1": {
                "level2": {
                    "level3": {
                        "visibleOnly": {"visible": true},
                        "data": "value"
                    },
                    "alsoVisibleOnly": {"visible": false}
                }
            }
        });

        remove_visible_only_objects(&mut tree).unwrap();

        assert!(tree["level1"]["level2"]["level3"].get("visibleOnly").is_none());
        assert_eq!(tree["level1"]["level2"]["level3"]["data"].as_str(), Some("value"));
        assert!(tree["level1"]["level2"].get("alsoVisibleOnly").is_none());
    }

    #[test]
    fn test_preserve_empty_objects() {
        let mut tree = json!({
            "name": "Test",
            "empty": {},
            "visibleOnly": {"visible": false}
        });

        remove_visible_only_objects(&mut tree).unwrap();

        // 应保留空对象，仅删除仅可见对象
        assert!(tree.get("empty").is_some());
        assert!(tree.get("visibleOnly").is_none());
    }

    #[test]
    fn test_visible_with_different_values() {
        let mut tree = json!({
            "items": [
                {"visible": false},
                {"visible": true},
                {"visible": 0},
                {"visible": 1},
                {"visible": null}
            ]
        });

        remove_visible_only_objects(&mut tree).unwrap();

        let items = tree.get("items").unwrap().as_array().unwrap();
        // 无论可见值如何，都应删除
        assert_eq!(items.len(), 0);
    }
}
