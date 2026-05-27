use crate::error::Result;
use serde_json::Value as JsonValue;

/// 从 JSON 树中删除文档级属性字段
///
/// 递归遍历 JSON 树并删除文档级配置：
/// - "documentColorProfile" - 颜色配置文件设置(SRGB 等)
///
/// 这些字段包含文档级元数据，这些元数据不需要
/// 基本 HTML/CSS 渲染。
///
/// # 参数
/// * `tree` - 要修改的 JSON 树(通常是文档根)
///
/// # 返回值
/// * `Ok(())` - 成功删除所有文档属性字段
///
/// # 示例
/// ```no_run
/// use fig2json::schema::remove_document_properties;
/// use serde_json::json;
///
/// let mut tree = json!({
///     "document": {
///         "name": "Document",
///         "documentColorProfile": {
///             "__enum__": "DocumentColorProfile",
///             "value": "SRGB"
///         }
///     }
/// });
/// remove_document_properties(&mut tree).unwrap();
/// // 文档现在只有 "name" 字段
/// ```
pub fn remove_document_properties(tree: &mut JsonValue) -> Result<()> {
    transform_recursive(tree)
}

/// 从 JSON 值中递归删除文档属性字段
fn transform_recursive(value: &mut JsonValue) -> Result<()> {
    match value {
        JsonValue::Object(map) => {
            // 删除文档属性字段(如果存在)
            map.remove("documentColorProfile");

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
#[path = "document_properties_removal_tests.rs"]
mod document_properties_removal_tests;
