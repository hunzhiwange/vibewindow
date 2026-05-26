use crate::error::Result;
use serde_json::Value as JsonValue;

/// 从 JSON 树中的所有对象中删除单个边框权重字段
///
/// 递归遍历 JSON 树并删除 Figma 特定的边框权重字段：
/// - "borderTopWeight" - 顶部边框粗细
/// - "borderBottomWeight" - 底部边框粗细
/// - "borderLeftWeight" - 左边框粗细
/// - "borderRightWeight" - 右边框粗细
/// - "borderStrokeWeightsIndependent" - 指示独立边界权重的标志
///
/// 这些字段允许 Figma 中的每边边框权重，但标准 HTML/CSS
/// uses uniform borders. For HTML/CSS rendering, these detailed border weights
/// 不需要，可以安全删除。
///
/// # 参数
/// * `tree` - 要修改的 JSON 树(通常是文档根)
///
/// # 返回值
/// * `Ok(())` - 成功删除所有边框权重字段
///
/// # 示例
/// ```no_run
/// use fig2json::schema::remove_border_weights;
/// use serde_json::json;
///
/// let mut tree = json!({
///     "name": "Rectangle",
///     "borderTopWeight": 1.0,
///     "borderBottomWeight": 1.0,
///     "borderLeftWeight": 1.0,
///     "borderRightWeight": 1.0,
///     "borderStrokeWeightsIndependent": true,
///     "visible": true
/// });
/// remove_border_weights(&mut tree).unwrap();
/// // 树现在只有 "name" 和 "visible" 字段
/// ```
pub fn remove_border_weights(tree: &mut JsonValue) -> Result<()> {
    transform_recursive(tree)
}

/// 从 JSON 值中递归删除边框权重字段
fn transform_recursive(value: &mut JsonValue) -> Result<()> {
    match value {
        JsonValue::Object(map) => {
            // 删除所有边框权重字段(如果存在)
            map.remove("borderTopWeight");
            map.remove("borderBottomWeight");
            map.remove("borderLeftWeight");
            map.remove("borderRightWeight");
            map.remove("borderStrokeWeightsIndependent");

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
    fn test_remove_all_border_weights() {
        let mut tree = json!({
            "name": "Rectangle",
            "borderTopWeight": 1.0,
            "borderBottomWeight": 2.0,
            "borderLeftWeight": 1.5,
            "borderRightWeight": 2.5,
            "visible": true
        });

        remove_border_weights(&mut tree).unwrap();

        assert!(tree.get("borderTopWeight").is_none());
        assert!(tree.get("borderBottomWeight").is_none());
        assert!(tree.get("borderLeftWeight").is_none());
        assert!(tree.get("borderRightWeight").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Rectangle"));
        assert_eq!(tree.get("visible").unwrap().as_bool(), Some(true));
    }

    #[test]
    fn test_remove_partial_border_weights() {
        let mut tree = json!({
            "name": "Shape",
            "borderTopWeight": 1.0,
            "borderLeftWeight": 1.0,
            "width": 100
        });

        remove_border_weights(&mut tree).unwrap();

        assert!(tree.get("borderTopWeight").is_none());
        assert!(tree.get("borderLeftWeight").is_none());
        assert_eq!(tree.get("width").unwrap().as_i64(), Some(100));
    }

    #[test]
    fn test_no_border_weights() {
        let mut tree = json!({
            "name": "Circle",
            "radius": 50,
            "visible": true
        });

        remove_border_weights(&mut tree).unwrap();

        // 没有边界权重的树应该保持不变
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Circle"));
        assert_eq!(tree.get("radius").unwrap().as_i64(), Some(50));
        assert_eq!(tree.get("visible").unwrap().as_bool(), Some(true));
    }

    #[test]
    fn test_nested_objects() {
        let mut tree = json!({
            "children": [
                {
                    "name": "Child1",
                    "borderTopWeight": 1.0,
                    "borderBottomWeight": 1.0
                },
                {
                    "name": "Child2",
                    "borderLeftWeight": 2.0,
                    "borderRightWeight": 2.0
                }
            ]
        });

        remove_border_weights(&mut tree).unwrap();

        // 所有嵌套边框权重应被删除
        assert!(tree["children"][0].get("borderTopWeight").is_none());
        assert!(tree["children"][0].get("borderBottomWeight").is_none());
        assert_eq!(tree["children"][0]["name"].as_str(), Some("Child1"));

        assert!(tree["children"][1].get("borderLeftWeight").is_none());
        assert!(tree["children"][1].get("borderRightWeight").is_none());
        assert_eq!(tree["children"][1]["name"].as_str(), Some("Child2"));
    }

    #[test]
    fn test_deeply_nested() {
        let mut tree = json!({
            "document": {
                "children": [
                    {
                        "type": "FRAME",
                        "borderTopWeight": 1.0,
                        "borderBottomWeight": 1.0,
                        "borderLeftWeight": 1.0,
                        "borderRightWeight": 1.0
                    }
                ]
            }
        });

        remove_border_weights(&mut tree).unwrap();

        let frame = &tree["document"]["children"][0];
        assert!(frame.get("borderTopWeight").is_none());
        assert!(frame.get("borderBottomWeight").is_none());
        assert!(frame.get("borderLeftWeight").is_none());
        assert!(frame.get("borderRightWeight").is_none());
        assert_eq!(frame["type"].as_str(), Some("FRAME"));
    }

    #[test]
    fn test_preserve_other_border_properties() {
        let mut tree = json!({
            "name": "Rectangle",
            "borderTopWeight": 1.0,
            "borderRadius": 5.0,
            "borderColor": "#ff0000"
        });

        remove_border_weights(&mut tree).unwrap();

        // 应保留其他边框属性
        assert!(tree.get("borderTopWeight").is_none());
        assert_eq!(tree.get("borderRadius").unwrap().as_f64(), Some(5.0));
        assert_eq!(tree.get("borderColor").unwrap().as_str(), Some("#ff0000"));
    }

    #[test]
    fn test_multiple_shapes_in_array() {
        let mut tree = json!({
            "shapes": [
                {
                    "type": "rectangle",
                    "borderTopWeight": 1.0,
                    "borderBottomWeight": 1.0
                },
                {
                    "type": "ellipse",
                    "borderLeftWeight": 2.0,
                    "borderRightWeight": 2.0
                },
                {
                    "type": "line",
                    "borderTopWeight": 0.5
                }
            ]
        });

        remove_border_weights(&mut tree).unwrap();

        // 所有数组元素中的所有边框权重应被删除
        assert!(tree["shapes"][0].get("borderTopWeight").is_none());
        assert!(tree["shapes"][0].get("borderBottomWeight").is_none());
        assert_eq!(tree["shapes"][0]["type"].as_str(), Some("rectangle"));

        assert!(tree["shapes"][1].get("borderLeftWeight").is_none());
        assert!(tree["shapes"][1].get("borderRightWeight").is_none());
        assert_eq!(tree["shapes"][1]["type"].as_str(), Some("ellipse"));

        assert!(tree["shapes"][2].get("borderTopWeight").is_none());
        assert_eq!(tree["shapes"][2]["type"].as_str(), Some("line"));
    }

    #[test]
    fn test_zero_border_weights() {
        let mut tree = json!({
            "name": "Shape",
            "borderTopWeight": 0.0,
            "borderBottomWeight": 0.0,
            "borderLeftWeight": 0.0,
            "borderRightWeight": 0.0
        });

        remove_border_weights(&mut tree).unwrap();

        // 即使是零值边界权重也应该被删除
        assert!(tree.get("borderTopWeight").is_none());
        assert!(tree.get("borderBottomWeight").is_none());
        assert!(tree.get("borderLeftWeight").is_none());
        assert!(tree.get("borderRightWeight").is_none());
    }

    #[test]
    fn test_remove_border_stroke_weights_independent() {
        let mut tree = json!({
            "name": "Rectangle",
            "borderStrokeWeightsIndependent": true,
            "borderTopWeight": 1.0,
            "borderBottomWeight": 2.0,
            "visible": true
        });

        remove_border_weights(&mut tree).unwrap();

        assert!(tree.get("borderStrokeWeightsIndependent").is_none());
        assert!(tree.get("borderTopWeight").is_none());
        assert!(tree.get("borderBottomWeight").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Rectangle"));
        assert_eq!(tree.get("visible").unwrap().as_bool(), Some(true));
    }

    #[test]
    fn test_remove_border_stroke_weights_independent_nested() {
        let mut tree = json!({
            "children": [
                {
                    "name": "Child1",
                    "borderStrokeWeightsIndependent": true,
                    "borderTopWeight": 1.0
                },
                {
                    "name": "Child2",
                    "borderStrokeWeightsIndependent": false
                }
            ]
        });

        remove_border_weights(&mut tree).unwrap();

        assert!(tree["children"][0].get("borderStrokeWeightsIndependent").is_none());
        assert!(tree["children"][0].get("borderTopWeight").is_none());
        assert_eq!(tree["children"][0]["name"].as_str(), Some("Child1"));

        assert!(tree["children"][1].get("borderStrokeWeightsIndependent").is_none());
        assert_eq!(tree["children"][1]["name"].as_str(), Some("Child2"));
    }
}
