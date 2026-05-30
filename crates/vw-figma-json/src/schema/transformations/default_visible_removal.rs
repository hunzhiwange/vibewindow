use crate::error::Result;
use serde_json::Value as JsonValue;

/// 当可见字段具有默认值 true 时删除它
///
/// 递归遍历 JSON 树并删除具有以下属性的 "visible" 字段
/// 值为真。由于 true 是 Figma 和 CSS 中的默认可见性，
/// 省略它会减少输出大小而不丢失信息。
///
/// # 参数
/// * `tree` - 要修改的 JSON 树(通常是文档根)
///
/// # 返回值
/// * `Ok(())` - 成功删除所有默认可见字段
///
/// # 示例
/// ```no_run
/// use fig2json::schema::remove_default_visible;
/// use serde_json::json;
///
/// let mut tree = json!({
///     "name": "Shape",
///     "visible": true,
///     "opacity": 0.5
/// });
/// remove_default_visible(&mut tree).unwrap();
/// // 树现在只有 "name" 和 "opacity" 字段
/// ```
pub fn remove_default_visible(tree: &mut JsonValue) -> Result<()> {
    transform_recursive(tree)
}

/// 从 JSON 值中递归删除默认可见字段
fn transform_recursive(value: &mut JsonValue) -> Result<()> {
    match value {
        JsonValue::Object(map) => {
            // 检查visible是否存在并且为true
            if let Some(visible) = map.get("visible")
                && let Some(b) = visible.as_bool()
                && b
            {
                map.remove("visible");
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

#[cfg(test)]
#[path = "default_visible_removal_tests.rs"]
mod default_visible_removal_tests;
