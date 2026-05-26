use crate::error::Result;
use serde_json::Value as JsonValue;

/// 当 BlendMode 字段具有默认值 "NORMAL" 时，删除它
///
/// 递归遍历 JSON 树并删除具有以下属性的 "blendMode" 字段
/// 值 "NORMAL" (在枚举简化将它们从枚举转换之后)
/// 对象到字符串)。 NORMAL 是 Figma 和 CSS 中的默认混合模式，
/// 因此省略它会减少输出大小而不丢失信息。
///
/// 重要提示：此转换必须在 enum_simplification 之后运行，这
/// converts `{"__enum__": "BlendMode", "value": "NORMAL"}` to `"NORMAL"`.
///
/// # 参数
/// * `tree` - 要修改的 JSON 树(通常是文档根)
///
/// # 返回值
/// * `Ok(())` - 成功删除所有默认的混合模式字段
///
/// # 示例
/// ```no_run
/// use fig2json::schema::remove_default_blend_mode;
/// use serde_json::json;
///
/// let mut tree = json!({
///     "name": "Shape",
///     "blendMode": "NORMAL",
///     "opacity": 1.0
/// });
/// remove_default_blend_mode(&mut tree).unwrap();
/// // 树现在只有 "name" 和 "opacity" 字段
/// ```
pub fn remove_default_blend_mode(tree: &mut JsonValue) -> Result<()> {
    transform_recursive(tree)
}

/// 从 JSON 值中递归删除默认的 blendMode 字段
fn transform_recursive(value: &mut JsonValue) -> Result<()> {
    match value {
        JsonValue::Object(map) => {
            // 检查 blendMode 是否存在且为 "NORMAL"
            if let Some(blend_mode) = map.get("blendMode")
                && let Some(s) = blend_mode.as_str()
                && s == "NORMAL"
            {
                map.remove("blendMode");
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
    fn test_remove_normal_blend_mode() {
        let mut tree = json!({
            "name": "Shape",
            "blendMode": "NORMAL",
            "opacity": 1.0
        });

        remove_default_blend_mode(&mut tree).unwrap();

        assert!(tree.get("blendMode").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Shape"));
        assert_eq!(tree.get("opacity").unwrap().as_f64(), Some(1.0));
    }

    #[test]
    fn test_preserve_non_normal_blend_mode() {
        let mut tree = json!({
            "name": "Shape",
            "blendMode": "MULTIPLY",
            "opacity": 0.8
        });

        remove_default_blend_mode(&mut tree).unwrap();

        // 应保留非正常混合模式
        assert_eq!(tree.get("blendMode").unwrap().as_str(), Some("MULTIPLY"));
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Shape"));
        assert_eq!(tree.get("opacity").unwrap().as_f64(), Some(0.8));
    }

    #[test]
    fn test_preserve_other_blend_modes() {
        let modes = vec!["MULTIPLY", "SCREEN", "OVERLAY", "DARKEN", "LIGHTEN"];

        for mode in modes {
            let mut tree = json!({
                "blendMode": mode
            });

            remove_default_blend_mode(&mut tree).unwrap();

            // 应保留所有非正常混合模式
            assert_eq!(tree.get("blendMode").unwrap().as_str(), Some(mode));
        }
    }

    #[test]
    fn test_no_blend_mode() {
        let mut tree = json!({
            "name": "Rectangle",
            "width": 100,
            "height": 200
        });

        remove_default_blend_mode(&mut tree).unwrap();

        // 没有 BlendMode 的树应该保持不变
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Rectangle"));
        assert_eq!(tree.get("width").unwrap().as_i64(), Some(100));
        assert!(tree.get("blendMode").is_none());
    }

    #[test]
    fn test_nested_objects() {
        let mut tree = json!({
            "children": [
                {
                    "name": "Child1",
                    "blendMode": "NORMAL"
                },
                {
                    "name": "Child2",
                    "blendMode": "MULTIPLY"
                }
            ]
        });

        remove_default_blend_mode(&mut tree).unwrap();

        // NORMAL 已删除，MULTIPLY 已保留
        assert!(tree["children"][0].get("blendMode").is_none());
        assert_eq!(tree["children"][0]["name"].as_str(), Some("Child1"));

        assert_eq!(tree["children"][1]["blendMode"].as_str(), Some("MULTIPLY"));
        assert_eq!(tree["children"][1]["name"].as_str(), Some("Child2"));
    }

    #[test]
    fn test_blend_mode_in_paints() {
        let mut tree = json!({
            "fillPaints": [
                {
                    "type": "SOLID",
                    "blendMode": "NORMAL",
                    "color": "#ff0000"
                },
                {
                    "type": "GRADIENT",
                    "blendMode": "MULTIPLY",
                    "color": "#00ff00"
                }
            ]
        });

        remove_default_blend_mode(&mut tree).unwrap();

        // NORMAL 从第一个paint中移除
        assert!(tree["fillPaints"][0].get("blendMode").is_none());
        assert_eq!(tree["fillPaints"][0]["type"].as_str(), Some("SOLID"));

        // 相乘保留在第二个paint中
        assert_eq!(tree["fillPaints"][1]["blendMode"].as_str(), Some("MULTIPLY"));
        assert_eq!(tree["fillPaints"][1]["type"].as_str(), Some("GRADIENT"));
    }

    #[test]
    fn test_deeply_nested() {
        let mut tree = json!({
            "document": {
                "children": [
                    {
                        "type": "FRAME",
                        "blendMode": "NORMAL",
                        "fillPaints": [
                            {
                                "type": "SOLID",
                                "blendMode": "NORMAL"
                            }
                        ]
                    }
                ]
            }
        });

        remove_default_blend_mode(&mut tree).unwrap();

        // 所有级别的所有 NORMAL 混合模式都应删除
        let frame = &tree["document"]["children"][0];
        assert!(frame.get("blendMode").is_none());
        assert!(frame["fillPaints"][0].get("blendMode").is_none());
        assert_eq!(frame["type"].as_str(), Some("FRAME"));
    }

    #[test]
    fn test_blend_mode_enum_object_not_touched() {
        let mut tree = json!({
            "name": "Shape",
            "blendMode": {
                "__enum__": "BlendMode",
                "value": "NORMAL"
            }
        });

        remove_default_blend_mode(&mut tree).unwrap();

        // 不应触及枚举对象(这在 enum_simplification 之后运行)
        // 所以这应该保留原样
        assert!(tree.get("blendMode").is_some());
        let blend_mode = tree.get("blendMode").unwrap();
        assert!(blend_mode.is_object());
    }

    #[test]
    fn test_case_sensitive() {
        let mut tree = json!({
            "blendMode": "normal"
        });

        remove_default_blend_mode(&mut tree).unwrap();

        // 小写 "normal" 不应被删除(仅 "NORMAL")
        assert_eq!(tree.get("blendMode").unwrap().as_str(), Some("normal"));
    }

    #[test]
    fn test_multiple_normal_blend_modes() {
        let mut tree = json!({
            "children": [
                {"blendMode": "NORMAL", "name": "A"},
                {"blendMode": "NORMAL", "name": "B"},
                {"blendMode": "NORMAL", "name": "C"}
            ]
        });

        remove_default_blend_mode(&mut tree).unwrap();

        // 所有正常混合模式应被删除
        assert!(tree["children"][0].get("blendMode").is_none());
        assert!(tree["children"][1].get("blendMode").is_none());
        assert!(tree["children"][2].get("blendMode").is_none());
        assert_eq!(tree["children"][0]["name"].as_str(), Some("A"));
        assert_eq!(tree["children"][1]["name"].as_str(), Some("B"));
        assert_eq!(tree["children"][2]["name"].as_str(), Some("C"));
    }
}
