use crate::error::Result;
use serde_json::Value as JsonValue;

/// 从文档树中删除 Figma 的角平滑属性。
///
/// 此转换删除了 `cornerSmoothing` 字段，该字段控制
/// Figma 中圆角的平滑度。此功能创建 "iOS-style"
/// 圆角比标准圆弧更平滑。
///
/// 标准 CSS `border-radius` 只支持圆弧，没有
/// 相当于 Figma 的角平滑功能。因此，该属性
/// 对于 HTML/CSS 渲染没有用，可以安全删除。
///
/// # 示例
///
/// ```rust
/// use serde_json::json;
/// use fig2json::schema::remove_corner_smoothing;
///
/// let mut tree = json!({
///     "name": "Rectangle",
///     "cornerRadius": 12.0,
///     "cornerSmoothing": 0.6000000238418579,
///     "type": "ROUNDED_RECTANGLE"
/// });
///
/// remove_corner_smoothing(&mut tree).unwrap();
///
/// assert!(tree.get("cornerSmoothing").is_none());
/// assert!(tree.get("cornerRadius").is_some());
/// assert!(tree.get("type").is_some());
/// ```
pub fn remove_corner_smoothing(tree: &mut JsonValue) -> Result<()> {
    transform_recursive(tree)
}

fn transform_recursive(value: &mut JsonValue) -> Result<()> {
    match value {
        JsonValue::Object(map) => {
            // 删除角平滑属性
            map.remove("cornerSmoothing");

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
    fn test_removes_corner_smoothing() {
        let mut tree = json!({
            "name": "Rectangle",
            "cornerRadius": 12.0,
            "cornerSmoothing": 0.6000000238418579,
            "type": "ROUNDED_RECTANGLE"
        });

        remove_corner_smoothing(&mut tree).unwrap();

        assert!(tree.get("cornerSmoothing").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Rectangle"));
        assert_eq!(tree.get("cornerRadius").unwrap().as_f64(), Some(12.0));
    }

    #[test]
    fn test_removes_corner_smoothing_with_integer_value() {
        let mut tree = json!({
            "name": "Rectangle",
            "cornerSmoothing": 1,
            "type": "FRAME"
        });

        remove_corner_smoothing(&mut tree).unwrap();

        assert!(tree.get("cornerSmoothing").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Rectangle"));
    }

    #[test]
    fn test_removes_corner_smoothing_zero_value() {
        let mut tree = json!({
            "name": "Rectangle",
            "cornerSmoothing": 0.0,
            "type": "FRAME"
        });

        remove_corner_smoothing(&mut tree).unwrap();

        assert!(tree.get("cornerSmoothing").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Rectangle"));
    }

    #[test]
    fn test_handles_nested_objects() {
        let mut tree = json!({
            "name": "Parent",
            "cornerSmoothing": 0.5,
            "children": [
                {
                    "name": "Child1",
                    "cornerSmoothing": 0.6000000238418579
                },
                {
                    "name": "Child2",
                    "cornerRadius": 8.0,
                    "cornerSmoothing": 0.4
                }
            ]
        });

        remove_corner_smoothing(&mut tree).unwrap();

        assert!(tree.get("cornerSmoothing").is_none());
        let children = tree.get("children").unwrap().as_array().unwrap();
        assert!(children[0].get("cornerSmoothing").is_none());
        assert!(children[1].get("cornerSmoothing").is_none());
        assert_eq!(children[0].get("name").unwrap().as_str(), Some("Child1"));
        assert_eq!(children[1].get("cornerRadius").unwrap().as_f64(), Some(8.0));
    }

    #[test]
    fn test_handles_deeply_nested_structures() {
        let mut tree = json!({
            "name": "Root",
            "cornerSmoothing": 0.6,
            "children": [
                {
                    "name": "Level1",
                    "children": [
                        {
                            "name": "Level2",
                            "cornerRadius": 16.0,
                            "cornerSmoothing": 0.6000000238418579
                        }
                    ]
                }
            ]
        });

        remove_corner_smoothing(&mut tree).unwrap();

        assert!(tree.get("cornerSmoothing").is_none());
        let level1 = &tree.get("children").unwrap().as_array().unwrap()[0];
        let level2 = &level1.get("children").unwrap().as_array().unwrap()[0];
        assert!(level2.get("cornerSmoothing").is_none());
        assert_eq!(level2.get("cornerRadius").unwrap().as_f64(), Some(16.0));
        assert_eq!(level2.get("name").unwrap().as_str(), Some("Level2"));
    }

    #[test]
    fn test_handles_missing_corner_smoothing() {
        let mut tree = json!({
            "name": "Rectangle",
            "cornerRadius": 12.0,
            "type": "ROUNDED_RECTANGLE"
        });

        remove_corner_smoothing(&mut tree).unwrap();

        assert_eq!(tree.get("name").unwrap().as_str(), Some("Rectangle"));
        assert!(tree.get("cornerRadius").is_some());
        assert!(tree.get("type").is_some());
    }

    #[test]
    fn test_handles_empty_object() {
        let mut tree = json!({});

        remove_corner_smoothing(&mut tree).unwrap();

        assert_eq!(tree.as_object().unwrap().len(), 0);
    }

    #[test]
    fn test_preserves_other_fields() {
        let mut tree = json!({
            "name": "Button",
            "type": "FRAME",
            "cornerRadius": 12.0,
            "cornerSmoothing": 0.6000000238418579,
            "fillPaints": [
                {
                    "color": "#1461f6",
                    "type": "SOLID"
                }
            ],
            "size": {"x": 343.0, "y": 52.0},
            "stackMode": "HORIZONTAL"
        });

        remove_corner_smoothing(&mut tree).unwrap();

        assert!(tree.get("cornerSmoothing").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Button"));
        assert_eq!(tree.get("type").unwrap().as_str(), Some("FRAME"));
        assert_eq!(tree.get("cornerRadius").unwrap().as_f64(), Some(12.0));
        assert!(tree.get("fillPaints").is_some());
        assert!(tree.get("size").is_some());
        assert_eq!(tree.get("stackMode").unwrap().as_str(), Some("HORIZONTAL"));
    }

    #[test]
    fn test_handles_multiple_occurrences_in_array() {
        let mut tree = json!({
            "children": [
                {"name": "A", "cornerSmoothing": 0.5},
                {"name": "B", "cornerSmoothing": 0.6},
                {"name": "C", "cornerSmoothing": 0.0},
                {"name": "D"}
            ]
        });

        remove_corner_smoothing(&mut tree).unwrap();

        let children = tree.get("children").unwrap().as_array().unwrap();
        for child in children {
            assert!(child.get("cornerSmoothing").is_none());
            assert!(child.get("name").is_some());
        }
    }
}
