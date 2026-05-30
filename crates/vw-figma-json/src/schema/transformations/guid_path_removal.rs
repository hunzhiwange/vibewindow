use crate::error::Result;
use serde_json::Value as JsonValue;

/// 从 JSON 树中的所有对象中删除 guidPath 字段
///
/// 递归遍历 JSON 树并删除所有 "guidPath" 字段。
/// 这些字段包含符号中使用的内部 Figma 节点参考路径
/// 覆盖并导出数据。 HTML/CSS 渲染不需要它们。
///
/// # 参数
/// * `tree` - 要修改的 JSON 树(通常是文档根)
///
/// # 返回值
/// * `Ok(())` - 成功删除所有 guidPath 字段
///
/// # 示例
/// ```no_run
/// use fig2json::schema::remove_guid_paths;
/// use serde_json::json;
///
/// let mut tree = json!({
///     "name": "Override",
///     "guidPath": {
///         "guids": [
///             {
///                 "localID": 123,
///                 "sessionID": 456
///             }
///         ]
///     },
///     "visible": false
/// });
/// remove_guid_paths(&mut tree).unwrap();
/// // 树现在只有 "name" 和 "visible" 字段
/// ```
pub fn remove_guid_paths(tree: &mut JsonValue) -> Result<()> {
    transform_recursive(tree)
}

/// 从 JSON 值中递归删除 guidPath 字段
fn transform_recursive(value: &mut JsonValue) -> Result<()> {
    match value {
        JsonValue::Object(map) => {
            // 删除 "guidPath" 字段(如果存在)
            map.remove("guidPath");

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
#[path = "guid_path_removal_tests.rs"]
mod guid_path_removal_tests;
