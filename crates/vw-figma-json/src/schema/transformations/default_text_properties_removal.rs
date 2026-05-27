use crate::error::Result;
use serde_json::Value as JsonValue;

/// 从 JSON 树中的所有对象中删除默认文本属性值
///
/// 递归遍历 JSON 树并删除具有以下属性的文本属性
/// 减少 JSON 大小的默认值：
/// - "letterSpacing"，值为 0，单位为 "PERCENT"(默认)
/// - "lineHeight"，值为 100，单位为 "PERCENT"(默认值，相当于 1.0)
///
/// 这些是 Figma 和 CSS 中的默认值，因此省略它们会减少
/// 输出大小而不丢失信息。
///
/// # 参数
/// * `tree` - 要修改的 JSON 树(通常是文档根)
///
/// # 返回值
/// * `Ok(())` - 成功删除所有默认文本属性字段
///
/// # 示例
/// ```no_run
/// use fig2json::schema::remove_default_text_properties;
/// use serde_json::json;
///
/// let mut tree = json!({
///     "name": "Text",
///     "letterSpacing": {"units": "PERCENT", "value": 0.0},
///     "lineHeight": {"units": "PERCENT", "value": 100.0},
///     "fontSize": 16.0
/// });
/// remove_default_text_properties(&mut tree).unwrap();
/// // 树现在只有 "name" 和 "fontSize" 字段
/// ```
pub fn remove_default_text_properties(tree: &mut JsonValue) -> Result<()> {
    transform_recursive(tree)
}

/// 从 JSON 值中递归删除默认文本属性
fn transform_recursive(value: &mut JsonValue) -> Result<()> {
    match value {
        JsonValue::Object(map) => {
            // 检查并删除 letterSpacing(如果是默认值)(0 PERCENT)
            if let Some(letter_spacing) = map.get("letterSpacing")
                && is_default_letter_spacing(letter_spacing)
            {
                map.remove("letterSpacing");
            }

            // 检查并删除 lineHeight(如果是默认值(100 PERCENT))
            if let Some(line_height) = map.get("lineHeight")
                && is_default_line_height(line_height)
            {
                map.remove("lineHeight");
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

/// 检查 letterSpacing 是否有默认值 (0 PERCENT)
fn is_default_letter_spacing(value: &JsonValue) -> bool {
    if let Some(obj) = value.as_object() {
        let has_percent_units =
            obj.get("units").and_then(|v| v.as_str()).map(|s| s == "PERCENT").unwrap_or(false);

        let has_zero_value =
            obj.get("value").and_then(|v| v.as_f64()).map(|f| f.abs() < 1e-10).unwrap_or(false);

        has_percent_units && has_zero_value
    } else {
        false
    }
}

/// 检查 lineHeight 是否有默认值 (100 PERCENT)
fn is_default_line_height(value: &JsonValue) -> bool {
    if let Some(obj) = value.as_object() {
        let has_percent_units =
            obj.get("units").and_then(|v| v.as_str()).map(|s| s == "PERCENT").unwrap_or(false);

        let has_hundred_value = obj
            .get("value")
            .and_then(|v| v.as_f64())
            .map(|f| (f - 100.0).abs() < 1e-10)
            .unwrap_or(false);

        has_percent_units && has_hundred_value
    } else {
        false
    }
}

#[cfg(test)]
#[path = "default_text_properties_removal_tests.rs"]
mod default_text_properties_removal_tests;
