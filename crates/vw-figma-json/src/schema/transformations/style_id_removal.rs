use crate::error::Result;
use serde_json::Value as JsonValue;

/// 从 JSON 树中的所有对象中删除样式 ID 引用字段
///
/// 递归遍历 JSON 树并删除 Figma 共享样式引用：
/// - "styleIdForFill" - 填充paint样式的参考
/// - "styleIdForText" - 参考文本样式
/// - "styleIdForStrokeFill" - 参考描边paint样式
///
/// 这些字段引用 Figma 的共享样式库。实际样式值
/// 已内联在节点属性中，因此不需要这些引用
/// 用于 HTML/CSS 渲染。
///
/// # 参数
/// * `tree` - 要修改的 JSON 树(通常是文档根)
///
/// # 返回值
/// * `Ok(())` - 成功删除所有样式 ID 字段
///
/// # 示例
/// ```no_run
/// use fig2json::schema::remove_style_ids;
/// use serde_json::json;
///
/// let mut tree = json!({
///     "name": "Text",
///     "fillPaints": [{"color": "#ffffff", "type": "SOLID"}],
///     "styleIdForFill": {
///         "assetRef": {
///             "key": "abc123",
///             "version": "1:101"
///         }
///     },
///     "fontSize": 14.0
/// });
/// remove_style_ids(&mut tree).unwrap();
/// // 树现在有 "name"、"fillPaints" 和 "fontSize" 字段
/// ```
pub fn remove_style_ids(tree: &mut JsonValue) -> Result<()> {
    transform_recursive(tree)
}

/// 从 JSON 值中递归删除样式 ID 字段
fn transform_recursive(value: &mut JsonValue) -> Result<()> {
    match value {
        JsonValue::Object(map) => {
            // 删除所有样式 ID 字段(如果存在)
            map.remove("styleIdForFill");
            map.remove("styleIdForText");
            map.remove("styleIdForStrokeFill");

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
#[path = "style_id_removal_tests.rs"]
mod style_id_removal_tests;
