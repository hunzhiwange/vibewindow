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
#[path = "enum_simplification_tests.rs"]
mod enum_simplification_tests;
