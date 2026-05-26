use crate::error::Result;
use serde_json::Value as JsonValue;

/// 从衍生文本数据对象中删除layoutSize字段
///
/// 递归遍历JSON树并移除其中的 "layoutSize" 字段
/// "derivedTextData" 对象。 layoutSize 是多余的，因为它通常
/// 与节点的 "size" 字段匹配，因此删除它会减少 JSON 大小而不用
/// 丢失信息。
///
/// # 参数
/// * `tree` - 要修改的 JSON 树(通常是文档根)
///
/// # 返回值
/// * `Ok(())` - 成功从衍生文本数据中删除所有布局大小字段
///
/// # 示例
/// ```no_run
/// use fig2json::schema::remove_derived_text_layout_size;
/// use serde_json::json;
///
/// let mut tree = json!({
///     "derivedTextData": {
///         "layoutSize": {"x": 100.0, "y": 50.0},
///         "otherInfo": "preserved"
///     },
///     "size": {"x": 100.0, "y": 50.0}
/// });
/// remove_derived_text_layout_size(&mut tree).unwrap();
/// // derivedTextData 现在只有 "otherInfo"
/// ```
pub fn remove_derived_text_layout_size(tree: &mut JsonValue) -> Result<()> {
    transform_recursive(tree)
}

/// 从衍生文本数据对象中递归删除layoutSize
fn transform_recursive(value: &mut JsonValue) -> Result<()> {
    match value {
        JsonValue::Object(map) => {
            // 检查该对象是否有 "derivedTextData" 字段
            let keys: Vec<String> = map.keys().cloned().collect();

            for key in keys {
                if key == "derivedTextData" {
                    // 这可能是具有layoutSize 的衍生文本数据对象
                    if let Some(derived_text_data) = map.get_mut(&key)
                        && let Some(data_obj) = derived_text_data.as_object_mut()
                    {
                        // 删除 layoutSize 字段
                        data_obj.remove("layoutSize");
                    }
                }

                // 递归到该值，不管
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
    fn test_remove_layout_size() {
        let mut tree = json!({
            "name": "Text",
            "derivedTextData": {
                "layoutSize": {"x": 100.0, "y": 50.0},
                "otherInfo": "test"
            },
            "size": {"x": 100.0, "y": 50.0}
        });

        remove_derived_text_layout_size(&mut tree).unwrap();

        let derived_text_data = tree.get("derivedTextData").unwrap();
        assert!(derived_text_data.get("layoutSize").is_none());
        assert_eq!(derived_text_data.get("otherInfo").unwrap().as_str(), Some("test"));
        // 大小字段
        assert!(tree.get("size").is_some());
    }

    #[test]
    fn test_preserve_other_fields() {
        let mut tree = json!({
            "name": "Text",
            "derivedTextData": {
                "layoutSize": {"x": 200.0, "y": 100.0},
                "fontFamily": "Arial",
                "fontSize": 16.0
            }
        });

        remove_derived_text_layout_size(&mut tree).unwrap();

        let derived_text_data = tree.get("derivedTextData").unwrap();
        assert!(derived_text_data.get("layoutSize").is_none());
        assert_eq!(derived_text_data.get("fontFamily").unwrap().as_str(), Some("Arial"));
        assert_eq!(derived_text_data.get("fontSize").unwrap().as_f64(), Some(16.0));
    }

    #[test]
    fn test_no_layout_size() {
        let mut tree = json!({
            "name": "Text",
            "derivedTextData": {
                "fontFamily": "Helvetica",
                "fontSize": 14.0
            }
        });

        remove_derived_text_layout_size(&mut tree).unwrap();

        let derived_text_data = tree.get("derivedTextData").unwrap();
        // 没有layoutSize的衍生文本数据应该保持不变
        assert!(derived_text_data.get("layoutSize").is_none());
        assert_eq!(derived_text_data.get("fontFamily").unwrap().as_str(), Some("Helvetica"));
    }

    #[test]
    fn test_no_derived_text_data() {
        let mut tree = json!({
            "name": "Rectangle",
            "width": 100,
            "height": 200
        });

        remove_derived_text_layout_size(&mut tree).unwrap();

        // 没有衍生文本数据的树应该保持不变
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Rectangle"));
        assert_eq!(tree.get("width").unwrap().as_i64(), Some(100));
        assert!(tree.get("derivedTextData").is_none());
    }

    #[test]
    fn test_nested_objects() {
        let mut tree = json!({
            "children": [
                {
                    "name": "Text1",
                    "derivedTextData": {
                        "layoutSize": {"x": 50.0, "y": 25.0},
                        "info1": "data1"
                    }
                },
                {
                    "name": "Text2",
                    "derivedTextData": {
                        "layoutSize": {"x": 60.0, "y": 30.0},
                        "info2": "data2"
                    }
                }
            ]
        });

        remove_derived_text_layout_size(&mut tree).unwrap();

        // 两个layoutSize字段都应该被删除
        assert!(tree["children"][0]["derivedTextData"].get("layoutSize").is_none());
        assert_eq!(tree["children"][0]["derivedTextData"]["info1"].as_str(), Some("data1"));

        assert!(tree["children"][1]["derivedTextData"].get("layoutSize").is_none());
        assert_eq!(tree["children"][1]["derivedTextData"]["info2"].as_str(), Some("data2"));
    }

    #[test]
    fn test_deeply_nested() {
        let mut tree = json!({
            "document": {
                "children": [
                    {
                        "type": "TEXT",
                        "derivedTextData": {
                            "layoutSize": {"x": 300.0, "y": 150.0},
                            "characters": "Hello"
                        }
                    }
                ]
            }
        });

        remove_derived_text_layout_size(&mut tree).unwrap();

        let derived_text_data = &tree["document"]["children"][0]["derivedTextData"];
        assert!(derived_text_data.get("layoutSize").is_none());
        assert_eq!(derived_text_data["characters"].as_str(), Some("Hello"));
    }

    #[test]
    fn test_empty_derived_text_data() {
        let mut tree = json!({
            "name": "Text",
            "derivedTextData": {}
        });

        remove_derived_text_layout_size(&mut tree).unwrap();

        let derived_text_data = tree.get("derivedTextData").unwrap();
        // 空的 derivedTextData 应保持为空
        assert!(derived_text_data.as_object().unwrap().is_empty());
    }

    #[test]
    fn test_layout_size_outside_derived_text_data() {
        let mut tree = json!({
            "name": "Node",
            "layoutSize": {"x": 100.0, "y": 100.0},
            "derivedTextData": {
                "layoutSize": {"x": 50.0, "y": 50.0}
            }
        });

        remove_derived_text_layout_size(&mut tree).unwrap();

        // derivedTextData 之外的 layoutSize 应保留
        assert!(tree.get("layoutSize").is_some());
        assert_eq!(tree["layoutSize"]["x"].as_f64(), Some(100.0));

        // derivedTextData 内的 layoutSize 应删除
        assert!(tree["derivedTextData"].get("layoutSize").is_none());
    }
}
