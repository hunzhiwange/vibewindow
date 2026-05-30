use crate::error::Result;
use serde_json::Value as JsonValue;

/// 当其默认值为 1.0 时，删除uniformScaleFactor字段
///
/// 递归遍历 JSON 树并删除具有 "uniformScaleFactor" 字段
/// 值 1.0。这是默认比例因子，因此省略它会减少输出大小
/// 而不丢失信息。
///
/// # 参数
/// * `tree` - 要修改的 JSON 树(通常是文档根)
///
/// # 返回值
/// * `Ok(())` - 成功删除所有默认的uniformScaleFactor字段
///
/// # 示例
/// ```no_run
/// use fig2json::schema::remove_default_uniform_scale_factor;
/// use serde_json::json;
///
/// let mut tree = json!({
///     "name": "Shape",
///     "uniformScaleFactor": 1.0,
///     "width": 100
/// });
/// remove_default_uniform_scale_factor(&mut tree).unwrap();
/// // 树现在只有 "name" 和 "width" 字段
/// ```
pub fn remove_default_uniform_scale_factor(tree: &mut JsonValue) -> Result<()> {
    transform_recursive(tree)
}

/// 从 JSON 值中递归删除默认的uniformScaleFactor字段
fn transform_recursive(value: &mut JsonValue) -> Result<()> {
    match value {
        JsonValue::Object(map) => {
            // 检查uniformScaleFactor是否存在并且是否为1.0
            if let Some(scale_factor) = map.get("uniformScaleFactor")
                && let Some(num) = scale_factor.as_f64()
            {
                // 如果恰好为 1.0，则删除
                if (num - 1.0).abs() < f64::EPSILON {
                    map.remove("uniformScaleFactor");
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
#[path = "uniform_scale_factor_removal_tests.rs"]
mod uniform_scale_factor_removal_tests;
