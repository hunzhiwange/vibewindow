use crate::error::Result;
use serde_json::Value as JsonValue;

/// 当衍生文本数据字段为空对象时删除它
///
/// 递归遍历 JSON 树并删除 "derivedTextData" 字段
/// 是空对象 ({})。空的derivedTextData不提供任何有用的信息
/// 用于 HTML/CSS 渲染，因此删除它会减少 JSON 大小。
///
/// 非空的derivedTextData对象被保留，以防它们包含有用的数据。
///
/// # 参数
/// * `tree` - 要修改的 JSON 树(通常是文档根)
///
/// # 返回值
/// * `Ok(())` - 成功删除所有空的derivedTextData字段
///
/// # 示例
/// ```no_run
/// use fig2json::schema::remove_empty_derived_text_data;
/// use serde_json::json;
///
/// let mut tree = json!({
///     "name": "Text",
///     "derivedTextData": {},
///     "fontSize": 16.0
/// });
/// remove_empty_derived_text_data(&mut tree).unwrap();
/// // 树现在只有 "name" 和 "fontSize" 字段
/// ```
pub fn remove_empty_derived_text_data(tree: &mut JsonValue) -> Result<()> {
    transform_recursive(tree)
}

/// 从 JSON 值中递归删除空的derivedTextData字段
fn transform_recursive(value: &mut JsonValue) -> Result<()> {
    match value {
        JsonValue::Object(map) => {
            // 检查衍生文本数据是否存在并且是一个空对象
            if let Some(derived_text_data) = map.get("derivedTextData")
                && let Some(obj) = derived_text_data.as_object()
                && obj.is_empty()
            {
                map.remove("derivedTextData");
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
    fn test_remove_empty_derived_text_data() {
        let mut tree = json!({
            "name": "Text",
            "derivedTextData": {},
            "fontSize": 16.0
        });

        remove_empty_derived_text_data(&mut tree).unwrap();

        assert!(tree.get("derivedTextData").is_none());
        assert_eq!(tree.get("name").unwrap().as_str(), Some("Text"));
        assert_eq!(tree.get("fontSize").unwrap().as_f64(), Some(16.0));
    }

    #[test]
    fn test_preserve_non_empty_derived_text_data() {
        let mut tree = json!({
            "name": "Text",
            "derivedTextData": {
                "fontFamily": "Arial",
                "fontSize": 12.0
            }
        });

        remove_empty_derived_text_data(&mut tree).unwrap();

        // 应保留非空的derivedTextData
        assert!(tree.get("derivedTextData").is_some());
        let derived = tree.get("derivedTextData").unwrap();
        assert_eq!(derived.get("fontFamily").unwrap().as_str(), Some("Arial"));
        assert_eq!(derived.get("fontSize").unwrap().as_f64(), Some(12.0));
    }

    #[test]
    fn test_preserve_derived_text_data_with_one_field() {
        let mut tree = json!({
            "name": "Text",
            "derivedTextData": {
                "characters": "Hello"
            }
        });

        remove_empty_derived_text_data(&mut tree).unwrap();

        // 衍生文本数据，即使只有一个字段也应该被保留
        assert!(tree.get("derivedTextData").is_some());
        assert_eq!(tree["derivedTextData"]["characters"].as_str(), Some("Hello"));
    }

    #[test]
    fn test_no_derived_text_data() {
        let mut tree = json!({
            "name": "Rectangle",
            "width": 100,
            "height": 200
        });

        remove_empty_derived_text_data(&mut tree).unwrap();

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
                    "derivedTextData": {}
                },
                {
                    "name": "Text2",
                    "derivedTextData": {
                        "info": "data"
                    }
                }
            ]
        });

        remove_empty_derived_text_data(&mut tree).unwrap();

        // 删除了空的derivedTextData
        assert!(tree["children"][0].get("derivedTextData").is_none());
        assert_eq!(tree["children"][0]["name"].as_str(), Some("Text1"));

        // 保留非空的derivedTextData
        assert!(tree["children"][1].get("derivedTextData").is_some());
        assert_eq!(tree["children"][1]["derivedTextData"]["info"].as_str(), Some("data"));
    }

    #[test]
    fn test_deeply_nested() {
        let mut tree = json!({
            "document": {
                "children": [
                    {
                        "type": "TEXT",
                        "derivedTextData": {},
                        "name": "Text"
                    }
                ]
            }
        });

        remove_empty_derived_text_data(&mut tree).unwrap();

        let text_node = &tree["document"]["children"][0];
        assert!(text_node.get("derivedTextData").is_none());
        assert_eq!(text_node["type"].as_str(), Some("TEXT"));
        assert_eq!(text_node["name"].as_str(), Some("Text"));
    }

    #[test]
    fn test_multiple_empty_derived_text_data() {
        let mut tree = json!({
            "children": [
                {"derivedTextData": {}, "name": "A"},
                {"derivedTextData": {}, "name": "B"},
                {"derivedTextData": {}, "name": "C"}
            ]
        });

        remove_empty_derived_text_data(&mut tree).unwrap();

        // 所有空的derivedTextData应被删除
        assert!(tree["children"][0].get("derivedTextData").is_none());
        assert!(tree["children"][1].get("derivedTextData").is_none());
        assert!(tree["children"][2].get("derivedTextData").is_none());
        assert_eq!(tree["children"][0]["name"].as_str(), Some("A"));
        assert_eq!(tree["children"][1]["name"].as_str(), Some("B"));
        assert_eq!(tree["children"][2]["name"].as_str(), Some("C"));
    }

    #[test]
    fn test_derived_text_data_in_arrays() {
        let mut tree = json!({
            "textNodes": [
                {
                    "derivedTextData": {}
                },
                {
                    "derivedTextData": {
                        "layoutSize": {"x": 100.0, "y": 50.0}
                    }
                }
            ]
        });

        remove_empty_derived_text_data(&mut tree).unwrap();

        assert!(tree["textNodes"][0].get("derivedTextData").is_none());
        assert!(tree["textNodes"][1].get("derivedTextData").is_some());
    }

    #[test]
    fn test_derived_text_data_not_object() {
        let mut tree = json!({
            "name": "Test",
            "derivedTextData": "not an object"
        });

        remove_empty_derived_text_data(&mut tree).unwrap();

        // 应保留非对象派生文本数据
        assert_eq!(tree.get("derivedTextData").unwrap().as_str(), Some("not an object"));
    }

    #[test]
    fn test_preserve_other_empty_objects() {
        let mut tree = json!({
            "name": "Test",
            "derivedTextData": {},
            "otherEmptyObject": {},
            "metadata": {}
        });

        remove_empty_derived_text_data(&mut tree).unwrap();

        // 仅应删除衍生文本数据
        assert!(tree.get("derivedTextData").is_none());
        assert!(tree.get("otherEmptyObject").is_some());
        assert!(tree.get("metadata").is_some());
    }
}
