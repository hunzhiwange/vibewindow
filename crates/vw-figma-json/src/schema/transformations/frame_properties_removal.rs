use crate::error::Result;
use serde_json::Value as JsonValue;

/// 从 JSON 树中的所有对象中删除框架属性字段
///
/// 递归遍历 JSON 树并删除特定于帧的字段：
/// - "frameMaskDisabled" - 帧掩码禁用标志
/// - "targetAspectRatio" - 框架的目标纵横比
///
/// 这些字段包含帧特定的配置，不需要
/// 基本 HTML/CSS 渲染。
///
/// # 参数
/// * `tree` - 要修改的 JSON 树(通常是文档根)
///
/// # 返回值
/// * `Ok(())` - 成功删除所有框架属性字段
///
/// # 示例
/// ```no_run
/// use fig2json::schema::remove_frame_properties;
/// use serde_json::json;
///
/// let mut tree = json!({
///     "name": "Frame",
///     "frameMaskDisabled": false,
///     "targetAspectRatio": {
///         "value": {
///             "x": 300.0,
///             "y": 300.0
///         }
///     },
///     "visible": true
/// });
/// remove_frame_properties(&mut tree).unwrap();
/// // 树现在只有 "name" 和 "visible" 字段
/// ```
pub fn remove_frame_properties(tree: &mut JsonValue) -> Result<()> {
    transform_recursive(tree)
}

/// 从 JSON 值中递归删除框架属性字段
fn transform_recursive(value: &mut JsonValue) -> Result<()> {
    match value {
        JsonValue::Object(map) => {
            // 删除框架属性字段(如果存在)
            map.remove("frameMaskDisabled");
            map.remove("targetAspectRatio");

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
#[path = "frame_properties_removal_tests.rs"]
mod frame_properties_removal_tests;
