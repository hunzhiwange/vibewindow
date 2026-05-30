use crate::error::Result;
use serde_json::Value as JsonValue;

/// 从 JSON 树中的所有对象中删除 userFacingVersion 字段
///
/// 递归遍历 JSON 树并删除所有 "userFacingVersion" 字段。
/// 这些字段包含 HTML/CSS 渲染不需要的 Figma 版本字符串。
///
/// # 参数
/// * `tree` - 要修改的 JSON 树(通常是文档根)
///
/// # 返回值
/// * `Ok(())` - 成功删除所有 userFacingVersion 字段
///
/// # 示例
/// ```no_run
/// use fig2json::schema::remove_user_facing_versions;
/// use serde_json::json;
///
/// let mut tree = json!({
///     "name": "Component",
///     "type": "INSTANCE",
///     "userFacingVersion": "434:917",
///     "visible": true
/// });
/// remove_user_facing_versions(&mut tree).unwrap();
/// // 树现在只有 "name"、"type" 和 "visible" 字段
/// ```
pub fn remove_user_facing_versions(tree: &mut JsonValue) -> Result<()> {
    transform_recursive(tree)
}

/// 从 JSON 值中递归删除 userFacingVersion 字段
fn transform_recursive(value: &mut JsonValue) -> Result<()> {
    match value {
        JsonValue::Object(map) => {
            // 删除 "userFacingVersion" 字段(如果存在)
            map.remove("userFacingVersion");

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
#[path = "user_facing_version_removal_tests.rs"]
mod user_facing_version_removal_tests;
