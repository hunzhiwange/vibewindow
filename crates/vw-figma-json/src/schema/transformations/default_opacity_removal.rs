use crate::error::Result;
use serde_json::Value as JsonValue;

/// 当不透明度字段具有默认值 1.0 时删除它
///
/// 递归遍历 JSON 树并删除具有以下属性的 "opacity" 字段
/// 值 1.0。由于 1.0 是 Figma 和 CSS 中的默认不透明度，
/// 省略它会减少输出大小而不丢失信息。
///
/// # 参数
/// * `tree` - 要修改的 JSON 树(通常是文档根)
///
/// # 返回值
/// * `Ok(())` - 成功删除所有默认不透明度字段
///
/// # 示例
/// ```no_run
/// use fig2json::schema::remove_default_opacity;
/// use serde_json::json;
///
/// let mut tree = json!({
///     "name": "Shape",
///     "opacity": 1.0,
///     "visible": true
/// });
/// remove_default_opacity(&mut tree).unwrap();
/// // 树现在只有 "name" 和 "visible" 字段
/// ```
pub fn remove_default_opacity(tree: &mut JsonValue) -> Result<()> {
    transform_recursive(tree)
}

/// 从 JSON 值中递归删除默认不透明度字段
fn transform_recursive(value: &mut JsonValue) -> Result<()> {
    match value {
        JsonValue::Object(map) => {
            // 检查不透明度是否存在且是否为 1.0
            if let Some(opacity) = map.get("opacity")
                && let Some(n) = opacity.as_f64()
            {
                // 使用 epsilon 比较浮点
                if (n - 1.0).abs() < f64::EPSILON {
                    map.remove("opacity");
                }
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
#[path = "default_opacity_removal_tests.rs"]
mod default_opacity_removal_tests;
