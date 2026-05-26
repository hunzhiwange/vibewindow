use crate::error::Result;
use serde_json::Value as JsonValue;

/// 从文档树中删除 Figma 特定的滚动和调整大小属性。
///
/// 此转换删除以下字段：
/// - `scrollBehavior`：控制当父级滚动时框架是否保持固定
/// - `resizeToFit`：控制框架是否自动调整大小以适应其内容
///
/// 这些属性特定于 Figma 的框架行为，HTML/CSS 渲染并不直接需要。
/// CSS 使用不同的机制，例如 `position: fixed`，
/// `position: sticky`，并使用 flexbox/grid 自动调整大小。
///
/// # 示例
///
/// ```rust
/// use serde_json::json;
/// use fig2json::schema::remove_scroll_resize_properties;
///
/// let mut tree = json!({
///     "name": "Frame",
///     "scrollBehavior": "FIXED_WHEN_CHILD_OF_SCROLLING_FRAME",
///     "resizeToFit": true,
///     "type": "FRAME"
/// });
///
/// remove_scroll_resize_properties(&mut tree).unwrap();
///
/// assert!(tree.get("scrollBehavior").is_none());
/// assert!(tree.get("resizeToFit").is_none());
/// assert!(tree.get("type").is_some());
/// ```
pub fn remove_scroll_resize_properties(tree: &mut JsonValue) -> Result<()> {
    transform_recursive(tree)
}

fn transform_recursive(value: &mut JsonValue) -> Result<()> {
    match value {
        JsonValue::Object(map) => {
            // 删除滚动和调整大小属性
            map.remove("scrollBehavior");
            map.remove("resizeToFit");

            // 递归到所有剩余值
            let keys: Vec<String> = map.keys().cloned().collect();
            for key in keys {
                if let Some(val) = map.get_mut(&key) {
                    transform_recursive(val)?;
                }
            }
        }
        JsonValue::Array(arr) => {
            // 递归到数组元素
            for val in arr.iter_mut() {
                transform_recursive(val)?;
            }
        }
        _ => {
            // 原始值，无需处理
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_removes_scroll_behavior() {
        let mut tree = json!({
            "name": "Frame",
            "scrollBehavior": "FIXED_WHEN_CHILD_OF_SCROLLING_FRAME",
            "type": "FRAME"
        });

        remove_scroll_resize_properties(&mut tree).unwrap();

        assert!(tree.get("scrollBehavior").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Frame"));
        assert_eq!(tree.get("type").unwrap().as_str(), Some("FRAME"));
    }

    #[test]
    fn test_removes_resize_to_fit() {
        let mut tree = json!({
            "name": "Frame",
            "resizeToFit": true,
            "type": "FRAME"
        });

        remove_scroll_resize_properties(&mut tree).unwrap();

        assert!(tree.get("resizeToFit").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Frame"));
    }

    #[test]
    fn test_removes_both_properties() {
        let mut tree = json!({
            "name": "Frame",
            "scrollBehavior": "SCROLLS",
            "resizeToFit": false,
            "type": "FRAME"
        });

        remove_scroll_resize_properties(&mut tree).unwrap();

        assert!(tree.get("scrollBehavior").is_none());
        assert!(tree.get("resizeToFit").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Frame"));
    }

    #[test]
    fn test_handles_nested_objects() {
        let mut tree = json!({
            "name": "Parent",
            "scrollBehavior": "FIXED_WHEN_CHILD_OF_SCROLLING_FRAME",
            "children": [
                {
                    "name": "Child1",
                    "resizeToFit": true
                },
                {
                    "name": "Child2",
                    "scrollBehavior": "SCROLLS",
                    "resizeToFit": false
                }
            ]
        });

        remove_scroll_resize_properties(&mut tree).unwrap();

        assert!(tree.get("scrollBehavior").is_none());
        let children = tree.get("children").unwrap().as_array().unwrap();
        assert!(children[0].get("resizeToFit").is_none());
        assert!(children[1].get("scrollBehavior").is_none());
        assert!(children[1].get("resizeToFit").is_none());
        assert_eq!(children[0].get("name").unwrap().as_str(), Some("Child1"));
    }

    #[test]
    fn test_handles_deeply_nested_structures() {
        let mut tree = json!({
            "name": "Root",
            "resizeToFit": true,
            "children": [
                {
                    "name": "Level1",
                    "scrollBehavior": "FIXED_WHEN_CHILD_OF_SCROLLING_FRAME",
                    "children": [
                        {
                            "name": "Level2",
                            "resizeToFit": false,
                            "scrollBehavior": "SCROLLS"
                        }
                    ]
                }
            ]
        });

        remove_scroll_resize_properties(&mut tree).unwrap();

        assert!(tree.get("resizeToFit").is_none());
        let level1 = &tree.get("children").unwrap().as_array().unwrap()[0];
        assert!(level1.get("scrollBehavior").is_none());
        let level2 = &level1.get("children").unwrap().as_array().unwrap()[0];
        assert!(level2.get("resizeToFit").is_none());
        assert!(level2.get("scrollBehavior").is_none());
        assert_eq!(level2.get("name").unwrap().as_str(), Some("Level2"));
    }

    #[test]
    fn test_handles_missing_properties() {
        let mut tree = json!({
            "name": "Frame",
            "type": "FRAME",
            "size": {"x": 100.0, "y": 100.0}
        });

        remove_scroll_resize_properties(&mut tree).unwrap();

        assert_eq!(tree.get("name").unwrap().as_str(), Some("Frame"));
        assert!(tree.get("type").is_some());
        assert!(tree.get("size").is_some());
    }

    #[test]
    fn test_handles_empty_object() {
        let mut tree = json!({});

        remove_scroll_resize_properties(&mut tree).unwrap();

        assert_eq!(tree.as_object().unwrap().len(), 0);
    }

    #[test]
    fn test_preserves_other_fields() {
        let mut tree = json!({
            "name": "Frame",
            "type": "FRAME",
            "scrollBehavior": "FIXED_WHEN_CHILD_OF_SCROLLING_FRAME",
            "resizeToFit": true,
            "stackMode": "HORIZONTAL",
            "size": {"x": 100.0, "y": 100.0},
            "children": []
        });

        remove_scroll_resize_properties(&mut tree).unwrap();

        assert!(tree.get("scrollBehavior").is_none());
        assert!(tree.get("resizeToFit").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Frame"));
        assert_eq!(tree.get("type").unwrap().as_str(), Some("FRAME"));
        assert_eq!(tree.get("stackMode").unwrap().as_str(), Some("HORIZONTAL"));
        assert!(tree.get("size").is_some());
        assert!(tree.get("children").is_some());
    }

    #[test]
    fn test_handles_multiple_occurrences_in_array() {
        let mut tree = json!({
            "children": [
                {"name": "A", "scrollBehavior": "SCROLLS"},
                {"name": "B", "resizeToFit": true},
                {"name": "C", "scrollBehavior": "FIXED_WHEN_CHILD_OF_SCROLLING_FRAME", "resizeToFit": false},
                {"name": "D"}
            ]
        });

        remove_scroll_resize_properties(&mut tree).unwrap();

        let children = tree.get("children").unwrap().as_array().unwrap();
        for child in children {
            assert!(child.get("scrollBehavior").is_none());
            assert!(child.get("resizeToFit").is_none());
            assert!(child.get("name").is_some());
        }
    }
}
