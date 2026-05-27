use crate::error::Result;
use serde_json::Value as JsonValue;

/// 从文档树中删除 Figma 组件实例元数据。
///
/// 此转换删除 `detachedSymbolId` 字段，其中包含
/// 引用已从其组件分离的 Figma 组件实例
/// 主要成分。该字段通常包含一个 `assetRef` 对象
/// `key` 和 `version` 属性。
///
/// 该元数据特定于 Figma 的组件系统，不需要
/// 用于 HTML/CSS 渲染。
///
/// # 示例
///
/// ```rust
/// use serde_json::json;
/// use fig2json::schema::remove_detached_symbol_id;
///
/// let mut tree = json!({
///     "name": "Frame",
///     "detachedSymbolId": {
///         "assetRef": {
///             "key": "b12947c871f268e97f688eb784bcf92431d9b6df",
///             "version": "186:107"
///         }
///     },
///     "type": "FRAME"
/// });
///
/// remove_detached_symbol_id(&mut tree).unwrap();
///
/// assert!(tree.get("detachedSymbolId").is_none());
/// assert!(tree.get("type").is_some());
/// ```
pub fn remove_detached_symbol_id(tree: &mut JsonValue) -> Result<()> {
    transform_recursive(tree)
}

fn transform_recursive(value: &mut JsonValue) -> Result<()> {
    match value {
        JsonValue::Object(map) => {
            // 删除 detachedSymbolId
            map.remove("detachedSymbolId");

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
#[path = "detached_symbol_id_removal_tests.rs"]
mod detached_symbol_id_removal_tests;
