use crate::error::Result;
use serde_json::Value as JsonValue;

/// 从数组中删除独立的 overridedenSymbolID 对象
///
/// 递归遍历JSON树，过滤掉包含以下内容的对象
/// 仅一个 `overriddenSymbolID` 字段(其本身仅包含 `localID` 和 `sessionID`)。
///
/// 这些是 Figma 组件交换元数据对象，出现在数组中，例如
/// `symbolOverrides`。它们表明嵌套组件已交换，但没有
/// 提供 HTML/CSS 转换所需的任何视觉呈现信息。
///
/// 具有 `overriddenSymbolID` 以及其他字段的对象(例如 `textData`、
/// `visible` 等)被保留，因为其他字段包含渲染信息。
///
/// # 参数
/// * `tree` - 要修改的 JSON 树(通常是文档根)
///
/// # 返回值
/// * `Ok(())` - 成功删除所有独立的 overridedenSymbolID 对象
///
/// # 示例
/// ```no_run
/// use fig2json::schema::remove_overridden_symbol_id;
/// use serde_json::json;
///
/// let mut tree = json!({
///     "symbolOverrides": [
///         {
///             "overriddenSymbolID": {
///                 "localID": 123,
///                 "sessionID": 456
///             }
///         },
///         {
///             "overriddenSymbolID": {
///                 "localID": 789,
///                 "sessionID": 12
///             },
///             "textData": {
///                 "characters": "Hello"
///             }
///         }
///     ]
/// });
/// remove_overridden_symbol_id(&mut tree).unwrap();
/// // 第一个对象被删除，第二个对象被保留(有文本数据)
/// ```
pub fn remove_overridden_symbol_id(tree: &mut JsonValue) -> Result<()> {
    transform_recursive(tree)
}

/// 检查对象是否仅包含 overridedenSymbolID 字段
/// 仅具有 localID 和 sessionID
fn is_standalone_overridden_symbol_id(obj: &serde_json::Map<String, JsonValue>) -> bool {
    // 必须恰好有 1 个密钥
    if obj.len() != 1 {
        return false;
    }

    // 该密钥必须是 "overriddenSymbolID"
    if let Some(overridden_symbol_id) = obj.get("overriddenSymbolID") {
        // 该值必须是一个对象
        if let Some(inner_obj) = overridden_symbol_id.as_object() {
            // 该对象必须仅包含 "localID" 和 "sessionID"
            if inner_obj.len() != 2 {
                return false;
            }
            return inner_obj.contains_key("localID") && inner_obj.contains_key("sessionID");
        }
    }

    false
}

/// 从 JSON 值中递归删除独立的 overridedenSymbolID 对象
fn transform_recursive(value: &mut JsonValue) -> Result<()> {
    match value {
        JsonValue::Object(map) => {
            // 递归到所有值
            for val in map.values_mut() {
                transform_recursive(val)?;
            }
        }
        JsonValue::Array(arr) => {
            // 过滤掉独立的 overridedenSymbolID 对象
            arr.retain(|element| {
                if let Some(obj) = element.as_object() {
                    // 如果元素不是独立的 overridedenSymbolID，则保留元素
                    !is_standalone_overridden_symbol_id(obj)
                } else {
                    // 保留非对象值
                    true
                }
            });

            // 然后递归到剩余的数组元素
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
#[path = "overridden_symbol_id_removal_tests.rs"]
mod overridden_symbol_id_removal_tests;
