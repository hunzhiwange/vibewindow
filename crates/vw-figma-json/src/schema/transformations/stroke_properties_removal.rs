use crate::error::Result;
use serde_json::Value as JsonValue;

/// 从 JSON 树中的所有对象中删除笔画属性字段
///
/// 递归遍历JSON树并删除笔画相关字段：
/// - "strokeAlign" - CSS 不支持笔画对齐(内部/中心/外部)
/// - "strokeJoin" - 笔划连接样式(MITRE/BEVEL/ROUND)
/// - "strokeWeight" - 描边宽度值
///
/// 这些字段包含没有直接 CSS 等效项的笔划属性
/// 或对于基本 HTML/CSS 渲染来说不是必需的。
///
/// # 参数
/// * `tree` - 要修改的 JSON 树(通常是文档根)
///
/// # 返回值
/// * `Ok(())` - 成功删除所有笔画属性字段
///
/// # 示例
/// ```no_run
/// use fig2json::schema::remove_stroke_properties;
/// use serde_json::json;
///
/// let mut tree = json!({
///     "name": "Rectangle",
///     "strokeAlign": {
///         "__enum__": "StrokeAlign",
///         "value": "INSIDE"
///     },
///     "strokeJoin": {
///         "__enum__": "StrokeJoin",
///         "value": "MITER"
///     },
///     "strokeWeight": 1.0,
///     "visible": true
/// });
/// remove_stroke_properties(&mut tree).unwrap();
/// // 树现在只有 "name" 和 "visible" 字段
/// ```
pub fn remove_stroke_properties(tree: &mut JsonValue) -> Result<()> {
    transform_recursive(tree)
}

/// 从 JSON 值中递归删除笔划属性字段
fn transform_recursive(value: &mut JsonValue) -> Result<()> {
    match value {
        JsonValue::Object(map) => {
            // 删除笔画属性字段(如果存在)
            map.remove("strokeAlign");
            map.remove("strokeJoin");
            map.remove("strokeWeight");

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
    fn test_remove_stroke_align() {
        let mut tree = json!({
            "name": "Rectangle",
            "strokeAlign": {
                "__enum__": "StrokeAlign",
                "value": "INSIDE"
            },
            "visible": true
        });

        remove_stroke_properties(&mut tree).unwrap();

        assert!(tree.get("strokeAlign").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Rectangle"));
        assert_eq!(tree.get("visible").unwrap().as_bool(), Some(true));
    }

    #[test]
    fn test_remove_stroke_join() {
        let mut tree = json!({
            "name": "Line",
            "strokeJoin": {
                "__enum__": "StrokeJoin",
                "value": "MITER"
            },
            "opacity": 1.0
        });

        remove_stroke_properties(&mut tree).unwrap();

        assert!(tree.get("strokeJoin").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Line"));
        assert_eq!(tree.get("opacity").unwrap().as_f64(), Some(1.0));
    }

    #[test]
    fn test_remove_stroke_weight() {
        let mut tree = json!({
            "name": "Shape",
            "strokeWeight": 1.0,
            "visible": true
        });

        remove_stroke_properties(&mut tree).unwrap();

        assert!(tree.get("strokeWeight").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Shape"));
        assert_eq!(tree.get("visible").unwrap().as_bool(), Some(true));
    }

    #[test]
    fn test_remove_all_stroke_properties() {
        let mut tree = json!({
            "name": "Rectangle",
            "strokeAlign": {
                "__enum__": "StrokeAlign",
                "value": "CENTER"
            },
            "strokeJoin": {
                "__enum__": "StrokeJoin",
                "value": "BEVEL"
            },
            "strokeWeight": 2.5,
            "opacity": 1.0
        });

        remove_stroke_properties(&mut tree).unwrap();

        assert!(tree.get("strokeAlign").is_none());
        assert!(tree.get("strokeJoin").is_none());
        assert!(tree.get("strokeWeight").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Rectangle"));
        assert_eq!(tree.get("opacity").unwrap().as_f64(), Some(1.0));
    }

    #[test]
    fn test_nested_stroke_properties() {
        let mut tree = json!({
            "name": "Root",
            "children": [
                {
                    "name": "Child1",
                    "strokeAlign": {
                        "__enum__": "StrokeAlign",
                        "value": "INSIDE"
                    },
                    "strokeWeight": 1.0
                },
                {
                    "name": "Child2",
                    "children": [
                        {
                            "name": "DeepChild",
                            "strokeJoin": {
                                "__enum__": "StrokeJoin",
                                "value": "ROUND"
                            }
                        }
                    ]
                }
            ]
        });

        remove_stroke_properties(&mut tree).unwrap();

        // 检查第一个嵌套元素
        assert!(tree["children"][0].get("strokeAlign").is_none());
        assert!(tree["children"][0].get("strokeWeight").is_none());
        assert_eq!(tree["children"][0].get("name").unwrap().as_str(), Some("Child1"));

        // 检查深层嵌套元素
        let deep_child = &tree["children"][1]["children"][0];
        assert!(deep_child.get("strokeJoin").is_none());
        assert_eq!(deep_child.get("name").unwrap().as_str(), Some("DeepChild"));
    }

    #[test]
    fn test_no_stroke_properties() {
        let mut tree = json!({
            "name": "Rectangle",
            "width": 100,
            "height": 200,
            "visible": true
        });

        remove_stroke_properties(&mut tree).unwrap();

        // 没有笔画属性的树应该保持不变
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Rectangle"));
        assert_eq!(tree.get("width").unwrap().as_i64(), Some(100));
        assert_eq!(tree.get("height").unwrap().as_i64(), Some(200));
        assert_eq!(tree.get("visible").unwrap().as_bool(), Some(true));
    }

    #[test]
    fn test_preserves_stroke_paints() {
        let mut tree = json!({
            "name": "Line",
            "strokeAlign": {
                "__enum__": "StrokeAlign",
                "value": "CENTER"
            },
            "strokeWeight": 1.0,
            "strokePaints": [
                {
                    "blendMode": {
                        "__enum__": "BlendMode",
                        "value": "NORMAL"
                    },
                    "color": "#000000",
                    "opacity": 1.0,
                    "visible": true
                }
            ]
        });

        remove_stroke_properties(&mut tree).unwrap();

        // 已删除笔画属性
        assert!(tree.get("strokeAlign").is_none());
        assert!(tree.get("strokeWeight").is_none());

        // 保留笔触颜色(包含实际笔触颜色数据)
        assert!(tree.get("strokePaints").is_some());
        assert_eq!(tree["strokePaints"][0]["color"].as_str(), Some("#000000"));
    }

    #[test]
    fn test_multiple_objects_with_stroke_properties() {
        let mut tree = json!({
            "items": [
                {
                    "name": "Item1",
                    "strokeAlign": {
                        "__enum__": "StrokeAlign",
                        "value": "INSIDE"
                    }
                },
                {
                    "name": "Item2",
                    "strokeJoin": {
                        "__enum__": "StrokeJoin",
                        "value": "MITER"
                    }
                },
                {
                    "name": "Item3",
                    "strokeWeight": 3.0
                }
            ]
        });

        remove_stroke_properties(&mut tree).unwrap();

        // 数组中的所有笔画属性应被删除
        assert!(tree["items"][0].get("strokeAlign").is_none());
        assert_eq!(tree["items"][0].get("name").unwrap().as_str(), Some("Item1"));

        assert!(tree["items"][1].get("strokeJoin").is_none());
        assert_eq!(tree["items"][1].get("name").unwrap().as_str(), Some("Item2"));

        assert!(tree["items"][2].get("strokeWeight").is_none());
        assert_eq!(tree["items"][2].get("name").unwrap().as_str(), Some("Item3"));
    }

    #[test]
    fn test_different_stroke_align_values() {
        let mut tree = json!({
            "shape1": {
                "strokeAlign": {
                    "__enum__": "StrokeAlign",
                    "value": "INSIDE"
                }
            },
            "shape2": {
                "strokeAlign": {
                    "__enum__": "StrokeAlign",
                    "value": "CENTER"
                }
            },
            "shape3": {
                "strokeAlign": {
                    "__enum__": "StrokeAlign",
                    "value": "OUTSIDE"
                }
            }
        });

        remove_stroke_properties(&mut tree).unwrap();

        // 应该删除所有的StrokeAlign变体
        assert!(tree["shape1"].get("strokeAlign").is_none());
        assert!(tree["shape2"].get("strokeAlign").is_none());
        assert!(tree["shape3"].get("strokeAlign").is_none());
    }
}
