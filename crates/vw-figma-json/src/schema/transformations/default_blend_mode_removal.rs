use crate::error::Result;
use serde_json::Value as JsonValue;

/// 当 BlendMode 字段具有默认值 "NORMAL" 时，删除它
///
/// 递归遍历 JSON 树并删除具有以下属性的 "blendMode" 字段
/// 值 "NORMAL" (在枚举简化将它们从枚举转换之后)
/// 对象到字符串)。 NORMAL 是 Figma 和 CSS 中的默认混合模式，
/// 因此省略它会减少输出大小而不丢失信息。
///
/// 重要提示：此转换必须在 enum_simplification 之后运行，这
/// converts `{"__enum__": "BlendMode", "value": "NORMAL"}` to `"NORMAL"`.
///
/// # 参数
/// * `tree` - 要修改的 JSON 树(通常是文档根)
///
/// # 返回值
/// * `Ok(())` - 成功删除所有默认的混合模式字段
///
/// # 示例
/// ```no_run
/// use fig2json::schema::remove_default_blend_mode;
/// use serde_json::json;
///
/// let mut tree = json!({
///     "name": "Shape",
///     "blendMode": "NORMAL",
///     "opacity": 1.0
/// });
/// remove_default_blend_mode(&mut tree).unwrap();
/// // 树现在只有 "name" 和 "opacity" 字段
/// ```
pub fn remove_default_blend_mode(tree: &mut JsonValue) -> Result<()> {
    transform_recursive(tree)
}

/// 从 JSON 值中递归删除默认的 blendMode 字段
fn transform_recursive(value: &mut JsonValue) -> Result<()> {
    match value {
        JsonValue::Object(map) => {
            // 检查 blendMode 是否存在且为 "NORMAL"
            if let Some(blend_mode) = map.get("blendMode")
                && let Some(s) = blend_mode.as_str()
                && s == "NORMAL"
            {
                map.remove("blendMode");
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
#[path = "default_blend_mode_removal_tests.rs"]
mod default_blend_mode_removal_tests;
