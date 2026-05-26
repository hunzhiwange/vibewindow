use crate::error::Result;
use serde_json::Value as JsonValue;

/// 从 JSON 树中的所有对象中删除默认文本行属性
///
/// 递归遍历 JSON 树并删除其中的行属性
/// 具有默认值的 `textData.lines` 和 `textValue.lines` 数组：
/// - "indentationLevel" 值为 0(无缩进)
/// - "isFirstLineOfList" 值为 false(不是列表项)
/// - "lineType"，值为 "PLAIN" (纯文本)
/// - "listStartOffset" 值为 0(无列表偏移量)
/// - "sourceDirectionality"，值为 "AUTO" (自动文本方向)
/// - "styleId" 值为 0(未应用样式)
///
/// 如果 `lines` 数组中的所有线条对象在删除默认值后都变为空，
/// 整个 `lines` 数组被删除。
///
/// 这些是 Figma 中用于纯文本渲染的默认值，因此省略
/// 它们减少了输出大小，而不会丢失 HTML/CSS 转换的信息。
///
/// # 参数
/// * `tree` - 要修改的 JSON 树(通常是文档根)
///
/// # 返回值
/// * `Ok(())` - 成功删除所有默认文本行字段
///
/// # 示例
/// ```no_run
/// use fig2json::schema::remove_default_text_line_properties;
/// use serde_json::json;
///
/// let mut tree = json!({
///     "textData": {
///         "characters": "Hello",
///         "lines": [
///             {
///                 "indentationLevel": 0,
///                 "isFirstLineOfList": false,
///                 "lineType": "PLAIN",
///                 "listStartOffset": 0,
///                 "sourceDirectionality": "AUTO",
///                 "styleId": 0
///             }
///         ]
///     }
/// });
/// remove_default_text_line_properties(&mut tree).unwrap();
/// // 删除整个 "lines" 数组，因为所有值都是默认值
/// ```
pub fn remove_default_text_line_properties(tree: &mut JsonValue) -> Result<()> {
    transform_recursive(tree)
}

/// 从 JSON 值中递归删除默认文本行属性
fn transform_recursive(value: &mut JsonValue) -> Result<()> {
    match value {
        JsonValue::Object(map) => {
            // 检查该对象是否有 "lines" 数组
            if let Some(lines_value) = map.get_mut("lines")
                && let Some(lines_array) = lines_value.as_array_mut()
            {
                // 处理数组中的每个线对象
                for line in lines_array.iter_mut() {
                    if let Some(line_obj) = line.as_object_mut() {
                        remove_default_line_fields(line_obj);
                    }
                }

                // 检查所有行现在是否都是空对象
                // 仅删除lines数组，如果它有元素并且所有元素都是空的
                let all_empty = !lines_array.is_empty()
                    && lines_array
                        .iter()
                        .all(|line| line.as_object().map(|obj| obj.is_empty()).unwrap_or(false));

                // 如果所有行都为空，则删除整个 "lines" 数组
                if all_empty {
                    map.remove("lines");
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

/// 从单行对象中删除默认值字段
fn remove_default_line_fields(line_obj: &mut serde_json::Map<String, JsonValue>) {
    // 如果为 0，则删除 indentationLevel
    if let Some(val) = line_obj.get("indentationLevel")
        && val.as_i64() == Some(0)
    {
        line_obj.remove("indentationLevel");
    }

    // 如果为 false，则删除 isFirstLineOfList
    if let Some(val) = line_obj.get("isFirstLineOfList")
        && val.as_bool() == Some(false)
    {
        line_obj.remove("isFirstLineOfList");
    }

    // 如果 "PLAIN" 则删除 lineType
    if let Some(val) = line_obj.get("lineType")
        && val.as_str() == Some("PLAIN")
    {
        line_obj.remove("lineType");
    }

    // 如果为 0，则删除 listStartOffset
    if let Some(val) = line_obj.get("listStartOffset")
        && val.as_i64() == Some(0)
    {
        line_obj.remove("listStartOffset");
    }

    // 如果 "AUTO" 则删除 sourceDirectionality
    if let Some(val) = line_obj.get("sourceDirectionality")
        && val.as_str() == Some("AUTO")
    {
        line_obj.remove("sourceDirectionality");
    }

    // 如果为 0，则删除 styleId
    if let Some(val) = line_obj.get("styleId")
        && val.as_i64() == Some(0)
    {
        line_obj.remove("styleId");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_remove_all_defaults_removes_lines_array() {
        let mut tree = json!({
            "textData": {
                "characters": "Hello",
                "lines": [
                    {
                        "indentationLevel": 0,
                        "isFirstLineOfList": false,
                        "lineType": "PLAIN",
                        "listStartOffset": 0,
                        "sourceDirectionality": "AUTO",
                        "styleId": 0
                    }
                ]
            }
        });

        remove_default_text_line_properties(&mut tree).unwrap();

        // 整个行数组应该被删除
        assert!(tree["textData"].get("lines").is_none());
        assert_eq!(tree["textData"]["characters"].as_str(), Some("Hello"));
    }

    #[test]
    fn test_preserve_non_default_indentation_level() {
        let mut tree = json!({
            "textData": {
                "characters": "Indented text",
                "lines": [
                    {
                        "indentationLevel": 2,
                        "isFirstLineOfList": false,
                        "lineType": "PLAIN",
                        "listStartOffset": 0,
                        "sourceDirectionality": "AUTO",
                        "styleId": 0
                    }
                ]
            }
        });

        remove_default_text_line_properties(&mut tree).unwrap();

        // 行数组应该仍然存在，因为 indentationLevel 不是默认的
        assert!(tree["textData"].get("lines").is_some());
        let lines = tree["textData"]["lines"].as_array().unwrap();
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0]["indentationLevel"].as_i64(), Some(2));
        // 所有其他默认值应删除
        assert!(lines[0].get("isFirstLineOfList").is_none());
        assert!(lines[0].get("lineType").is_none());
        assert!(lines[0].get("listStartOffset").is_none());
        assert!(lines[0].get("sourceDirectionality").is_none());
        assert!(lines[0].get("styleId").is_none());
    }

    #[test]
    fn test_preserve_list_item() {
        let mut tree = json!({
            "textData": {
                "characters": "• List item",
                "lines": [
                    {
                        "indentationLevel": 1,
                        "isFirstLineOfList": true,
                        "lineType": "UNORDERED_LIST",
                        "listStartOffset": 0,
                        "sourceDirectionality": "AUTO",
                        "styleId": 0
                    }
                ]
            }
        });

        remove_default_text_line_properties(&mut tree).unwrap();

        // 行数组应该仍然存在
        let lines = tree["textData"]["lines"].as_array().unwrap();
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0]["indentationLevel"].as_i64(), Some(1));
        assert_eq!(lines[0]["isFirstLineOfList"].as_bool(), Some(true));
        assert_eq!(lines[0]["lineType"].as_str(), Some("UNORDERED_LIST"));
        // 应删除默认值
        assert!(lines[0].get("listStartOffset").is_none());
        assert!(lines[0].get("sourceDirectionality").is_none());
        assert!(lines[0].get("styleId").is_none());
    }

    #[test]
    fn test_preserve_non_zero_style_id() {
        let mut tree = json!({
            "textData": {
                "characters": "Styled text",
                "lines": [
                    {
                        "indentationLevel": 0,
                        "isFirstLineOfList": false,
                        "lineType": "PLAIN",
                        "listStartOffset": 0,
                        "sourceDirectionality": "AUTO",
                        "styleId": 5
                    }
                ]
            }
        });

        remove_default_text_line_properties(&mut tree).unwrap();

        // Lines 数组应该仍然存在，因为 styleId 不是默认的
        let lines = tree["textData"]["lines"].as_array().unwrap();
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0]["styleId"].as_i64(), Some(5));
        // 所有其他默认值应删除
        assert!(lines[0].get("indentationLevel").is_none());
        assert!(lines[0].get("isFirstLineOfList").is_none());
        assert!(lines[0].get("lineType").is_none());
        assert!(lines[0].get("listStartOffset").is_none());
        assert!(lines[0].get("sourceDirectionality").is_none());
    }

    #[test]
    fn test_multiple_lines_mixed() {
        let mut tree = json!({
            "textData": {
                "characters": "Multi-line text",
                "lines": [
                    {
                        "indentationLevel": 0,
                        "isFirstLineOfList": false,
                        "lineType": "PLAIN",
                        "listStartOffset": 0,
                        "sourceDirectionality": "AUTO",
                        "styleId": 0
                    },
                    {
                        "indentationLevel": 1,
                        "isFirstLineOfList": false,
                        "lineType": "PLAIN",
                        "listStartOffset": 0,
                        "sourceDirectionality": "AUTO",
                        "styleId": 0
                    }
                ]
            }
        });

        remove_default_text_line_properties(&mut tree).unwrap();

        // 行数组应该仍然存在，因为第二行有非默认缩进
        let lines = tree["textData"]["lines"].as_array().unwrap();
        assert_eq!(lines.len(), 2);
        // 第一行应为空(均为默认值)
        assert!(lines[0].as_object().unwrap().is_empty());
        // 第二行应该只有 indentationLevel
        assert_eq!(lines[1]["indentationLevel"].as_i64(), Some(1));
        assert!(lines[1].get("isFirstLineOfList").is_none());
    }

    #[test]
    fn test_nested_text_data() {
        let mut tree = json!({
            "children": [
                {
                    "textData": {
                        "characters": "First",
                        "lines": [
                            {
                                "indentationLevel": 0,
                                "isFirstLineOfList": false,
                                "lineType": "PLAIN",
                                "listStartOffset": 0,
                                "sourceDirectionality": "AUTO",
                                "styleId": 0
                            }
                        ]
                    }
                },
                {
                    "textData": {
                        "characters": "Second",
                        "lines": [
                            {
                                "indentationLevel": 1,
                                "isFirstLineOfList": false,
                                "lineType": "PLAIN",
                                "listStartOffset": 0,
                                "sourceDirectionality": "AUTO",
                                "styleId": 0
                            }
                        ]
                    }
                }
            ]
        });

        remove_default_text_line_properties(&mut tree).unwrap();

        // 第一个子节点应该删除行数组(所有默认值)
        assert!(tree["children"][0]["textData"].get("lines").is_none());

        // 第二个子节点应该仍然有行数组(缩进不是默认的)
        assert!(tree["children"][1]["textData"].get("lines").is_some());
        let lines = tree["children"][1]["textData"]["lines"].as_array().unwrap();
        assert_eq!(lines[0]["indentationLevel"].as_i64(), Some(1));
    }

    #[test]
    fn test_text_value_lines() {
        let mut tree = json!({
            "textValue": {
                "characters": "Hello",
                "lines": [
                    {
                        "indentationLevel": 0,
                        "isFirstLineOfList": false,
                        "lineType": "PLAIN",
                        "listStartOffset": 0,
                        "sourceDirectionality": "AUTO",
                        "styleId": 0
                    }
                ]
            }
        });

        remove_default_text_line_properties(&mut tree).unwrap();

        // 也适用于 textValue.lines
        assert!(tree["textValue"].get("lines").is_none());
    }

    #[test]
    fn test_no_lines_field() {
        let mut tree = json!({
            "textData": {
                "characters": "Hello"
            }
        });

        remove_default_text_line_properties(&mut tree).unwrap();

        // 没有行的树应该保持不变
        assert_eq!(tree["textData"]["characters"].as_str(), Some("Hello"));
    }

    #[test]
    fn test_preserve_non_default_list_start_offset() {
        let mut tree = json!({
            "textData": {
                "characters": "Numbered list",
                "lines": [
                    {
                        "indentationLevel": 0,
                        "isFirstLineOfList": false,
                        "lineType": "PLAIN",
                        "listStartOffset": 5,
                        "sourceDirectionality": "AUTO",
                        "styleId": 0
                    }
                ]
            }
        });

        remove_default_text_line_properties(&mut tree).unwrap();

        // 行数组应该仍然存在
        let lines = tree["textData"]["lines"].as_array().unwrap();
        assert_eq!(lines[0]["listStartOffset"].as_i64(), Some(5));
        // 删除了其他默认值
        assert!(lines[0].get("indentationLevel").is_none());
        assert!(lines[0].get("isFirstLineOfList").is_none());
        assert!(lines[0].get("lineType").is_none());
        assert!(lines[0].get("sourceDirectionality").is_none());
        assert!(lines[0].get("styleId").is_none());
    }

    #[test]
    fn test_deeply_nested_structure() {
        let mut tree = json!({
            "document": {
                "children": [
                    {
                        "type": "TEXT",
                        "textData": {
                            "characters": "Deep text",
                            "lines": [
                                {
                                    "indentationLevel": 0,
                                    "isFirstLineOfList": false,
                                    "lineType": "PLAIN",
                                    "listStartOffset": 0,
                                    "sourceDirectionality": "AUTO",
                                    "styleId": 0
                                }
                            ]
                        }
                    }
                ]
            }
        });

        remove_default_text_line_properties(&mut tree).unwrap();

        let text_data = &tree["document"]["children"][0]["textData"];
        assert!(text_data.get("lines").is_none());
        assert_eq!(text_data["characters"].as_str(), Some("Deep text"));
    }

    #[test]
    fn test_empty_lines_array() {
        let mut tree = json!({
            "textData": {
                "characters": "Hello",
                "lines": []
            }
        });

        remove_default_text_line_properties(&mut tree).unwrap();

        // 应该保留空行数组(不是我们关心的)
        assert!(tree["textData"].get("lines").is_some());
        assert_eq!(tree["textData"]["lines"].as_array().unwrap().len(), 0);
    }

    #[test]
    fn test_multiple_all_default_lines() {
        let mut tree = json!({
            "textData": {
                "characters": "Multi-line",
                "lines": [
                    {
                        "indentationLevel": 0,
                        "isFirstLineOfList": false,
                        "lineType": "PLAIN",
                        "listStartOffset": 0,
                        "sourceDirectionality": "AUTO",
                        "styleId": 0
                    },
                    {
                        "indentationLevel": 0,
                        "isFirstLineOfList": false,
                        "lineType": "PLAIN",
                        "listStartOffset": 0,
                        "sourceDirectionality": "AUTO",
                        "styleId": 0
                    },
                    {
                        "indentationLevel": 0,
                        "isFirstLineOfList": false,
                        "lineType": "PLAIN",
                        "listStartOffset": 0,
                        "sourceDirectionality": "AUTO",
                        "styleId": 0
                    }
                ]
            }
        });

        remove_default_text_line_properties(&mut tree).unwrap();

        // 所有行均为默认行，因此应删除整个行数组
        assert!(tree["textData"].get("lines").is_none());
    }
}
