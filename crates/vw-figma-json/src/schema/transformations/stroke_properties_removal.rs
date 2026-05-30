use crate::error::Result;
use serde_json::Value as JsonValue;

/// 从 JSON 树中的所有对象中删除笔画属性字段
///
/// 递归遍历JSON树并删除笔画相关字段：
/// - "strokeAlign" - CSS 不支持笔画对齐(内部/中心/外部)
/// - "strokeJoin" - 笔划连接样式(MITRE/BEVEL/ROUND)
/// - "strokeWeight" - 描边宽度值
///
/// 这些字段包含没有直接 CSS 等效项的笔划属性
/// 或对于基本 HTML/CSS 渲染来说不是必需的。
///
/// # 参数
/// * `tree` - 要修改的 JSON 树(通常是文档根)
///
/// # 返回值
/// * `Ok(())` - 成功删除所有笔画属性字段
///
/// # 示例
/// ```no_run
/// use fig2json::schema::remove_stroke_properties;
/// use serde_json::json;
///
/// let mut tree = json!({
///     "name": "Rectangle",
///     "strokeAlign": {
///         "__enum__": "StrokeAlign",
///         "value": "INSIDE"
///     },
///     "strokeJoin": {
///         "__enum__": "StrokeJoin",
///         "value": "MITER"
///     },
///     "strokeWeight": 1.0,
///     "visible": true
/// });
/// remove_stroke_properties(&mut tree).unwrap();
/// // 树现在只有 "name" 和 "visible" 字段
/// ```
pub fn remove_stroke_properties(tree: &mut JsonValue) -> Result<()> {
    transform_recursive(tree)
}

/// 从 JSON 值中递归删除笔划属性字段
fn transform_recursive(value: &mut JsonValue) -> Result<()> {
    match value {
        JsonValue::Object(map) => {
            // 删除笔画属性字段(如果存在)
            map.remove("strokeAlign");
            map.remove("strokeJoin");
            map.remove("strokeWeight");

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
#[path = "stroke_properties_removal_tests.rs"]
mod stroke_properties_removal_tests;
