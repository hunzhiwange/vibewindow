use crate::error::Result;
use serde_json::Value as JsonValue;

/// 当可见字段具有默认值 true 时删除它
///
/// 递归遍历 JSON 树并删除具有以下属性的 "visible" 字段
/// 值为真。由于 true 是 Figma 和 CSS 中的默认可见性，
/// 省略它会减少输出大小而不丢失信息。
///
/// # 参数
/// * `tree` - 要修改的 JSON 树(通常是文档根)
///
/// # 返回值
/// * `Ok(())` - 成功删除所有默认可见字段
///
/// # 示例
/// ```no_run
/// use fig2json::schema::remove_default_visible;
/// use serde_json::json;
///
/// let mut tree = json!({
///     "name": "Shape",
///     "visible": true,
///     "opacity": 0.5
/// });
/// remove_default_visible(&mut tree).unwrap();
/// // 树现在只有 "name" 和 "opacity" 字段
/// ```
pub fn remove_default_visible(tree: &mut JsonValue) -> Result<()> {
    transform_recursive(tree)
}

/// 从 JSON 值中递归删除默认可见字段
fn transform_recursive(value: &mut JsonValue) -> Result<()> {
    match value {
        JsonValue::Object(map) => {
            // 检查visible是否存在并且为true
            if let Some(visible) = map.get("visible")
                && let Some(b) = visible.as_bool()
                && b
            {
                map.remove("visible");
            }

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
    fn test_remove_default_visible() {
        let mut tree = json!({
            "name": "Shape",
            "visible": true,
            "opacity": 0.5
        });

        remove_default_visible(&mut tree).unwrap();

        assert!(tree.get("visible").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Shape"));
        assert_eq!(tree.get("opacity").unwrap().as_f64(), Some(0.5));
    }

    #[test]
    fn test_preserve_visible_false() {
        let mut tree = json!({
            "name": "Shape",
            "visible": false,
            "opacity": 1.0
        });

        remove_default_visible(&mut tree).unwrap();

        // 可见：应保留 false
        assert_eq!(tree.get("visible").unwrap().as_bool(), Some(false));
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Shape"));
    }

    #[test]
    fn test_no_visible() {
        let mut tree = json!({
            "name": "Rectangle",
            "width": 100,
            "height": 200
        });

        remove_default_visible(&mut tree).unwrap();

        // 不可见的树应该保持不变
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Rectangle"));
        assert_eq!(tree.get("width").unwrap().as_i64(), Some(100));
        assert!(tree.get("visible").is_none());
    }

    #[test]
    fn test_nested_objects() {
        let mut tree = json!({
            "children": [
                {
                    "name": "Child1",
                    "visible": true
                },
                {
                    "name": "Child2",
                    "visible": false
                }
            ]
        });

        remove_default_visible(&mut tree).unwrap();

        // 可见: true 已删除, 可见: false 保留
        assert!(tree["children"][0].get("visible").is_none());
        assert_eq!(tree["children"][0]["name"].as_str(), Some("Child1"));

        assert_eq!(tree["children"][1]["visible"].as_bool(), Some(false));
        assert_eq!(tree["children"][1]["name"].as_str(), Some("Child2"));
    }

    #[test]
    fn test_visible_in_paints() {
        let mut tree = json!({
            "fillPaints": [
                {
                    "type": "SOLID",
                    "visible": true,
                    "color": "#ff0000"
                },
                {
                    "type": "GRADIENT",
                    "visible": false,
                    "color": "#00ff00"
                }
            ]
        });

        remove_default_visible(&mut tree).unwrap();

        // 可见：true 从第一次绘制中删除
        assert!(tree["fillPaints"][0].get("visible").is_none());
        assert_eq!(tree["fillPaints"][0]["type"].as_str(), Some("SOLID"));

        // 可见：错误保留在第二个paint中
        assert_eq!(tree["fillPaints"][1]["visible"].as_bool(), Some(false));
        assert_eq!(tree["fillPaints"][1]["type"].as_str(), Some("GRADIENT"));
    }

    #[test]
    fn test_deeply_nested() {
        let mut tree = json!({
            "document": {
                "children": [
                    {
                        "type": "FRAME",
                        "visible": true,
                        "fillPaints": [
                            {
                                "type": "SOLID",
                                "visible": true
                            }
                        ]
                    }
                ]
            }
        });

        remove_default_visible(&mut tree).unwrap();

        // 所有可见：true 应在所有级别删除
        let frame = &tree["document"]["children"][0];
        assert!(frame.get("visible").is_none());
        assert!(frame["fillPaints"][0].get("visible").is_none());
        assert_eq!(frame["type"].as_str(), Some("FRAME"));
    }

    #[test]
    fn test_multiple_default_visible() {
        let mut tree = json!({
            "children": [
                {"visible": true, "name": "A"},
                {"visible": true, "name": "B"},
                {"visible": true, "name": "C"}
            ]
        });

        remove_default_visible(&mut tree).unwrap();

        // 所有可见： true 应删除
        assert!(tree["children"][0].get("visible").is_none());
        assert!(tree["children"][1].get("visible").is_none());
        assert!(tree["children"][2].get("visible").is_none());
        assert_eq!(tree["children"][0]["name"].as_str(), Some("A"));
        assert_eq!(tree["children"][1]["name"].as_str(), Some("B"));
        assert_eq!(tree["children"][2]["name"].as_str(), Some("C"));
    }

    #[test]
    fn test_multiple_false_visible() {
        let mut tree = json!({
            "children": [
                {"visible": false, "name": "A"},
                {"visible": false, "name": "B"},
                {"visible": false, "name": "C"}
            ]
        });

        remove_default_visible(&mut tree).unwrap();

        // 所有可见：应保留 false
        assert_eq!(tree["children"][0]["visible"].as_bool(), Some(false));
        assert_eq!(tree["children"][1]["visible"].as_bool(), Some(false));
        assert_eq!(tree["children"][2]["visible"].as_bool(), Some(false));
        assert_eq!(tree["children"][0]["name"].as_str(), Some("A"));
        assert_eq!(tree["children"][1]["name"].as_str(), Some("B"));
        assert_eq!(tree["children"][2]["name"].as_str(), Some("C"));
    }

    #[test]
    fn test_mixed_visible_values() {
        let mut tree = json!({
            "children": [
                {"visible": true, "name": "A"},
                {"visible": false, "name": "B"},
                {"visible": true, "name": "C"},
                {"visible": false, "name": "D"}
            ]
        });

        remove_default_visible(&mut tree).unwrap();

        // true removed, false preserved
        assert!(tree["children"][0].get("visible").is_none());
        assert_eq!(tree["children"][1]["visible"].as_bool(), Some(false));
        assert!(tree["children"][2].get("visible").is_none());
        assert_eq!(tree["children"][3]["visible"].as_bool(), Some(false));
    }

    #[test]
    fn test_visible_string_not_touched() {
        let mut tree = json!({
            "name": "Test",
            "visible": "true"
        });

        remove_default_visible(&mut tree).unwrap();

        // 可见字符串不应被触摸
        assert_eq!(tree.get("visible").unwrap().as_str(), Some("true"));
    }
}
