use crate::error::Result;
use serde_json::Value as JsonValue;

/// 从文档树中删除空的 fillPaints 和 strokePaints 数组。
///
/// 此转换会删除 `fillPaints` 和 `strokePaints` 字段
/// 包含空数组。隐形涂料被清除后，可能会出现空涂料阵列。
/// 被过滤掉，或者当节点明确没有填充或描边时。
///
/// 删除空绘制数组会导致 HTML/CSS 转换的 JSON 输出更清晰，
/// 因为缺少该字段在语义上等同于空数组(无填充/笔画)。
///
/// 此转换通常应在 `invisible_paints_removal` 之后运行以进行清理
/// 过滤产生的空数组。
///
/// # 示例
///
/// ```rust
/// use serde_json::json;
/// use fig2json::schema::remove_empty_paint_arrays;
///
/// let mut tree = json!({
///     "name": "Rectangle",
///     "fillPaints": [],
///     "strokePaints": [],
///     "size": {"x": 100.0, "y": 100.0}
/// });
///
/// remove_empty_paint_arrays(&mut tree).unwrap();
///
/// assert!(tree.get("fillPaints").is_none());
/// assert!(tree.get("strokePaints").is_none());
/// assert!(tree.get("size").is_some());
/// ```
pub fn remove_empty_paint_arrays(tree: &mut JsonValue) -> Result<()> {
    transform_recursive(tree)
}

fn transform_recursive(value: &mut JsonValue) -> Result<()> {
    match value {
        JsonValue::Object(map) => {
            // 检查并删除空的 fillPaints 数组
            if let Some(JsonValue::Array(paints)) = map.get("fillPaints")
                && paints.is_empty()
            {
                map.remove("fillPaints");
            }

            // 检查并删除空的 strokePaints 数组
            if let Some(JsonValue::Array(paints)) = map.get("strokePaints")
                && paints.is_empty()
            {
                map.remove("strokePaints");
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
    fn test_removes_empty_fill_paints() {
        let mut tree = json!({
            "name": "Rectangle",
            "fillPaints": [],
            "size": {"x": 100.0, "y": 100.0}
        });

        remove_empty_paint_arrays(&mut tree).unwrap();

        assert!(tree.get("fillPaints").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Rectangle"));
        assert!(tree.get("size").is_some());
    }

    #[test]
    fn test_removes_empty_stroke_paints() {
        let mut tree = json!({
            "name": "Rectangle",
            "strokePaints": [],
            "size": {"x": 100.0, "y": 100.0}
        });

        remove_empty_paint_arrays(&mut tree).unwrap();

        assert!(tree.get("strokePaints").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Rectangle"));
        assert!(tree.get("size").is_some());
    }

    #[test]
    fn test_removes_both_empty_paint_arrays() {
        let mut tree = json!({
            "name": "Rectangle",
            "fillPaints": [],
            "strokePaints": []
        });

        remove_empty_paint_arrays(&mut tree).unwrap();

        assert!(tree.get("fillPaints").is_none());
        assert!(tree.get("strokePaints").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Rectangle"));
    }

    #[test]
    fn test_preserves_non_empty_fill_paints() {
        let mut tree = json!({
            "name": "Rectangle",
            "fillPaints": [
                {
                    "color": "#ffffff",
                    "type": "SOLID"
                }
            ]
        });

        remove_empty_paint_arrays(&mut tree).unwrap();

        assert!(tree.get("fillPaints").is_some());
        let fills = tree.get("fillPaints").unwrap().as_array().unwrap();
        assert_eq!(fills.len(), 1);
        assert_eq!(fills[0].get("color").unwrap().as_str(), Some("#ffffff"));
    }

    #[test]
    fn test_preserves_non_empty_stroke_paints() {
        let mut tree = json!({
            "name": "Rectangle",
            "strokePaints": [
                {
                    "color": "#000000",
                    "type": "SOLID"
                }
            ]
        });

        remove_empty_paint_arrays(&mut tree).unwrap();

        assert!(tree.get("strokePaints").is_some());
        let strokes = tree.get("strokePaints").unwrap().as_array().unwrap();
        assert_eq!(strokes.len(), 1);
        assert_eq!(strokes[0].get("color").unwrap().as_str(), Some("#000000"));
    }

    #[test]
    fn test_handles_missing_paint_arrays() {
        let mut tree = json!({
            "name": "Rectangle",
            "size": {"x": 100.0, "y": 100.0}
        });

        remove_empty_paint_arrays(&mut tree).unwrap();

        assert!(tree.get("fillPaints").is_none());
        assert!(tree.get("strokePaints").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Rectangle"));
    }

    #[test]
    fn test_handles_nested_objects() {
        let mut tree = json!({
            "name": "Parent",
            "children": [
                {
                    "name": "Child1",
                    "fillPaints": []
                },
                {
                    "name": "Child2",
                    "strokePaints": []
                }
            ]
        });

        remove_empty_paint_arrays(&mut tree).unwrap();

        let children = tree.get("children").unwrap().as_array().unwrap();
        assert!(children[0].get("fillPaints").is_none());
        assert!(children[1].get("strokePaints").is_none());
        assert_eq!(children[0].get("name").unwrap().as_str(), Some("Child1"));
        assert_eq!(children[1].get("name").unwrap().as_str(), Some("Child2"));
    }

    #[test]
    fn test_handles_deeply_nested_structures() {
        let mut tree = json!({
            "name": "Root",
            "fillPaints": [],
            "children": [
                {
                    "name": "Level1",
                    "strokePaints": [],
                    "children": [
                        {
                            "name": "Level2",
                            "fillPaints": [],
                            "strokePaints": []
                        }
                    ]
                }
            ]
        });

        remove_empty_paint_arrays(&mut tree).unwrap();

        assert!(tree.get("fillPaints").is_none());
        let level1 = &tree.get("children").unwrap().as_array().unwrap()[0];
        assert!(level1.get("strokePaints").is_none());
        let level2 = &level1.get("children").unwrap().as_array().unwrap()[0];
        assert!(level2.get("fillPaints").is_none());
        assert!(level2.get("strokePaints").is_none());
        assert_eq!(level2.get("name").unwrap().as_str(), Some("Level2"));
    }

    #[test]
    fn test_handles_empty_object() {
        let mut tree = json!({});

        remove_empty_paint_arrays(&mut tree).unwrap();

        assert_eq!(tree.as_object().unwrap().len(), 0);
    }

    #[test]
    fn test_preserves_other_fields() {
        let mut tree = json!({
            "name": "icon/ai",
            "type": "INSTANCE",
            "fillPaints": [],
            "strokePaints": [],
            "size": {"x": 20.0, "y": 20.0},
            "transform": {"x": 0.0, "y": 9.0}
        });

        remove_empty_paint_arrays(&mut tree).unwrap();

        assert!(tree.get("fillPaints").is_none());
        assert!(tree.get("strokePaints").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("icon/ai"));
        assert_eq!(tree.get("type").unwrap().as_str(), Some("INSTANCE"));
        assert!(tree.get("size").is_some());
        assert!(tree.get("transform").is_some());
    }

    #[test]
    fn test_mixed_empty_and_non_empty() {
        let mut tree = json!({
            "children": [
                {
                    "name": "EmptyFills",
                    "fillPaints": [],
                    "strokePaints": [{"color": "#000", "type": "SOLID"}]
                },
                {
                    "name": "EmptyStrokes",
                    "fillPaints": [{"color": "#fff", "type": "SOLID"}],
                    "strokePaints": []
                },
                {
                    "name": "BothEmpty",
                    "fillPaints": [],
                    "strokePaints": []
                },
                {
                    "name": "NeitherEmpty",
                    "fillPaints": [{"color": "#f00", "type": "SOLID"}],
                    "strokePaints": [{"color": "#0f0", "type": "SOLID"}]
                }
            ]
        });

        remove_empty_paint_arrays(&mut tree).unwrap();

        let children = tree.get("children").unwrap().as_array().unwrap();

        // 子级 0：删除空填充，保留笔划
        assert!(children[0].get("fillPaints").is_none());
        assert!(children[0].get("strokePaints").is_some());

        // 子 1：保留填充，删除空笔划
        assert!(children[1].get("fillPaints").is_some());
        assert!(children[1].get("strokePaints").is_none());

        // 子 2：两个空数组均已删除
        assert!(children[2].get("fillPaints").is_none());
        assert!(children[2].get("strokePaints").is_none());

        // 子 3：均保留
        assert!(children[3].get("fillPaints").is_some());
        assert!(children[3].get("strokePaints").is_some());
    }
}
