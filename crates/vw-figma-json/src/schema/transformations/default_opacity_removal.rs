use crate::error::Result;
use serde_json::Value as JsonValue;

/// 当不透明度字段具有默认值 1.0 时删除它
///
/// 递归遍历 JSON 树并删除具有以下属性的 "opacity" 字段
/// 值 1.0。由于 1.0 是 Figma 和 CSS 中的默认不透明度，
/// 省略它会减少输出大小而不丢失信息。
///
/// # 参数
/// * `tree` - 要修改的 JSON 树(通常是文档根)
///
/// # 返回值
/// * `Ok(())` - 成功删除所有默认不透明度字段
///
/// # 示例
/// ```no_run
/// use fig2json::schema::remove_default_opacity;
/// use serde_json::json;
///
/// let mut tree = json!({
///     "name": "Shape",
///     "opacity": 1.0,
///     "visible": true
/// });
/// remove_default_opacity(&mut tree).unwrap();
/// // 树现在只有 "name" 和 "visible" 字段
/// ```
pub fn remove_default_opacity(tree: &mut JsonValue) -> Result<()> {
    transform_recursive(tree)
}

/// 从 JSON 值中递归删除默认不透明度字段
fn transform_recursive(value: &mut JsonValue) -> Result<()> {
    match value {
        JsonValue::Object(map) => {
            // 检查不透明度是否存在且是否为 1.0
            if let Some(opacity) = map.get("opacity")
                && let Some(n) = opacity.as_f64()
            {
                // 使用 epsilon 比较浮点
                if (n - 1.0).abs() < f64::EPSILON {
                    map.remove("opacity");
                }
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
    fn test_remove_default_opacity() {
        let mut tree = json!({
            "name": "Shape",
            "opacity": 1.0,
            "visible": true
        });

        remove_default_opacity(&mut tree).unwrap();

        assert!(tree.get("opacity").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Shape"));
        assert_eq!(tree.get("visible").unwrap().as_bool(), Some(true));
    }

    #[test]
    fn test_preserve_non_default_opacity() {
        let mut tree = json!({
            "name": "Shape",
            "opacity": 0.5,
            "visible": true
        });

        remove_default_opacity(&mut tree).unwrap();

        // 应保留非 1.0 不透明度
        assert_eq!(tree.get("opacity").unwrap().as_f64(), Some(0.5));
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Shape"));
    }

    #[test]
    fn test_preserve_zero_opacity() {
        let mut tree = json!({
            "name": "Shape",
            "opacity": 0.0
        });

        remove_default_opacity(&mut tree).unwrap();

        // 应保留零不透明度
        assert_eq!(tree.get("opacity").unwrap().as_f64(), Some(0.0));
    }

    #[test]
    fn test_preserve_various_opacities() {
        let opacities = vec![0.0, 0.25, 0.5, 0.75, 0.9, 0.99];

        for opacity_value in opacities {
            let mut tree = json!({
                "opacity": opacity_value
            });

            remove_default_opacity(&mut tree).unwrap();

            // 应保留所有非 1.0 不透明度
            assert_eq!(tree.get("opacity").unwrap().as_f64(), Some(opacity_value));
        }
    }

    #[test]
    fn test_no_opacity() {
        let mut tree = json!({
            "name": "Rectangle",
            "width": 100,
            "height": 200
        });

        remove_default_opacity(&mut tree).unwrap();

        // 不透明的树应该保持不变
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Rectangle"));
        assert_eq!(tree.get("width").unwrap().as_i64(), Some(100));
        assert!(tree.get("opacity").is_none());
    }

    #[test]
    fn test_nested_objects() {
        let mut tree = json!({
            "children": [
                {
                    "name": "Child1",
                    "opacity": 1.0
                },
                {
                    "name": "Child2",
                    "opacity": 0.7
                }
            ]
        });

        remove_default_opacity(&mut tree).unwrap();

        // 不透明度 1.0 已删除，0.7 保留
        assert!(tree["children"][0].get("opacity").is_none());
        assert_eq!(tree["children"][0]["name"].as_str(), Some("Child1"));

        assert_eq!(tree["children"][1]["opacity"].as_f64(), Some(0.7));
        assert_eq!(tree["children"][1]["name"].as_str(), Some("Child2"));
    }

    #[test]
    fn test_opacity_in_paints() {
        let mut tree = json!({
            "fillPaints": [
                {
                    "type": "SOLID",
                    "opacity": 1.0,
                    "color": "#ff0000"
                },
                {
                    "type": "GRADIENT",
                    "opacity": 0.8,
                    "color": "#00ff00"
                }
            ]
        });

        remove_default_opacity(&mut tree).unwrap();

        // 不透明度 1.0 从第一次绘制中移除
        assert!(tree["fillPaints"][0].get("opacity").is_none());
        assert_eq!(tree["fillPaints"][0]["type"].as_str(), Some("SOLID"));

        // 不透明度 0.8 保留在第二个涂料中
        assert_eq!(tree["fillPaints"][1]["opacity"].as_f64(), Some(0.8));
        assert_eq!(tree["fillPaints"][1]["type"].as_str(), Some("GRADIENT"));
    }

    #[test]
    fn test_deeply_nested() {
        let mut tree = json!({
            "document": {
                "children": [
                    {
                        "type": "FRAME",
                        "opacity": 1.0,
                        "fillPaints": [
                            {
                                "type": "SOLID",
                                "opacity": 1.0
                            }
                        ]
                    }
                ]
            }
        });

        remove_default_opacity(&mut tree).unwrap();

        // 所有级别的不透明度 1.0 都应被删除
        let frame = &tree["document"]["children"][0];
        assert!(frame.get("opacity").is_none());
        assert!(frame["fillPaints"][0].get("opacity").is_none());
        assert_eq!(frame["type"].as_str(), Some("FRAME"));
    }

    #[test]
    fn test_multiple_default_opacities() {
        let mut tree = json!({
            "children": [
                {"opacity": 1.0, "name": "A"},
                {"opacity": 1.0, "name": "B"},
                {"opacity": 1.0, "name": "C"}
            ]
        });

        remove_default_opacity(&mut tree).unwrap();

        // 所有不透明度 1.0 应被删除
        assert!(tree["children"][0].get("opacity").is_none());
        assert!(tree["children"][1].get("opacity").is_none());
        assert!(tree["children"][2].get("opacity").is_none());
        assert_eq!(tree["children"][0]["name"].as_str(), Some("A"));
        assert_eq!(tree["children"][1]["name"].as_str(), Some("B"));
        assert_eq!(tree["children"][2]["name"].as_str(), Some("C"));
    }

    #[test]
    fn test_opacity_as_integer() {
        let mut tree = json!({
            "name": "Shape",
            "opacity": 1
        });

        remove_default_opacity(&mut tree).unwrap();

        // 整数 1 也应该被删除(因为 1 == 1.0)
        assert!(tree.get("opacity").is_none());
    }

    #[test]
    fn test_opacity_string_not_touched() {
        let mut tree = json!({
            "name": "Test",
            "opacity": "1.0"
        });

        remove_default_opacity(&mut tree).unwrap();

        // 不应触及字符串不透明度
        assert_eq!(tree.get("opacity").unwrap().as_str(), Some("1.0"));
    }
}
