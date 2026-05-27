use crate::error::Result;
use serde_json::Value as JsonValue;

/// 当旋转字段具有默认值 0.0 时删除它
///
/// 递归遍历 JSON 树并删除具有以下属性的 "rotation" 字段
/// 值 0.0。由于 0.0 是 Figma 中的默认旋转(不旋转)
/// 和 CSS，省略它会减少输出大小而不丢失信息。
///
/// 这通常出现在图像绘制变换和其他变换上下文中。
///
/// # 参数
/// * `tree` - 要修改的 JSON 树(通常是文档根)
///
/// # 返回值
/// * `Ok(())` - 成功删除所有默认旋转字段
///
/// # 示例
/// ```no_run
/// use fig2json::schema::remove_default_rotation;
/// use serde_json::json;
///
/// let mut tree = json!({
///     "image": {
///         "rotation": 0.0,
///         "scale": 0.5
///     }
/// });
/// remove_default_rotation(&mut tree).unwrap();
/// // 图像现在只有 "scale" 字段
/// ```
pub fn remove_default_rotation(tree: &mut JsonValue) -> Result<()> {
    transform_recursive(tree)
}

/// 从 JSON 值中递归删除默认旋转字段
fn transform_recursive(value: &mut JsonValue) -> Result<()> {
    match value {
        JsonValue::Object(map) => {
            // 检查旋转是否存在且为0.0
            if let Some(rotation) = map.get("rotation")
                && let Some(n) = rotation.as_f64()
            {
                // 使用 epsilon 比较浮点
                if n.abs() < f64::EPSILON {
                    map.remove("rotation");
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
#[path = "default_rotation_removal_tests.rs"]
mod default_rotation_removal_tests;
