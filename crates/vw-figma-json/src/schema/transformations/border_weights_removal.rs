use crate::error::Result;
use serde_json::Value as JsonValue;

/// 从 JSON 树中的所有对象中删除单个边框权重字段
///
/// 递归遍历 JSON 树并删除 Figma 特定的边框权重字段：
/// - "borderTopWeight" - 顶部边框粗细
/// - "borderBottomWeight" - 底部边框粗细
/// - "borderLeftWeight" - 左边框粗细
/// - "borderRightWeight" - 右边框粗细
/// - "borderStrokeWeightsIndependent" - 指示独立边界权重的标志
///
/// 这些字段允许 Figma 中的每边边框权重，但标准 HTML/CSS
/// uses uniform borders. For HTML/CSS rendering, these detailed border weights
/// 不需要，可以安全删除。
///
/// # 参数
/// * `tree` - 要修改的 JSON 树(通常是文档根)
///
/// # 返回值
/// * `Ok(())` - 成功删除所有边框权重字段
///
/// # 示例
/// ```no_run
/// use fig2json::schema::remove_border_weights;
/// use serde_json::json;
///
/// let mut tree = json!({
///     "name": "Rectangle",
///     "borderTopWeight": 1.0,
///     "borderBottomWeight": 1.0,
///     "borderLeftWeight": 1.0,
///     "borderRightWeight": 1.0,
///     "borderStrokeWeightsIndependent": true,
///     "visible": true
/// });
/// remove_border_weights(&mut tree).unwrap();
/// // 树现在只有 "name" 和 "visible" 字段
/// ```
pub fn remove_border_weights(tree: &mut JsonValue) -> Result<()> {
    transform_recursive(tree)
}

/// 从 JSON 值中递归删除边框权重字段
fn transform_recursive(value: &mut JsonValue) -> Result<()> {
    match value {
        JsonValue::Object(map) => {
            // 删除所有边框权重字段(如果存在)
            map.remove("borderTopWeight");
            map.remove("borderBottomWeight");
            map.remove("borderLeftWeight");
            map.remove("borderRightWeight");
            map.remove("borderStrokeWeightsIndependent");

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
#[path = "border_weights_removal_tests.rs"]
mod border_weights_removal_tests;
