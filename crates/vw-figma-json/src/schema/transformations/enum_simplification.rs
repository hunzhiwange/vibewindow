use crate::error::Result;
use serde_json::Value as JsonValue;

/// 将枚举对象简化为简单的字符串值
///
/// 递归遍历 JSON 树并通过转换来简化枚举对象
/// 详细枚举格式到简单字符串：
/// - 来自：`{"__enum__": "BlendMode", "value": "NORMAL"}`
/// - 收件人：`"NORMAL"`
///
/// 这适用于 Figma 格式的所有枚举类型，包括：
/// 节点类型、混合模式、PaintType、StrokeAlign、StrokeJoin、NodePhase、
/// WindingRule、TextAlignVertical、TextAutoResize、LineType、FontStyle、
/// EmojiImageSet、ImageScaleMode、Directionality、DocumentColorProfile 等
///
/// # 参数
/// * `tree` - 要修改的 JSON 树(通常是文档根)
///
/// # 返回值
/// * `Ok(())` - 成功简化所有枚举对象
///
/// # 示例
/// ```no_run
/// use fig2json::schema::simplify_enums;
/// use serde_json::json;
///
/// let mut tree = json!({
///     "type": {
///         "__enum__": "NodeType",
///         "value": "FRAME"
///     },
///     "blendMode": {
///         "__enum__": "BlendMode",
///         "value": "NORMAL"
///     }
/// });
/// simplify_enums(&mut tree).unwrap();
/// // 树现在有 "type": "FRAME" 和 "blendMode": "NORMAL"
/// ```
pub fn simplify_enums(tree: &mut JsonValue) -> Result<()> {
    transform_recursive(tree)
}

/// 递归简化 JSON 值中的枚举对象
fn transform_recursive(value: &mut JsonValue) -> Result<()> {
    match value {
        JsonValue::Object(map) => {
            // 收集键以避免借用检查器问题
            let keys: Vec<String> = map.keys().cloned().collect();

            for key in keys {
                if let Some(val) = map.get(&key) {
                    // 检查该值是否是枚举对象
                    if let Some(obj) = val.as_object()
                        && is_enum_object(obj)
                    {
                        // 提取值并替换枚举对象
                        if let Some(enum_value) = extract_enum_value(obj) {
                            map.insert(key.clone(), JsonValue::String(enum_value));
                            continue; // Skip recursion since we replaced the object
                        }
                    }
                }

                // 如果未替换则递归到该值
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

/// 检查对象是否是枚举对象(具有 __enum__ 和值字段)
fn is_enum_object(obj: &serde_json::Map<String, JsonValue>) -> bool {
    obj.contains_key("__enum__") && obj.contains_key("value")
}

/// 从枚举对象中提取值
///
/// # 参数
/// * `obj` - 具有 __enum__ 和值字段的枚举对象
///
/// # 返回值
/// * `Some(String)` - 枚举值字符串
/// * `None` - 如果值字段不是字符串
fn extract_enum_value(obj: &serde_json::Map<String, JsonValue>) -> Option<String> {
    obj.get("value")?.as_str().map(|s| s.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_simplify_node_type() {
        let mut tree = json!({
            "name": "Frame",
            "type": {
                "__enum__": "NodeType",
                "value": "FRAME"
            }
        });

        simplify_enums(&mut tree).unwrap();

        assert_eq!(tree.get("type").unwrap().as_str(), Some("FRAME"));
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Frame"));
    }

    #[test]
    fn test_simplify_blend_mode() {
        let mut tree = json!({
            "blendMode": {
                "__enum__": "BlendMode",
                "value": "NORMAL"
            }
        });

        simplify_enums(&mut tree).unwrap();

        assert_eq!(tree.get("blendMode").unwrap().as_str(), Some("NORMAL"));
    }

    #[test]
    fn test_simplify_paint_type() {
        let mut tree = json!({
            "type": {
                "__enum__": "PaintType",
                "value": "SOLID"
            }
        });

        simplify_enums(&mut tree).unwrap();

        assert_eq!(tree.get("type").unwrap().as_str(), Some("SOLID"));
    }

    #[test]
    fn test_simplify_multiple_enums() {
        let mut tree = json!({
            "type": {
                "__enum__": "NodeType",
                "value": "ROUNDED_RECTANGLE"
            },
            "blendMode": {
                "__enum__": "BlendMode",
                "value": "NORMAL"
            },
            "strokeAlign": {
                "__enum__": "StrokeAlign",
                "value": "INSIDE"
            }
        });

        simplify_enums(&mut tree).unwrap();

        assert_eq!(tree.get("type").unwrap().as_str(), Some("ROUNDED_RECTANGLE"));
        assert_eq!(tree.get("blendMode").unwrap().as_str(), Some("NORMAL"));
        assert_eq!(tree.get("strokeAlign").unwrap().as_str(), Some("INSIDE"));
    }

    #[test]
    fn test_simplify_nested_enums() {
        let mut tree = json!({
            "name": "Root",
            "type": {
                "__enum__": "NodeType",
                "value": "DOCUMENT"
            },
            "children": [
                {
                    "name": "Child1",
                    "type": {
                        "__enum__": "NodeType",
                        "value": "FRAME"
                    }
                },
                {
                    "name": "Child2",
                    "phase": {
                        "__enum__": "NodePhase",
                        "value": "CREATED"
                    }
                }
            ]
        });

        simplify_enums(&mut tree).unwrap();

        // 根枚举简化
        assert_eq!(tree.get("type").unwrap().as_str(), Some("DOCUMENT"));

        // 儿童枚举简化
        assert_eq!(tree["children"][0]["type"].as_str(), Some("FRAME"));
        assert_eq!(tree["children"][1]["phase"].as_str(), Some("CREATED"));
    }

    #[test]
    fn test_simplify_deeply_nested_enums() {
        let mut tree = json!({
            "document": {
                "children": [
                    {
                        "fillPaints": [
                            {
                                "type": {
                                    "__enum__": "PaintType",
                                    "value": "IMAGE"
                                },
                                "blendMode": {
                                    "__enum__": "BlendMode",
                                    "value": "NORMAL"
                                }
                            }
                        ]
                    }
                ]
            }
        });

        simplify_enums(&mut tree).unwrap();

        // 深度嵌套枚举的简化
        let paint = &tree["document"]["children"][0]["fillPaints"][0];
        assert_eq!(paint["type"].as_str(), Some("IMAGE"));
        assert_eq!(paint["blendMode"].as_str(), Some("NORMAL"));
    }

    #[test]
    fn test_preserve_non_enum_objects() {
        let mut tree = json!({
            "name": "Rectangle",
            "transform": {
                "x": 100.0,
                "y": 200.0,
                "rotation": 0.0
            },
            "type": {
                "__enum__": "NodeType",
                "value": "FRAME"
            }
        });

        simplify_enums(&mut tree).unwrap();

        // 枚举简化
        assert_eq!(tree.get("type").unwrap().as_str(), Some("FRAME"));

        // 保留非枚举对象
        assert_eq!(tree["transform"]["x"].as_f64(), Some(100.0));
        assert_eq!(tree["transform"]["y"].as_f64(), Some(200.0));
        assert_eq!(tree["transform"]["rotation"].as_f64(), Some(0.0));
    }

    #[test]
    fn test_no_enums() {
        let mut tree = json!({
            "name": "Rectangle",
            "width": 100,
            "height": 200,
            "visible": true
        });

        simplify_enums(&mut tree).unwrap();

        // 没有枚举的树应该保持不变
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Rectangle"));
        assert_eq!(tree.get("width").unwrap().as_i64(), Some(100));
        assert_eq!(tree.get("height").unwrap().as_i64(), Some(200));
        assert_eq!(tree.get("visible").unwrap().as_bool(), Some(true));
    }

    #[test]
    fn test_different_enum_types() {
        let mut tree = json!({
            "textAlignVertical": {
                "__enum__": "TextAlignVertical",
                "value": "TOP"
            },
            "textAutoResize": {
                "__enum__": "TextAutoResize",
                "value": "WIDTH_AND_HEIGHT"
            },
            "lineType": {
                "__enum__": "LineType",
                "value": "PLAIN"
            },
            "fontStyle": {
                "__enum__": "FontStyle",
                "value": "NORMAL"
            }
        });

        simplify_enums(&mut tree).unwrap();

        // 所有枚举类型均已简化
        assert_eq!(tree.get("textAlignVertical").unwrap().as_str(), Some("TOP"));
        assert_eq!(tree.get("textAutoResize").unwrap().as_str(), Some("WIDTH_AND_HEIGHT"));
        assert_eq!(tree.get("lineType").unwrap().as_str(), Some("PLAIN"));
        assert_eq!(tree.get("fontStyle").unwrap().as_str(), Some("NORMAL"));
    }

    #[test]
    fn test_enum_in_array() {
        let mut tree = json!({
            "paints": [
                {
                    "type": {
                        "__enum__": "PaintType",
                        "value": "SOLID"
                    }
                },
                {
                    "type": {
                        "__enum__": "PaintType",
                        "value": "IMAGE"
                    }
                }
            ]
        });

        simplify_enums(&mut tree).unwrap();

        // 简化数组中的所有枚举
        assert_eq!(tree["paints"][0]["type"].as_str(), Some("SOLID"));
        assert_eq!(tree["paints"][1]["type"].as_str(), Some("IMAGE"));
    }

    #[test]
    fn test_is_enum_object() {
        let enum_obj = serde_json::from_value::<serde_json::Map<String, JsonValue>>(json!({
            "__enum__": "BlendMode",
            "value": "NORMAL"
        }))
        .unwrap();
        assert!(is_enum_object(&enum_obj));

        let not_enum = serde_json::from_value::<serde_json::Map<String, JsonValue>>(json!({
            "x": 10,
            "y": 20
        }))
        .unwrap();
        assert!(!is_enum_object(&not_enum));

        let incomplete = serde_json::from_value::<serde_json::Map<String, JsonValue>>(json!({
            "__enum__": "BlendMode"
        }))
        .unwrap();
        assert!(!is_enum_object(&incomplete));
    }

    #[test]
    fn test_extract_enum_value() {
        let enum_obj = serde_json::from_value::<serde_json::Map<String, JsonValue>>(json!({
            "__enum__": "BlendMode",
            "value": "NORMAL"
        }))
        .unwrap();

        let value = extract_enum_value(&enum_obj).unwrap();
        assert_eq!(value, "NORMAL");
    }
}
