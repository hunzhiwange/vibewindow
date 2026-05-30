use crate::error::Result;
use serde_json::Value as JsonValue;

/// 从 JSON 树中的所有对象中删除后台元数据字段
///
/// 递归遍历JSON树并删除后台相关元数据：
/// - "backgroundEnabled" - 是否启用背景(与背景颜色存在冗余)
/// - "backgroundOpacity" - 背景不透明度(应该在颜色 Alpha 通道中)
///
/// 这些字段包含冗余或应该表示的元数据
/// 对于 HTML/CSS 渲染有所不同。
///
/// # 参数
/// * `tree` - 要修改的 JSON 树(通常是文档根)
///
/// # 返回值
/// * `Ok(())` - 成功删除所有背景属性字段
///
/// # 示例
/// ```no_run
/// use fig2json::schema::remove_background_properties;
/// use serde_json::json;
///
/// let mut tree = json!({
///     "name": "Canvas",
///     "backgroundColor": "#f5f5f5",
///     "backgroundEnabled": true,
///     "backgroundOpacity": 1.0
/// });
/// remove_background_properties(&mut tree).unwrap();
/// // 树现在只有 "name" 和 "backgroundColor" 字段
/// ```
pub fn remove_background_properties(tree: &mut JsonValue) -> Result<()> {
    transform_recursive(tree)
}

/// 从 JSON 值中递归删除背景属性字段
fn transform_recursive(value: &mut JsonValue) -> Result<()> {
    match value {
        JsonValue::Object(map) => {
            // 删除背景属性字段(如果存在)
            map.remove("backgroundEnabled");
            map.remove("backgroundOpacity");

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
#[path = "background_properties_removal_tests.rs"]
mod background_properties_removal_tests;
