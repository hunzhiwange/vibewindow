use crate::error::Result;
use serde_json::Value as JsonValue;

/// 将冗长的文本属性结构简化为 CSS 就绪字符串。
///
/// 此转换将 letterSpacing 和 lineHeight 从 verbose 转换
/// Figma 格式 `{"units": "PERCENT", "value": X}` 或 `{"units": "PIXELS", "value": Y}`
/// 到简单的 CSS 就绪字符串，如 "-1%" 或 "20px"。
///
/// 这使得 JSON 更具可读性并且更接近 CSS 表示，而
/// 从 Figma 格式中删除不必要的冗长内容。
///
/// # 应用转换：
/// - `{"units": "PERCENT", "value": -1.0}` → `"-1%"`
/// - `{"units": "PIXELS", "value": 20.0}` → `"20px"`
/// - 应用于 `letterSpacing` 和 `lineHeight` 属性
///
/// # 示例
///
/// ```rust
/// use serde_json::json;
/// use fig2json::schema::simplify_text_properties;
///
/// let mut tree = json!({
///     "name": "Text",
///     "fontSize": 14.0,
///     "letterSpacing": {
///         "units": "PERCENT",
///         "value": -1.0
///     },
///     "lineHeight": {
///         "units": "PIXELS",
///         "value": 20.0
///     }
/// });
///
/// simplify_text_properties(&mut tree).unwrap();
///
/// assert_eq!(tree.get("letterSpacing").unwrap().as_str(), Some("-1%"));
/// assert_eq!(tree.get("lineHeight").unwrap().as_str(), Some("20px"));
/// ```
pub fn simplify_text_properties(tree: &mut JsonValue) -> Result<()> {
    transform_recursive(tree)
}

fn transform_recursive(value: &mut JsonValue) -> Result<()> {
    match value {
        JsonValue::Object(map) => {
            // 收集键以避免借用检查器问题
            let keys: Vec<String> = map.keys().cloned().collect();

            for key in keys {
                // 检查这是否是 letterSpacing 或 lineHeight 属性
                if (key == "letterSpacing" || key == "lineHeight")
                    && let Some(val) = map.get(&key)
                {
                    // 检查该值是否是单位/值对象
                    if let Some(obj) = val.as_object()
                        && is_text_property_object(obj)
                    {
                        // 转换为 CSS 字符串
                        if let Some(css_value) = convert_to_css_string(obj) {
                            map.insert(key.clone(), JsonValue::String(css_value));
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

/// 检查对象是否是具有单位和值的文本属性对象
fn is_text_property_object(obj: &serde_json::Map<String, JsonValue>) -> bool {
    obj.contains_key("units") && obj.contains_key("value")
}

/// 将文本属性对象转换为 CSS 字符串
///
/// 将 Figma 的详细格式转换为 CSS 就绪字符串：
/// - 百分比单位：将 "%" 附加到值
/// - 像素单位：将 "px" 附加到值
///
/// # 参数
/// * `obj` - 具有单位和值字段的文本属性对象
///
/// # 返回值
/// * `Some(String)` - CSS 就绪字符串(例如 "-1%" 或 "20px")
/// * `None` - 如果单位或值丢失/无效
fn convert_to_css_string(obj: &serde_json::Map<String, JsonValue>) -> Option<String> {
    // 提取单位和值
    let units = obj.get("units")?.as_str()?;
    let value = obj.get("value")?.as_f64()?;

    // 根据单位类型转换
    match units {
        "PERCENT" => {
            // 格式为百分比
            // 如果值为整数，则删除不必要的小数位
            if value.fract() == 0.0 {
                Some(format!("{}%", value as i64))
            } else {
                Some(format!("{}%", value))
            }
        }
        "PIXELS" => {
            // 格式为像素
            // 如果值为整数，则删除不必要的小数位
            if value.fract() == 0.0 {
                Some(format!("{}px", value as i64))
            } else {
                Some(format!("{}px", value))
            }
        }
        _ => None, // Unknown unit type
    }
}

#[cfg(test)]
#[path = "text_properties_simplification_tests.rs"]
mod text_properties_simplification_tests;
