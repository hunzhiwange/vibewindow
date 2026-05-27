use crate::error::Result;
use serde_json::Value as JsonValue;

/// 删除仅包含 localID 和/或 sessionID 的 symbolID 字段。
///
/// 当 `symbolID` 对象仅包含
/// 字段 `localID` 和/或 `sessionID`。如果 `symbolID` 对象有
/// 除了这两个字段之外的任何其他字段，都会被保留。
///
/// # 示例
///
/// 已删除(仅限标准字段)：
/// ```json
/// {
///   "symbolID": {
///     "localID": 10596,
///     "sessionID": 4331
///   }
/// }
/// ```
///
/// 保留(有额外字段)：
/// ```json
/// {
///   "symbolID": {
///     "localID": 10596,
///     "sessionID": 4331,
///     "customField": "value"
///   }
/// }
/// ```
pub fn remove_symbol_id_fields(tree: &mut JsonValue) -> Result<()> {
    transform_recursive(tree)
}

fn transform_recursive(value: &mut JsonValue) -> Result<()> {
    match value {
        JsonValue::Object(map) => {
            // 检查此对象是否具有应删除的 "symbolID" 字段
            if let Some(symbol_id_value) = map.get("symbolID")
                && should_remove_symbol_id(symbol_id_value)
            {
                map.remove("symbolID");
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

/// 确定是否应删除 symbolID 字段。
///
/// 如果值是仅包含字段的对象，则返回 true
/// "localID" 和/或 "sessionID" 没有其他字段。
fn should_remove_symbol_id(value: &JsonValue) -> bool {
    if let JsonValue::Object(map) = value {
        // 检查所有键是否在允许的集合中
        for key in map.keys() {
            if key != "localID" && key != "sessionID" {
                // 发现一个不是 localID 或 sessionID 的字段
                return false;
            }
        }
        // 所有键都是 localID 或 sessionID (或为空)
        true
    } else {
        // 不是一个对象，不要删除
        false
    }
}

#[cfg(test)]
#[path = "symbol_id_removal_tests.rs"]
mod symbol_id_removal_tests;
