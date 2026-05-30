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
#[path = "text_line_defaults_removal_tests.rs"]
mod text_line_defaults_removal_tests;
