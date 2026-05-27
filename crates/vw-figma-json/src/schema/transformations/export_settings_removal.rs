use crate::error::Result;
use serde_json::Value as JsonValue;

/// 从 JSON 树中的所有对象中删除 exportSettings 字段
///
/// 递归遍历 JSON 树并删除所有 "exportSettings" 字段。
/// 这些字段包含 Figma 资源导出配置(SVG、PNG 设置等)
/// HTML/CSS 渲染不需要。
///
/// # 参数
/// * `tree` - 要修改的 JSON 树(通常是文档根)
///
/// # 返回值
/// * `Ok(())` - 成功删除所有导出设置字段
///
/// # 示例
/// ```no_run
/// use fig2json::schema::remove_export_settings;
/// use serde_json::json;
///
/// let mut tree = json!({
///     "name": "Icon",
///     "exportSettings": [
///         {
///             "colorProfile": "DOCUMENT",
///             "constraint": {"type": "CONTENT_SCALE", "value": 1.0},
///             "imageType": "SVG"
///         }
///     ],
///     "visible": true
/// });
/// remove_export_settings(&mut tree).unwrap();
/// // 树现在只有 "name" 和 "visible" 字段
/// ```
pub fn remove_export_settings(tree: &mut JsonValue) -> Result<()> {
    transform_recursive(tree)
}

/// 从 JSON 值中递归删除 exportSettings 字段
fn transform_recursive(value: &mut JsonValue) -> Result<()> {
    match value {
        JsonValue::Object(map) => {
            // 删除 "exportSettings" 字段(如果存在)
            map.remove("exportSettings");

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
#[path = "export_settings_removal_tests.rs"]
mod export_settings_removal_tests;
